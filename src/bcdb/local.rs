use super::generated::bcdb_server::Bcdb as BcdbServiceTrait;
use super::generated::*;
use super::FailureExt;
use crate::acl::*;
use anyhow::Error;
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tonic::{Request, Response, Status};

use crate::auth::MetadataMapExt;
use crate::database::{self, Index};
use crate::storage::Storage as ObjectStorage;

const TAG_COLLECTION: &str = ":collection";
const TAG_ACL: &str = ":acl";
const TAG_SIZE: &str = ":size";
const TAG_CREATED: &str = ":created";
const TAG_UPDATED: &str = ":updated";
const TAG_DELETED: &str = ":deleted";

//TODO: use generics for both object store type and meta factory type.
pub struct LocalBcdb<S, M>
where
    S: ObjectStorage,
    M: Index,
{
    db: S,
    meta: M,
    //NOTE: acl storage doesn't have to use the same underlying storage type S
    acl: ACLStorage<S>,
}

impl<S, M> LocalBcdb<S, M>
where
    S: ObjectStorage,
    M: Index + Clone,
{
    pub fn new(db: S, meta: M, acl: ACLStorage<S>) -> LocalBcdb<S, M> {
        LocalBcdb {
            db: db,
            meta: meta,
            acl: acl,
        }
    }

    fn get_permissions(&self, acl: u64, user: u64) -> Result<Permissions, Error> {
        // self.acl.g
        let mut store = self.acl.clone();
        let acl = match store.get(acl as u32)? {
            Some(acl) => acl,
            None => return Ok(Permissions::default()),
        };

        debug!("checking user {} against acl: {:?}", user, acl);

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

        return Err(Status::unauthenticated("not authorized"));
    }

    fn build_pb_meta<C>(collection: C, meta: database::Meta) -> Metadata
    where
        C: Into<String>,
    {
        let mut tags = vec![];
        let mut acl = None;
        for (k, v) in meta {
            if k == TAG_ACL {
                acl = Some(AclRef {
                    acl: v.parse().unwrap_or(0),
                });
            }
            tags.push(Tag { key: k, value: v })
        }

        Metadata {
            acl: acl,
            tags: tags,
            collection: collection.into(),
        }
    }

    fn build_meta(&self, metadata: &Metadata) -> Result<database::Meta, Status> {
        //build metadata for storage
        let mut m = database::Meta::default();
        for tag in metadata.tags.iter() {
            if database::is_reserved(&tag.key) {
                return Err(Status::invalid_argument(format!(
                    "not allowed use of reserved tags: {}",
                    tag.key
                )));
            }

            m.insert(tag.key.clone(), tag.value.clone());
        }

        Ok(m)
    }

    async fn get_metadata(&self, collection: Option<&str>, id: u32) -> Result<Metadata, Status> {
        let mut meta = self.meta.clone();
        let info = match meta.get(id).await {
            Ok(info) => info,
            Err(err) => return Err(err.status()),
        };

        let col = info.get(TAG_COLLECTION);
        if let Some(collection) = collection {
            match col {
                Some(v) if v == collection => {}
                _ => return Err(Status::not_found("object not found")),
            };
        }

        let col: String = match col {
            Some(v) => v.into(),
            None => ":unknown".into(),
        };

        Ok(Self::build_pb_meta(col, info))
    }
}

#[tonic::async_trait]
impl<S, M> BcdbServiceTrait for LocalBcdb<S, M>
where
    S: ObjectStorage + Send + Sync + 'static,
    M: Index + Clone,
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

        if let Some(acl) = metadata.acl {
            m.insert(TAG_ACL, format!("{}", acl.acl));
        }

        m.insert(TAG_COLLECTION, metadata.collection);
        m.insert(TAG_SIZE, format!("{}", data.len()));
        m.insert(
            TAG_CREATED,
            format!(
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
        let mut meta = self.meta.clone();
        match meta.set(id, m).await {
            Ok(_) => Ok(Response::new(SetResponse { id })),
            Err(err) => Err(err.status()),
        }
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let auth = request.metadata().clone();
        let request = request.into_inner();
        let id = request.id;

        let metadata = self.get_metadata(Some(&request.collection), id).await?;

        if !auth.is_owner() {
            match metadata.acl {
                Some(ref acl) => {
                    self.is_authorized(acl.acl, auth.get_user().unwrap(), "r--".parse().unwrap())?;
                }

                None => return Err(Status::unauthenticated("not authorized")),
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

    async fn fetch(&self, request: Request<FetchRequest>) -> Result<Response<GetResponse>, Status> {
        let auth = request.metadata().clone();
        let request = request.into_inner();
        let id = request.id;

        let metadata = self.get_metadata(None, id).await?;

        if !auth.is_owner() {
            match metadata.acl {
                Some(ref acl) => {
                    self.is_authorized(acl.acl, auth.get_user().unwrap(), "r--".parse().unwrap())?;
                }

                None => return Err(Status::unauthenticated("not authorized")),
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

        let metadata = self.get_metadata(Some(&request.collection), id).await?;

        if !auth.is_owner() {
            match metadata.acl {
                Some(ref acl) => {
                    self.is_authorized(acl.acl, auth.get_user().unwrap(), "-w-".parse().unwrap())?;
                }

                None => return Err(Status::unauthenticated("not authorized")),
            };
        }

        let mut m = database::Meta::default();

        m.insert(TAG_DELETED, "1");
        let mut meta = self.meta.clone();
        match meta.set(request.id, m).await {
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

        let metadata = self
            .get_metadata(Some(&new_metadata.collection), id)
            .await?;

        if !auth.is_owner() {
            match metadata.acl {
                Some(ref acl) => {
                    self.is_authorized(acl.acl, auth.get_user().unwrap(), "-w-".parse().unwrap())?;
                }

                None => return Err(Status::unauthenticated("not authorized")),
            };
        }

        let mut m = self.build_meta(&new_metadata)?;

        match new_metadata.acl {
            Some(ref acl) => {
                if auth.is_owner() {
                    m.insert(TAG_ACL, format!("{}", acl.acl));
                } else {
                    //trying to update acl while u are not the owner
                    return Err(Status::unauthenticated("only owner can update acl"));
                }
            }
            None => {}
        };

        m.insert(
            TAG_UPDATED,
            format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        );

        // updating data is optional in an update call
        if let Some(data) = data {
            m.insert(TAG_SIZE, format!("{}", data.data.len()));

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

        let mut meta = self.meta.clone();
        match meta.set(request.id, m).await {
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
        let mut meta = self.meta.clone();

        let mut tags = database::Meta::default();
        for tag in request.tags {
            tags.insert(tag.key, tag.value);
        }

        tags.insert(TAG_COLLECTION, request.collection);

        let (mut tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            let mut rx = match meta.find(tags).await {
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
        let mut meta = self.meta.clone();

        let mut tags = database::Meta::default();
        for tag in request.tags {
            tags.insert(tag.key, tag.value);
        }

        tags.insert(TAG_COLLECTION, collection_name.clone());

        let (mut tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            let mut rx = match meta.find(tags).await {
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

                let meta = match meta.get(id).await {
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
