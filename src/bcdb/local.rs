use super::generated::bcdb_server::Bcdb as BcdbServiceTrait;
use super::generated::*;
use super::FailureExt;
use crate::acl::*;
use failure::Error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use tonic::{Request, Response, Status};

use crate::auth::MetadataMapAuth;
use crate::meta::{self, Storage as MetaStorage, StorageFactory as MetaStorageFactory};
use crate::storage::Storage as ObjectStorage;

//TODO: use generics for both object store type and meta factory type.
pub struct LocalBcdb<S, M>
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

impl<S, M> LocalBcdb<S, M>
where
    S: ObjectStorage,
    M: MetaStorageFactory,
{
    pub fn new(db: S, factory: M, acl: ACLStorage<S>) -> LocalBcdb<S, M> {
        LocalBcdb {
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

    fn build_pb_meta<C>(collection: C, meta: meta::Meta) -> Metadata
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

    fn build_meta(&self, metadata: &Metadata) -> Result<meta::Meta, Status> {
        //build metadata for storage
        let mut m = meta::Meta { tags: vec![] };
        for tag in metadata.tags.iter() {
            let t = meta::Tag {
                key: tag.key.clone(),
                value: tag.value.clone(),
            };
            if t.is_reserved() {
                return Err(Status::invalid_argument(format!(
                    "not allowed use of reserved tags: {}",
                    t.key
                )));
            }
            m.tags.push(t);
        }

        Ok(m)
    }

    async fn get_metadata(&self, collection: &str, id: u32) -> Result<Metadata, Status> {
        let mut col = match self.get_collection(collection).await {
            Ok(store) => store,
            Err(err) => return Err(err.status()),
        };

        let info = match col.get(id).await {
            Ok(info) => info,
            Err(err) => return Err(err.status()),
        };

        Ok(Self::build_pb_meta(collection, info))
    }
}

#[tonic::async_trait]
impl<S, M> BcdbServiceTrait for LocalBcdb<S, M>
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

        let mut m = self.build_meta(&metadata)?;

        match metadata.acl {
            Some(ref acl) => m.add(":acl", &format!("{}", acl.acl)),
            None => {}
        };

        m.add(":size", &format!("{}", data.len()));
        m.add(
            ":created",
            &format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        );

        let mut db = self.db.clone();
        let handle = spawn_blocking(move || db.set(None, &data).expect("failed to set data"));
        let id = match handle.await {
            Ok(id) => id,
            Err(err) => {
                return Err(Status::internal(format!(
                    "failed to run blocking task: {}",
                    err
                )));
            }
        };

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

        let metadata = self.get_metadata(&request.collection, id).await?;

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
    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let auth = request.metadata().clone();

        let request = request.into_inner();
        let id = request.id;

        let metadata = self.get_metadata(&request.collection, id).await?;

        if !auth.is_owner() {
            match metadata.acl {
                Some(ref acl) => {
                    self.is_authorized(acl.acl, auth.get_user().unwrap(), "-w-".parse().unwrap())?;
                }

                None => (),
            };
        }

        let mut m = meta::Meta::default();

        m.add(":deleted", "1");

        let mut collection = match self.get_collection(&request.collection).await {
            Ok(store) => store,
            Err(err) => return Err(err.status()),
        };

        match collection.set(request.id, m).await {
            Ok(_) => Ok(Response::new(DeleteResponse {})),
            Err(err) => Err(err.status()),
        }
    }

    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        let auth = request.metadata().clone();

        let request = request.into_inner();
        let id = request.id;
        let data = request.data;

        let new_metadata = match request.metadata {
            Some(metadata) => metadata,
            None => return Err(Status::invalid_argument("metadata is required")),
        };

        let metadata = self.get_metadata(&new_metadata.collection, id).await?;

        if !auth.is_owner() {
            match metadata.acl {
                Some(ref acl) => {
                    self.is_authorized(acl.acl, auth.get_user().unwrap(), "-w-".parse().unwrap())?;
                }

                None => (),
            };
        }

        let mut m = self.build_meta(&new_metadata)?;

        match metadata.acl {
            Some(ref acl) => {
                if auth.is_owner() {
                    m.add(":acl", &format!("{}", acl.acl));
                } else {
                    //trying to update acl while u are not the owner
                    return Err(Status::unauthenticated("only owner can update acl"));
                }
            }
            None => {}
        };

        m.add(
            ":updated",
            &format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        );

        // updating data is optional in an update call
        if let Some(data) = data {
            m.add(":size", &format!("{}", data.data.len()));

            let mut db = self.db.clone();
            let handle =
                spawn_blocking(move || db.set(Some(id), &data.data).expect("failed to set data"));
            if let Err(err) = handle.await {
                return Err(Status::internal(format!(
                    "failed to run blocking task: {}",
                    err
                )));
            }
        }

        let mut collection = match self.get_collection(&new_metadata.collection).await {
            Ok(store) => store,
            Err(err) => return Err(err.status()),
        };

        match collection.set(request.id, m).await {
            Ok(_) => Ok(Response::new(UpdateResponse {})),
            Err(err) => Err(err.status()),
        }
    }

    type ListStream = super::ListStream;

    async fn list(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<super::ListStream>, Status> {
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

    type FindStream = super::FindStream;

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

                let metadata = Self::build_pb_meta(&collection_name, meta);

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
