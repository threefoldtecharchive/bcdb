use crate::acl::*;
use crate::identity::Identity;
use failure::Error;
use generated::acl_server::Acl as AclServiceTrait;
use generated::bcdb_client::BcdbClient;
use generated::bcdb_server::Bcdb as BcdbServiceTrait;
use generated::identity_server::Identity as IdentityTrait;
use generated::*;
use std::collections::HashSet;
use std::iter::FromIterator;
use tokio::sync::mpsc;
use tonic::{Code, Request, Response, Status};

use crate::auth::{MetadataMapExt, Route};
use crate::storage::{zdb::Collection, zdb::Zdb, Storage as ObjectStorage};

pub use generated::acl_server::AclServer;
pub use generated::bcdb_server::BcdbServer;
pub use generated::identity_server::IdentityServer;

pub mod generated {
    tonic::include_proto!("bcdb"); // The string specified here must match the proto package name
}

mod local;
mod peers;

pub use local::LocalBcdb;
pub use peers::{Either, Explorer, PeersFile, PeersList, Tracker};

trait FailureExt {
    fn status(&self) -> Status;
}

impl FailureExt for Error {
    fn status(&self) -> Status {
        Status::new(Code::Internal, format!("{}", self))
    }
}

type ListStream = mpsc::Receiver<Result<ListResponse, Status>>;
type FindStream = mpsc::Receiver<Result<FindResponse, Status>>;

//TODO: use generics for both object store type and meta factory type.
pub struct BcdbService<L, P>
where
    L: BcdbServiceTrait,
    P: PeersList,
{
    /// id is the 3bot id of this bcdb instance
    id: u32,
    local: L,
    peers: P,
}

impl<L, P> BcdbService<L, P>
where
    L: BcdbServiceTrait,
    P: PeersList,
{
    pub fn new(id: u32, local: L, peers: P) -> Self {
        BcdbService {
            id: id,
            local: local,
            peers: peers,
        }
    }

    async fn get_peer(
        &self,
        id: u32,
    ) -> Result<BcdbClient<tonic::transport::channel::Channel>, Status> {
        let peer = match self.peers.get(id).await {
            Ok(peer) => peer,
            Err(err) => {
                return Err(Status::unavailable(format!(
                    "could not find peer '{}': {}",
                    id, err
                )))
            }
        };

        let con = match peer.connect().await {
            Ok(con) => con,
            Err(err) => {
                return Err(Status::unavailable(format!(
                    "could not reach peer '{}': {}",
                    id, err
                )));
            }
        };

        Ok(BcdbClient::new(con))
    }

    async fn proxy_set(
        &self,
        id: u32,
        request: Request<SetRequest>,
    ) -> Result<Response<SetResponse>, Status> {
        let mut client = self.get_peer(id).await?;

        client.set(request).await
    }

    async fn proxy_get(
        &self,
        id: u32,
        request: Request<GetRequest>,
    ) -> Result<Response<GetResponse>, Status> {
        let mut client = self.get_peer(id).await?;

        client.get(request).await
    }

    async fn proxy_fetch(
        &self,
        id: u32,
        request: Request<FetchRequest>,
    ) -> Result<Response<GetResponse>, Status> {
        let mut client = self.get_peer(id).await?;

        client.fetch(request).await
    }

    async fn proxy_delete(
        &self,
        id: u32,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let mut client = self.get_peer(id).await?;

        client.delete(request).await
    }

    async fn proxy_update(
        &self,
        id: u32,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let mut client = self.get_peer(id).await?;

        client.update(request).await
    }

    async fn proxy_list(
        &self,
        id: u32,
        request: Request<QueryRequest>,
    ) -> Result<Response<ListStream>, Status> {
        let mut client = self.get_peer(id).await?;

        let stream = client.list(request).await?;
        let mut inbound = stream.into_inner();

        let (mut tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            while let Some(obj) = inbound.message().await.unwrap() {
                tx.send(Ok(obj)).await.unwrap();
            }
        });

        Ok(Response::new(rx))
    }

    async fn proxy_find(
        &self,
        id: u32,
        request: Request<QueryRequest>,
    ) -> Result<Response<FindStream>, Status> {
        let mut client = self.get_peer(id).await?;

        let stream = client.find(request).await?;
        let mut inbound = stream.into_inner();

        let (mut tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            while let Some(obj) = inbound.message().await.unwrap() {
                tx.send(Ok(obj)).await.unwrap();
            }
        });

        Ok(Response::new(rx))
    }
}

#[tonic::async_trait]
impl<L, P> BcdbServiceTrait for BcdbService<L, P>
where
    L: BcdbServiceTrait<ListStream = ListStream, FindStream = FindStream>,
    P: PeersList,
{
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        match request.metadata().route(self.id)? {
            Route::Local if request.metadata().is_authenticated() => self.local.set(request).await,
            Route::Proxy(id) => self.proxy_set(id, request).await,
            _ => Err(Status::unauthenticated("unauthenticated request")),
        }
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        match request.metadata().route(self.id)? {
            Route::Local if request.metadata().is_authenticated() => self.local.get(request).await,
            Route::Proxy(id) => self.proxy_get(id, request).await,
            _ => Err(Status::unauthenticated("unauthenticated request")),
        }
    }

    async fn fetch(&self, request: Request<FetchRequest>) -> Result<Response<GetResponse>, Status> {
        match request.metadata().route(self.id)? {
            Route::Local if request.metadata().is_authenticated() => {
                self.local.fetch(request).await
            }
            Route::Proxy(id) => self.proxy_fetch(id, request).await,
            _ => Err(Status::unauthenticated("unauthenticated request")),
        }

        //return Err(Status::unimplemented("not implemented"));
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        match request.metadata().route(self.id)? {
            Route::Local if request.metadata().is_authenticated() => {
                self.local.delete(request).await
            }
            Route::Proxy(id) => self.proxy_delete(id, request).await,
            _ => Err(Status::unauthenticated("unauthenticated request")),
        }
    }

    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        match request.metadata().route(self.id)? {
            Route::Local if request.metadata().is_authenticated() => {
                self.local.update(request).await
            }
            Route::Proxy(id) => self.proxy_update(id, request).await,
            _ => Err(Status::unauthenticated("unauthenticated request")),
        }
    }

    type ListStream = ListStream;
    async fn list(&self, request: Request<QueryRequest>) -> Result<Response<ListStream>, Status> {
        match request.metadata().route(self.id)? {
            Route::Local if request.metadata().is_authenticated() => self.local.list(request).await,
            Route::Proxy(id) => self.proxy_list(id, request).await,
            _ => Err(Status::unauthenticated("unauthenticated request")),
        }
    }

    type FindStream = FindStream;
    async fn find(&self, request: Request<QueryRequest>) -> Result<Response<FindStream>, Status> {
        match request.metadata().route(self.id)? {
            Route::Local if request.metadata().is_authenticated() => self.local.find(request).await,
            Route::Proxy(id) => self.proxy_find(id, request).await,
            _ => Err(Status::unauthenticated("unauthenticated request")),
        }
    }
}

pub struct AclService<S>
where
    S: ObjectStorage,
{
    store: ACLStorage<S>,
}

impl Default for AclService<Collection> {
    fn default() -> AclService<Collection> {
        AclService {
            store: ACLStorage::new(Zdb::default().collection("acl")),
        }
    }
}

impl<S> AclService<S>
where
    S: ObjectStorage,
{
    pub fn new(store: ACLStorage<S>) -> AclService<S> {
        AclService { store: store }
    }
}

#[tonic::async_trait]
impl<S> AclServiceTrait for AclService<S>
where
    S: ObjectStorage + Send + Sync + 'static,
{
    async fn get(
        &self,
        request: Request<AclGetRequest>,
    ) -> Result<Response<AclGetResponse>, Status> {
        let meta = request.metadata();

        if !meta.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let request = request.into_inner();
        let mut store = self.store.clone();

        let acl = match store.get(request.key) {
            Ok(acl) => match acl {
                Some(acl) => acl,
                None => return Err(Status::not_found("acl not found")),
            },
            Err(err) => {
                return Err(err.status());
            }
        };

        Ok(Response::new(AclGetResponse {
            acl: Some(Acl {
                perm: acl.perm.to_string(),
                users: acl.users,
            }),
        }))
    }

    async fn create(
        &self,
        request: Request<AclCreateRequest>,
    ) -> Result<Response<AclCreateResponse>, Status> {
        let meta = request.metadata();

        if !meta.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let request = match request.into_inner().acl {
            Some(request) => request,
            None => return Err(Status::invalid_argument("missing acl")),
        };

        let set: HashSet<u64> = HashSet::from_iter(request.users);

        let acl = ACL {
            perm: match request.perm.parse() {
                Ok(perm) => perm,
                Err(err) => return Err(err.status()),
            },
            users: Vec::from_iter(set.into_iter()),
        };

        let mut store = self.store.clone();
        match store.create(&acl) {
            Ok(k) => Ok(Response::new(AclCreateResponse { key: k })),
            Err(err) => Err(err.status()),
        }
    }

    type ListStream = mpsc::Receiver<Result<AclListResponse, Status>>;

    async fn list(
        &self,
        request: Request<AclListRequest>,
    ) -> Result<Response<Self::ListStream>, Status> {
        let meta = request.metadata();

        if !meta.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let (mut tx, rx) = mpsc::channel(10);
        let mut store = self.store.clone();

        tokio::spawn(async move {
            let iter = match store.list() {
                Ok(iter) => iter,
                Err(err) => {
                    tx.send(Err(err.status())).await.unwrap();
                    return;
                }
            };

            for item in iter {
                let (key, acl) = match item {
                    Ok(item) => item,
                    Err(err) => {
                        tx.send(Err(err.status())).await.unwrap();
                        return;
                    }
                };

                tx.send(Ok(AclListResponse {
                    key: key,
                    acl: Some(Acl {
                        perm: acl.perm.to_string(),
                        users: acl.users,
                    }),
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(rx))
    }

    async fn set(
        &self,
        request: Request<AclSetRequest>,
    ) -> Result<Response<AclSetResponse>, Status> {
        let meta = request.metadata();

        if !meta.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let request = request.into_inner();
        let perm: Permissions = match request.perm.parse() {
            Ok(perm) => perm,
            Err(err) => {
                return Err(Status::invalid_argument(format!(
                    "failed to parse permission '{}': {}",
                    request.perm, err
                )))
            }
        };

        let mut store = self.store.clone();
        let mut acl = match store.get(request.key) {
            Ok(acl) => match acl {
                Some(acl) => acl,
                None => return Err(Status::not_found("no acl found with key")),
            },
            Err(err) => return Err(err.status()),
        };

        acl.perm = perm;

        match store.update(request.key, &acl) {
            Ok(_) => Ok(Response::new(AclSetResponse {})),
            Err(err) => Err(err.status()),
        }
    }

    async fn grant(
        &self,
        request: Request<AclUsersRequest>,
    ) -> Result<Response<AclUsersResponse>, Status> {
        let meta = request.metadata();

        if !meta.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let request = request.into_inner();

        let mut store = self.store.clone();
        let mut acl = match store.get(request.key) {
            Ok(acl) => match acl {
                Some(acl) => acl,
                None => return Err(Status::not_found("no acl found with key")),
            },
            Err(err) => return Err(err.status()),
        };

        let mut set: HashSet<u64> = HashSet::from_iter(acl.users);
        let len = set.len();
        request.users.iter().for_each(|u| {
            set.insert(*u);
        });

        let updated = set.len() - len;
        if updated > 0 {
            acl.users = Vec::from_iter(set.into_iter());

            match store.update(request.key, &acl) {
                Ok(_) => Ok(Response::new(AclUsersResponse {
                    updated: updated as u64,
                })),
                Err(err) => Err(err.status()),
            }
        } else {
            Ok(Response::new(AclUsersResponse { updated: 0 }))
        }
    }
    async fn revoke(
        &self,
        request: Request<AclUsersRequest>,
    ) -> Result<Response<AclUsersResponse>, Status> {
        let meta = request.metadata();

        if !meta.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let request = request.into_inner();

        let mut store = self.store.clone();
        let mut acl = match store.get(request.key) {
            Ok(acl) => match acl {
                Some(acl) => acl,
                None => return Err(Status::not_found("no acl found with key")),
            },
            Err(err) => return Err(err.status()),
        };

        let mut set: HashSet<u64> = HashSet::from_iter(acl.users);
        let len = set.len();
        request.users.iter().for_each(|u| {
            set.remove(u);
        });

        let updated = len - set.len();
        if updated > 0 {
            acl.users = Vec::from_iter(set.into_iter());

            match store.update(request.key, &acl) {
                Ok(_) => Ok(Response::new(AclUsersResponse {
                    updated: updated as u64,
                })),
                Err(err) => Err(err.status()),
            }
        } else {
            Ok(Response::new(AclUsersResponse { updated: 0 }))
        }
    }
}

pub struct IdentityService {
    id: Identity,
}

impl IdentityService {
    pub fn new(id: Identity) -> IdentityService {
        IdentityService { id }
    }

    fn identity_info(&self) -> IdentityInfo {
        IdentityInfo {
            id: self.id.id(),
            key: hex::encode(self.id.public_key().as_bytes()),
        }
    }
}

#[tonic::async_trait]
impl IdentityTrait for IdentityService {
    async fn info(&self, _request: Request<InfoRequest>) -> Result<Response<InfoResponse>, Status> {
        Ok(Response::new(InfoResponse {
            identity: Some(self.identity_info()),
        }))
    }

    async fn sign(&self, request: Request<SignRequest>) -> Result<Response<SignResponse>, Status> {
        return Err(Status::unimplemented("disabled for security reasons"));

        let request = request.into_inner();
        let signature = self.id.sign(&request.message);

        Ok(Response::new(SignResponse {
            identity: Some(self.identity_info()),
            signature: signature.to_bytes().to_vec(),
        }))
    }
}
