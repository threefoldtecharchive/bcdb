use failure::Error;
use generated::acl_server::Acl as AclServiceTrait;
use generated::bcdb_server::Bcdb as BcdbServiceTrait;
use generated::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tonic::{Code, Request, Response, Status};

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
pub struct BcdbService<M>
where
    M: MetaStorageFactory,
{
    db: Zdb,
    factory: M,
    stores: Arc<Mutex<HashMap<String, M::Storage>>>,
}

impl Default for BcdbService<SqliteMetaStoreFactory> {
    fn default() -> Self {
        BcdbService {
            db: Zdb::default(),
            factory: SqliteMetaStoreFactory::new("/tmp/meta")
                .expect("failed to create default meta store factory"),
            stores: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<M> BcdbService<M>
where
    M: MetaStorageFactory,
{
    async fn get_store(&self, collection: &str) -> Result<M::Storage, Error> {
        {
            let stores = self.stores.lock().unwrap();
            if let Some(store) = stores.get(collection) {
                return Ok(store.clone());
            }
        }

        let store = self.factory.new(collection).await?;
        let mut stores = self.stores.lock().unwrap();
        stores.insert(collection.into(), store.clone());

        Ok(store)
    }
}

#[tonic::async_trait]
impl<M> BcdbServiceTrait for BcdbService<M>
where
    M: MetaStorageFactory,
{
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        debug!("received a set call");

        let request = request.into_inner();

        let data = request.data;
        let metadata = match request.metadata {
            Some(metadata) => metadata,
            None => return Err(Status::invalid_argument("metadata is required")),
        };

        //build metadata for storage
        let mut m = meta::Meta { tags: vec![] };
        for tag in metadata.tags {
            m.tags.push(meta::Tag {
                key: tag.key,
                value: tag.value,
            });
        }

        m.tags
            .push(meta::Tag::new("_size", &format!("{}", data.len())));
        m.tags.push(meta::Tag::new(
            "_created",
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

        debug!("data stored in zdb with id: {}", id);
        let mut metastore = match self.get_store(&metadata.collection).await {
            Ok(store) => store,
            Err(err) => return Err(err.status()),
        };
        debug!("storing data in metadata");
        match metastore.set(id, m).await {
            Ok(_) => Ok(Response::new(SetResponse { id })),
            Err(err) => Err(err.status()),
        }
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let id = request.into_inner().id;

        // TODO: from impl for error
        let mut db = self.db.clone();
        let handle = spawn_blocking(move || db.get(id).expect("failed to load data"));
        // TODO: proper error handling
        let data = handle.await.expect("failed threadpool task");
        if data.is_none() {
            return Err(Status::not_found(format!("Key {} doesn't exist", id)));
        }
        Ok(Response::new(GetResponse {
            data: data.unwrap(), // This unwrap is safe as we checked the none case above
            metadata: None,
        }))
    }

    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        Err(Status::unimplemented("not implemented yet!"))
    }

    type ListStream = mpsc::Receiver<Result<ListResponse, Status>>;

    async fn list(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::ListStream>, Status> {
        let (mut tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            for i in 0..3 {
                tx.send(Ok(ListResponse { id: i })).await.unwrap();
            }
        });

        Ok(Response::new(rx))
    }

    type FindStream = mpsc::Receiver<Result<FindResponse, Status>>;

    async fn find(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::FindStream>, Status> {
        Err(Status::unimplemented("not implemented yet!"))
    }
}

use crate::acl::*;

pub struct AclService {
    store: ACLStorage<Collection>,
}

impl Default for AclService {
    fn default() -> AclService {
        AclService {
            store: ACLStorage::new(Zdb::default().collection("acl")),
        }
    }
}

#[tonic::async_trait]
impl AclServiceTrait for AclService {
    async fn get(
        &self,
        request: Request<AclGetRequest>,
    ) -> Result<Response<AclGetResponse>, Status> {
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
        _request: Request<AclListRequest>,
    ) -> Result<Response<Self::ListStream>, Status> {
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
