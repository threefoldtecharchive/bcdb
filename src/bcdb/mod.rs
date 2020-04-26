use failure::Error;
use generated::acl_server::Acl as AclServiceTrait;
use generated::bcdb_server::Bcdb as BcdbServiceTrait;
use generated::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use tonic::{Code, Request, Response, Status};

use crate::auth::MetadataMapAuth;
use crate::meta::sqlite::SqliteMetaStoreFactory;
use crate::meta::{self, Storage as MetaStorage, StorageFactory as MetaStorageFactory};
use crate::storage::{zdb::Collection, zdb::Zdb, Storage as ObjectStorage};

pub use generated::acl_server::AclServer;
pub use generated::bcdb_server::BcdbServer;

pub mod generated {
    tonic::include_proto!("bcdb"); // The string specified here must match the proto package name
}

trait FailureExt {
    fn status(&self) -> Status;
}

impl FailureExt for Error {
    fn status(&self) -> Status {
        Status::new(Code::Internal, format!("{}", self))
    }
}

//TODO: use generics for both object store type and meta factory type.
pub struct BcdbService<S, M>
where
    S: ObjectStorage,
    M: MetaStorageFactory,
{
    db: S,
    factory: M,
    collections: Arc<Mutex<HashMap<String, M::Storage>>>,
    //NOTE: acl storage doesn't have to use the same underlying storage type S
    acl: ACLStorage<S>,
}

impl<S, M> BcdbService<S, M>
where
    S: ObjectStorage,
    M: MetaStorageFactory,
{
    pub fn new(db: S, factory: M, acl: ACLStorage<S>) -> BcdbService<S, M> {
        BcdbService {
            db,
            factory,
            collections: Arc::new(Mutex::new(HashMap::new())),
            acl: acl,
        }
    }

    async fn get_collection(&self, collection: &str) -> Result<M::Storage, Error> {
        let mut stores = self.collections.lock().await;
        if let Some(store) = stores.get(collection) {
            return Ok(store.clone());
        }

        let store = self.factory.new(collection).await?;
        stores.insert(collection.into(), store.clone());

        Ok(store)
    }

    fn get_permissions(&self, acl: u64, user: u64) -> Result<Permissions, Error> {
        // self.acl.g
        let mut store = self.acl.clone();
        let acl = match store.get(acl as u32)? {
            Some(acl) => acl,
            None => return Ok(Permissions::default()),
        };

        if acl.users.contains(&user) {
            return Ok(acl.perm);
        }

        Ok(Permissions::default())
    }

    fn is_authorized(&self, acl: u64, user: u64, perm: Permissions) -> Result<(), Status> {
        let stored = match self.get_permissions(acl, user) {
            Ok(stored) => stored,
            Err(err) => {
                return Err(Status::unauthenticated(format!(
                    "failed to get assigned permissions: {}",
                    err
                )))
            }
        };
        if stored.grants(perm) {
            return Ok(());
        }

        return Err(Status::unauthenticated("unauthorized request"));
    }

    fn build_meta<C>(collection: C, meta: meta::Meta) -> Metadata
    where
        C: Into<String>,
    {
        let mut tags = vec![];
        let mut acl = None;
        for tag in meta.tags {
            if tag.key == ":acl" {
                acl = Some(AclRef {
                    acl: tag.value.parse().unwrap_or(0),
                });
            }
            tags.push(Tag {
                key: tag.key,
                value: tag.value,
            })
        }

        Metadata {
            acl: acl,
            tags: tags,
            collection: collection.into(),
        }
    }
}

#[tonic::async_trait]
impl<S, M> BcdbServiceTrait for BcdbService<S, M>
where
    S: ObjectStorage + Send + Sync + 'static,
    M: MetaStorageFactory,
{
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let auth = request.metadata();

        if !auth.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let request = request.into_inner();

        let data = request.data;
        let metadata = match request.metadata {
            Some(metadata) => metadata,
            None => return Err(Status::invalid_argument("metadata is required")),
        };

        //build metadata for storage
        let mut m = meta::Meta { tags: vec![] };
        for tag in metadata.tags {
            let t = meta::Tag {
                key: tag.key,
                value: tag.value,
            };
            if t.is_reserved() {
                return Err(Status::invalid_argument(format!(
                    "not allowed use of reserved tags: {}",
                    t.key
                )));
            }
            m.tags.push(t);
        }

        //adding default tags to object.
        // note: acl is optional
        match metadata.acl {
            Some(acl) => m.tags.push(meta::Tag::new(":acl", &format!("{}", acl.acl))),
            None => {}
        };

        m.tags
            .push(meta::Tag::new(":size", &format!("{}", data.len())));
        m.tags.push(meta::Tag::new(
            ":created",
            &format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        ));

        // TODO: create from impl for Tonic status for StorageError
        let mut db = self.db.clone();
        let handle = spawn_blocking(move || db.set(None, &data).expect("failed to set data"));
        // TODO: proper error
        let id = handle.await.expect("failed to run blocking task");

        let mut collection = match self.get_collection(&metadata.collection).await {
            Ok(store) => store,
            Err(err) => return Err(err.status()),
        };
        match collection.set(id, m).await {
            Ok(_) => Ok(Response::new(SetResponse { id })),
            Err(err) => Err(err.status()),
        }
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let auth = request.metadata().clone();
        let request = request.into_inner();
        let id = request.id;

        let mut collection = match self.get_collection(&request.collection).await {
            Ok(store) => store,
            Err(err) => return Err(err.status()),
        };

        let info = match collection.get(id).await {
            Ok(info) => info,
            Err(err) => return Err(err.status()),
        };

        let metadata = Self::build_meta(request.collection, info);

        if !auth.is_owner() {
            match metadata.acl {
                Some(ref acl) => {
                    self.is_authorized(acl.acl, auth.get_user().unwrap(), "r--".parse().unwrap())?;
                }

                None => (),
            };
        }

        let mut db = self.db.clone();
        let handle = spawn_blocking(move || db.get(id).expect("failed to load data"));
        // TODO: proper error handling
        let data = handle.await.expect("failed thread pool task");
        if data.is_none() {
            return Err(Status::not_found(format!("Key {} doesn't exist", id)));
        }
        Ok(Response::new(GetResponse {
            data: data.unwrap(), // This unwrap is safe as we checked the none case above
            metadata: Some(metadata),
        }))
    }

    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let auth = request.metadata();

        if !auth.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let request = request.into_inner();

        let metadata = match request.metadata {
            Some(metadata) => metadata,
            None => return Err(Status::invalid_argument("metadata is required")),
        };

        //build metadata for storage
        let mut m = meta::Meta { tags: vec![] };
        for tag in metadata.tags {
            let t = meta::Tag {
                key: tag.key,
                value: tag.value,
            };
            if t.is_reserved() {
                return Err(Status::invalid_argument(format!(
                    "not allowed use of reserved tags: {}",
                    t.key
                )));
            }
            m.tags.push(t);
        }

        //adding default tags to object.
        // note: acl is optional
        match metadata.acl {
            Some(acl) => m.tags.push(meta::Tag::new(":acl", &format!("{}", acl.acl))),
            None => {}
        };

        m.tags.push(meta::Tag::new(
            ":updated",
            &format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        ));

        let mut collection = match self.get_collection(&metadata.collection).await {
            Ok(store) => store,
            Err(err) => return Err(err.status()),
        };

        match collection.set(request.id, m).await {
            Ok(_) => Ok(Response::new(UpdateResponse {})),
            Err(err) => Err(err.status()),
        }
    }

    type ListStream = mpsc::Receiver<Result<ListResponse, Status>>;

    async fn list(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::ListStream>, Status> {
        let auth = request.metadata();

        if !auth.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let request = request.into_inner();
        let mut collection = match self.get_collection(&request.collection).await {
            Ok(store) => store,
            Err(err) => return Err(err.status()),
        };

        let mut tags = vec![];
        for tag in request.tags {
            tags.push(meta::Tag {
                key: tag.key,
                value: tag.value,
            })
        }

        let (mut tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            let mut rx = match collection.find(tags).await {
                Ok(rx) => rx,
                Err(err) => {
                    tx.send(Err(Status::internal(format!("{}", err))))
                        .await
                        .unwrap();
                    return;
                }
            };

            while let Some(id) = rx.recv().await {
                match id {
                    Ok(id) => tx.send(Ok(ListResponse { id: id })).await.unwrap(),
                    Err(err) => tx
                        .send(Err(Status::internal(format!("{}", err))))
                        .await
                        .unwrap(),
                }
            }
        });

        Ok(Response::new(rx))
    }

    type FindStream = mpsc::Receiver<Result<FindResponse, Status>>;

    async fn find(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::FindStream>, Status> {
        let auth = request.metadata();

        if !auth.is_owner() {
            return Err(Status::unauthenticated("not authorized"));
        }

        let request = request.into_inner();
        let collection_name = request.collection;
        let mut collection = match self.get_collection(&collection_name).await {
            Ok(store) => store,
            Err(err) => return Err(err.status()),
        };

        let mut tags = vec![];
        for tag in request.tags {
            tags.push(meta::Tag {
                key: tag.key,
                value: tag.value,
            })
        }

        let (mut tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            let mut getter = collection.clone();
            let mut rx = match collection.find(tags).await {
                Ok(rx) => rx,
                Err(err) => {
                    tx.send(Err(Status::internal(format!("{}", err))))
                        .await
                        .unwrap();
                    return;
                }
            };

            while let Some(id) = rx.recv().await {
                let id = match id {
                    Ok(id) => id,
                    Err(err) => {
                        tx.send(Err(err.status())).await.unwrap();
                        return;
                    }
                };

                let meta = match getter.get(id).await {
                    Ok(meta) => meta,
                    Err(err) => {
                        tx.send(Err(err.status())).await.unwrap();
                        return;
                    }
                };

                let metadata = Self::build_meta(&collection_name, meta);

                tx.send(Ok(FindResponse {
                    id: id,
                    metadata: Some(metadata),
                }))
                .await
                .unwrap();
            }
        });

        Ok(Response::new(rx))
    }
}

use crate::acl::*;

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
