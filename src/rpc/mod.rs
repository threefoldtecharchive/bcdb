use crate::acl::*;
use crate::database::{Database, Meta, Reason};
use crate::identity::Identity;
use anyhow::Error;
use generated::acl_server::Acl as AclServiceTrait;
use generated::bcdb_server::Bcdb as BcdbServiceTrait;
use generated::identity_server::Identity as IdentityTrait;
use generated::*;
use std::collections::HashSet;
use std::iter::FromIterator;
use tokio::sync::mpsc;
use tonic::{Code, Request, Response, Status};

use crate::auth::MetadataMapExt;
use crate::storage::{zdb::Collection, zdb::Zdb, Storage as ObjectStorage};

pub use generated::acl_server::AclServer;
pub use generated::bcdb_server::BcdbServer;
pub use generated::identity_server::IdentityServer;

pub mod generated {
    tonic::include_proto!("bcdb"); // The string specified here must match the proto package name
}

trait FailureExt {
    fn status(&self) -> Status;
}

impl FailureExt for Error {
    fn status(&self) -> Status {
        match Reason::from(self) {
            Reason::Unauthorized => Status::unauthenticated("unauthorized"),
            Reason::NotFound => Status::not_found("object not found"),
            Reason::NotSupported => Status::unimplemented("operation not supported"),
            Reason::InvalidTag => Status::invalid_argument(
                "use of invalid tag string (':' prefix is for internal use)",
            ),
            Reason::CannotGetPeer(m) => Status::unavailable(m),
            Reason::Unknown(m) => Status::internal(m),
        }
    }
}

type ListStream = mpsc::Receiver<Result<ListResponse, Status>>;
type FindStream = mpsc::Receiver<Result<FindResponse, Status>>;

//TODO: use generics for both object store type and meta factory type.
pub struct BcdbService<D>
where
    D: Database,
{
    db: D,
}

impl<D> BcdbService<D>
where
    D: Database + Clone,
{
    pub fn new(db: D) -> Self {
        BcdbService { db: db }
    }

    fn build_meta(metadata: Meta) -> Metadata {
        //build metadata for storage
        let collection = metadata.collection().unwrap_or_default();
        let acl = metadata.acl().map(|acl| AclRef { acl });
        Metadata {
            tags: metadata.into(),
            collection: collection,
            acl: acl,
        }
    }
}

#[tonic::async_trait]
impl<D> BcdbServiceTrait for BcdbService<D>
where
    D: Database + Clone,
{
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let context = request.metadata().context();

        let request = request.into_inner();

        let data = request.data;
        let metadata = match request.metadata {
            Some(metadata) => metadata,
            None => return Err(Status::invalid_argument("metadata is required")),
        };

        let acl = metadata.acl.map(|a| a.acl);

        let mut db = self.db.clone();
        let id = db
            .set(context, metadata.collection, data, metadata.tags, acl)
            .await
            .map_err(|e| e.status())?;

        Ok(Response::new(SetResponse { id }))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();
        let id = request.id;

        let mut db = self.db.clone();
        let object = db
            .get(ctx, id, request.collection)
            .await
            .map_err(|e| e.status())?;

        Ok(Response::new(GetResponse {
            data: object.data.unwrap_or_default(), // This unwrap is safe as we checked the none case above
            metadata: Some(Self::build_meta(object.meta)),
        }))
    }

    async fn fetch(&self, request: Request<FetchRequest>) -> Result<Response<GetResponse>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();
        let id = request.id;

        let mut db = self.db.clone();
        let object = db.fetch(ctx, id).await.map_err(|e| e.status())?;

        Ok(Response::new(GetResponse {
            data: object.data.unwrap_or_default(), // This unwrap is safe as we checked the none case above
            metadata: Some(Self::build_meta(object.meta)),
        }))
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();
        let id = request.id;

        let mut db = self.db.clone();
        db.delete(ctx, id, request.collection)
            .await
            .map_err(|e| e.status())?;

        Ok(Response::new(DeleteResponse {}))
    }

    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();

        let id = request.id;
        let data = request.data;
        let metadata = match request.metadata {
            Some(metadata) => metadata,
            None => return Err(Status::invalid_argument("metadata is required")),
        };

        let acl = metadata.acl.map(|a| a.acl);

        let mut db = self.db.clone();
        let _ = db
            .update(
                ctx,
                id,
                metadata.collection,
                data.map(|d| d.data),
                metadata.tags,
                acl,
            )
            .await
            .map_err(|e| e.status())?;

        Ok(Response::new(UpdateResponse {}))
    }

    type ListStream = ListStream;

    async fn list(&self, request: Request<QueryRequest>) -> Result<Response<ListStream>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();

        let mut db = self.db.clone();

        let mut results = db
            .list(ctx, request.tags, Some(request.collection))
            .await
            .map_err(|e| e.status())?;

        let (mut tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            while let Some(id) = results.recv().await {
                match id {
                    Ok(id) => tx.send(Ok(ListResponse { id: id })).await.unwrap(),
                    Err(err) => tx.send(Err(err.status())).await.unwrap(),
                }
            }
        });

        Ok(Response::new(rx))
    }

    type FindStream = FindStream;

    async fn find(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::FindStream>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();

        let mut db = self.db.clone();

        let mut results = db
            .find(ctx, request.tags, Some(request.collection))
            .await
            .map_err(|e| e.status())?;

        let (mut tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            while let Some(object) = results.recv().await {
                match object {
                    Ok(object) => tx
                        .send(Ok(FindResponse {
                            id: object.key,
                            metadata: Some(Self::build_meta(object.meta)),
                        }))
                        .await
                        .unwrap(),
                    Err(err) => tx.send(Err(err.status())).await.unwrap(),
                }
            }
        });

        Ok(Response::new(rx))
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
        let ctx = request.metadata().context();

        if !ctx.is_owner() {
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

        let ctx = request.metadata().context();

        if !ctx.is_owner() {
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
        let ctx = request.metadata().context();

        if !ctx.is_owner() {
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
        let ctx = request.metadata().context();

        if !ctx.is_owner() {
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
        let ctx = request.metadata().context();

        if !ctx.is_owner() {
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
        let ctx = request.metadata().context();

        if !ctx.is_owner() {
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
