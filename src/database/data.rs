use crate::acl::*;
use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;

use super::*;
use crate::storage::Storage;

const TAG_COLLECTION: &str = ":collection";
const TAG_ACL: &str = ":acl";
const TAG_SIZE: &str = ":size";
const TAG_CREATED: &str = ":created";
const TAG_UPDATED: &str = ":updated";
const TAG_DELETED: &str = ":deleted";

//TODO: use generics for both object store type and meta factory type.
pub struct BcdbDatabase<S, I>
where
    S: Storage,
    I: Index,
{
    data: S,
    meta: I,
    acl: ACLStorage<S>,
}

impl<S, I> BcdbDatabase<S, I>
where
    S: Storage,
    I: Index + Clone,
{
    pub fn new(data: S, meta: I, acl: ACLStorage<S>) -> Self {
        BcdbDatabase {
            data: data,
            meta: meta,
            acl: acl,
        }
    }

    // fn get_permissions(&self, acl: u64, user: u64) -> Result<Permissions, Error> {
    //     // self.acl.g
    //     let mut store = self.acl.clone();
    //     let acl = match store.get(acl as u32)? {
    //         Some(acl) => acl,
    //         None => return Ok(Permissions::default()),
    //     };

    //     debug!("checking user {} against acl: {:?}", user, acl);

    //     if acl.users.contains(&user) {
    //         return Ok(acl.perm);
    //     }

    //     Ok(Permissions::default())
    // }

    // fn is_authorized(&self, acl: u64, user: u64, perm: Permissions) -> Result<(), Status> {
    //     let stored = match self.get_permissions(acl, user) {
    //         Ok(stored) => stored,
    //         Err(err) => {
    //             return Err(Status::unauthenticated(format!(
    //                 "failed to get assigned permissions: {}",
    //                 err
    //             )))
    //         }
    //     };
    //     if stored.grants(perm) {
    //         return Ok(());
    //     }

    //     return Err(Status::unauthenticated("not authorized"));
    // }

    // // fn build_pb_meta<C>(collection: C, meta: Meta) -> Metadata
    // // where
    // //     C: Into<String>,
    // // {
    // //     let mut tags = vec![];
    // //     let mut acl = None;
    // //     for tag in meta.tags {
    // //         if tag.key == TAG_ACL {
    // //             acl = Some(AclRef {
    // //                 acl: tag.value.parse().unwrap_or(0),
    // //             });
    // //         }
    // //         tags.push(Tag {
    // //             key: tag.key,
    // //             value: tag.value,
    // //         })
    // //     }

    // //     Metadata {
    // //         acl: acl,
    // //         tags: tags,
    // //         collection: collection.into(),
    // //     }
    // // }

    // fn build_meta(&self, metadata: &Metadata) -> Result<database::Meta, Status> {
    //     //build metadata for storage
    //     let mut m = database::Meta { tags: vec![] };
    //     for tag in metadata.tags.iter() {
    //         let t = database::Tag {
    //             key: tag.key.clone(),
    //             value: tag.value.clone(),
    //         };
    //         if t.is_reserved() {
    //             return Err(Status::invalid_argument(format!(
    //                 "not allowed use of reserved tags: {}",
    //                 t.key
    //             )));
    //         }
    //         m.tags.push(t);
    //     }

    //     Ok(m)
    // }

    // async fn get_metadata(&self, collection: Option<&str>, id: u32) -> Result<Metadata, Status> {
    //     let mut meta = self.meta.clone();
    //     let info = match meta.get(id).await {
    //         Ok(info) => info,
    //         Err(err) => return Err(err.status()),
    //     };

    //     let col = info.find(TAG_COLLECTION);
    //     if let Some(collection) = collection {
    //         match col {
    //             Some(ref v) if v == collection => {}
    //             _ => return Err(Status::not_found("object not found")),
    //         };
    //     }

    //     Ok(Self::build_pb_meta(
    //         col.unwrap_or_else(|| ":unknown".into()),
    //         info,
    //     ))
    // }
}

#[tonic::async_trait]
impl<S, I> Database for BcdbDatabase<S, I>
where
    S: Storage + Send + Sync + 'static,
    I: Index + Clone,
{
    async fn set(
        &mut self,
        ctx: Context,
        collection: String,
        data: Vec<u8>,
        meta: Meta,
        acl: Option<u64>,
    ) -> Result<Key> {
        if !ctx.is_owner() {
            bail!("unauthorized");
        }

        let mut meta = meta;
        if let Some(acl) = acl {
            meta.insert(TAG_ACL.into(), format!("{}", acl));
        }

        meta.insert(TAG_COLLECTION.into(), collection);
        meta.insert(TAG_SIZE.into(), format!("{}", data.len()));
        meta.insert(
            TAG_CREATED.into(),
            format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        );

        let mut db = self.data.clone();
        let handle = spawn_blocking(move || db.set(None, &data).expect("failed to set data"));
        let id = match handle.await {
            Ok(id) => id,
            Err(err) => {
                bail!("failed to run blocking task: {}", err);
            }
        };

        let mut index = self.meta.clone();
        index.set(id, meta).await.map(|_| id)
    }

    async fn get(&mut self, ctx: Context, key: Key) -> Result<Object> {
        bail!("not implemented")
    }
    async fn find(
        &mut self,
        ctx: Context,
        meta: Meta,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Key>>> {
        bail!("not implemented")
    }

    // async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
    //     let auth = request.metadata();

    //     if !auth.is_owner() {
    //         return Err(Status::unauthenticated("not authorized"));
    //     }

    //     let request = request.into_inner();

    //     let data = request.data;
    //     let metadata = match request.metadata {
    //         Some(metadata) => metadata,
    //         None => return Err(Status::invalid_argument("metadata is required")),
    //     };

    //     let mut m = self.build_meta(&metadata)?;

    //     match metadata.acl {
    //         Some(ref acl) => m.add(TAG_ACL, &format!("{}", acl.acl)),
    //         None => {}
    //     };

    //     m.add(TAG_COLLECTION, metadata.collection);
    //     m.add(TAG_SIZE, &format!("{}", data.len()));
    //     m.add(
    //         TAG_CREATED,
    //         &format!(
    //             "{}",
    //             std::time::SystemTime::now()
    //                 .duration_since(std::time::UNIX_EPOCH)
    //                 .unwrap()
    //                 .as_secs()
    //         ),
    //     );

    //     let mut db = self.db.clone();
    //     let handle = spawn_blocking(move || db.set(None, &data).expect("failed to set data"));
    //     let id = match handle.await {
    //         Ok(id) => id,
    //         Err(err) => {
    //             return Err(Status::internal(format!(
    //                 "failed to run blocking task: {}",
    //                 err
    //             )));
    //         }
    //     };
    //     let mut meta = self.meta.clone();
    //     match meta.set(id, m).await {
    //         Ok(_) => Ok(Response::new(SetResponse { id })),
    //         Err(err) => Err(err.status()),
    //     }
    // }

    // async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
    //     let auth = request.metadata().clone();
    //     let request = request.into_inner();
    //     let id = request.id;

    //     let metadata = self.get_metadata(Some(&request.collection), id).await?;

    //     if !auth.is_owner() {
    //         match metadata.acl {
    //             Some(ref acl) => {
    //                 self.is_authorized(acl.acl, auth.get_user().unwrap(), "r--".parse().unwrap())?;
    //             }

    //             None => return Err(Status::unauthenticated("not authorized")),
    //         };
    //     }

    //     let mut db = self.db.clone();
    //     let handle = spawn_blocking(move || db.get(id).expect("failed to load data"));
    //     // TODO: proper error handling
    //     let data = handle.await.expect("failed thread pool task");
    //     if data.is_none() {
    //         return Err(Status::not_found(format!("Key {} doesn't exist", id)));
    //     }
    //     Ok(Response::new(GetResponse {
    //         data: data.unwrap(), // This unwrap is safe as we checked the none case above
    //         metadata: Some(metadata),
    //     }))
    // }

    // async fn fetch(&self, request: Request<FetchRequest>) -> Result<Response<GetResponse>, Status> {
    //     let auth = request.metadata().clone();
    //     let request = request.into_inner();
    //     let id = request.id;

    //     let metadata = self.get_metadata(None, id).await?;

    //     if !auth.is_owner() {
    //         match metadata.acl {
    //             Some(ref acl) => {
    //                 self.is_authorized(acl.acl, auth.get_user().unwrap(), "r--".parse().unwrap())?;
    //             }

    //             None => return Err(Status::unauthenticated("not authorized")),
    //         };
    //     }

    //     let mut db = self.db.clone();
    //     let handle = spawn_blocking(move || db.get(id).expect("failed to load data"));
    //     // TODO: proper error handling
    //     let data = handle.await.expect("failed thread pool task");
    //     if data.is_none() {
    //         return Err(Status::not_found(format!("Key {} doesn't exist", id)));
    //     }
    //     Ok(Response::new(GetResponse {
    //         data: data.unwrap(), // This unwrap is safe as we checked the none case above
    //         metadata: Some(metadata),
    //     }))
    // }

    // async fn delete(
    //     &self,
    //     request: Request<DeleteRequest>,
    // ) -> Result<Response<DeleteResponse>, Status> {
    //     let auth = request.metadata().clone();

    //     let request = request.into_inner();
    //     let id = request.id;

    //     let metadata = self.get_metadata(Some(&request.collection), id).await?;

    //     if !auth.is_owner() {
    //         match metadata.acl {
    //             Some(ref acl) => {
    //                 self.is_authorized(acl.acl, auth.get_user().unwrap(), "-w-".parse().unwrap())?;
    //             }

    //             None => return Err(Status::unauthenticated("not authorized")),
    //         };
    //     }

    //     let mut m = database::Meta::default();

    //     m.add(TAG_DELETED, "1");
    //     let mut meta = self.meta.clone();
    //     match meta.set(request.id, m).await {
    //         Ok(_) => Ok(Response::new(DeleteResponse {})),
    //         Err(err) => Err(err.status()),
    //     }
    // }

    // async fn update(
    //     &self,
    //     request: Request<UpdateRequest>,
    // ) -> Result<Response<UpdateResponse>, Status> {
    //     let auth = request.metadata().clone();

    //     let request = request.into_inner();
    //     let id = request.id;
    //     let data = request.data;

    //     let new_metadata = match request.metadata {
    //         Some(metadata) => metadata,
    //         None => return Err(Status::invalid_argument("metadata is required")),
    //     };

    //     let metadata = self
    //         .get_metadata(Some(&new_metadata.collection), id)
    //         .await?;

    //     if !auth.is_owner() {
    //         match metadata.acl {
    //             Some(ref acl) => {
    //                 self.is_authorized(acl.acl, auth.get_user().unwrap(), "-w-".parse().unwrap())?;
    //             }

    //             None => return Err(Status::unauthenticated("not authorized")),
    //         };
    //     }

    //     let mut m = self.build_meta(&new_metadata)?;

    //     match new_metadata.acl {
    //         Some(ref acl) => {
    //             if auth.is_owner() {
    //                 m.add(TAG_ACL, &format!("{}", acl.acl));
    //             } else {
    //                 //trying to update acl while u are not the owner
    //                 return Err(Status::unauthenticated("only owner can update acl"));
    //             }
    //         }
    //         None => {}
    //     };

    //     m.add(
    //         TAG_UPDATED,
    //         &format!(
    //             "{}",
    //             std::time::SystemTime::now()
    //                 .duration_since(std::time::UNIX_EPOCH)
    //                 .unwrap()
    //                 .as_secs()
    //         ),
    //     );

    //     // updating data is optional in an update call
    //     if let Some(data) = data {
    //         m.add(":size", &format!("{}", data.data.len()));

    //         let mut db = self.db.clone();
    //         let handle =
    //             spawn_blocking(move || db.set(Some(id), &data.data).expect("failed to set data"));
    //         if let Err(err) = handle.await {
    //             return Err(Status::internal(format!(
    //                 "failed to run blocking task: {}",
    //                 err
    //             )));
    //         }
    //     }

    //     let mut meta = self.meta.clone();
    //     match meta.set(request.id, m).await {
    //         Ok(_) => Ok(Response::new(UpdateResponse {})),
    //         Err(err) => Err(err.status()),
    //     }
    // }

    // type ListStream = super::ListStream;

    // async fn list(
    //     &self,
    //     request: Request<QueryRequest>,
    // ) -> Result<Response<super::ListStream>, Status> {
    //     let auth = request.metadata();

    //     if !auth.is_owner() {
    //         return Err(Status::unauthenticated("not authorized"));
    //     }

    //     let request = request.into_inner();
    //     let mut meta = self.meta.clone();

    //     let mut tags = vec![];
    //     for tag in request.tags {
    //         tags.push(database::Tag {
    //             key: tag.key,
    //             value: tag.value,
    //         })
    //     }

    //     tags.push(database::Tag {
    //         key: TAG_COLLECTION.into(),
    //         value: request.collection,
    //     });

    //     let (mut tx, rx) = mpsc::channel(10);
    //     tokio::spawn(async move {
    //         let mut rx = match meta.find(tags).await {
    //             Ok(rx) => rx,
    //             Err(err) => {
    //                 tx.send(Err(Status::internal(format!("{}", err))))
    //                     .await
    //                     .unwrap();
    //                 return;
    //             }
    //         };

    //         while let Some(id) = rx.recv().await {
    //             match id {
    //                 Ok(id) => tx.send(Ok(ListResponse { id: id })).await.unwrap(),
    //                 Err(err) => tx
    //                     .send(Err(Status::internal(format!("{}", err))))
    //                     .await
    //                     .unwrap(),
    //             }
    //         }
    //     });

    //     Ok(Response::new(rx))
    // }

    // type FindStream = super::FindStream;

    // async fn find(
    //     &self,
    //     request: Request<QueryRequest>,
    // ) -> Result<Response<Self::FindStream>, Status> {
    //     let auth = request.metadata();

    //     if !auth.is_owner() {
    //         return Err(Status::unauthenticated("not authorized"));
    //     }

    //     let request = request.into_inner();
    //     let collection_name = request.collection;
    //     let mut meta = self.meta.clone();
    //     let mut tags = vec![];
    //     for tag in request.tags {
    //         tags.push(database::Tag {
    //             key: tag.key,
    //             value: tag.value,
    //         })
    //     }

    //     tags.push(database::Tag {
    //         key: TAG_COLLECTION.into(),
    //         value: collection_name.clone(),
    //     });

    //     let (mut tx, rx) = mpsc::channel(10);
    //     tokio::spawn(async move {
    //         let mut rx = match meta.find(tags).await {
    //             Ok(rx) => rx,
    //             Err(err) => {
    //                 tx.send(Err(Status::internal(format!("{}", err))))
    //                     .await
    //                     .unwrap();
    //                 return;
    //             }
    //         };

    //         while let Some(id) = rx.recv().await {
    //             let id = match id {
    //                 Ok(id) => id,
    //                 Err(err) => {
    //                     tx.send(Err(err.status())).await.unwrap();
    //                     return;
    //                 }
    //             };

    //             let meta = match meta.get(id).await {
    //                 Ok(meta) => meta,
    //                 Err(err) => {
    //                     tx.send(Err(err.status())).await.unwrap();
    //                     return;
    //                 }
    //             };

    //             let metadata = Self::build_pb_meta(&collection_name, meta);

    //             tx.send(Ok(FindResponse {
    //                 id: id,
    //                 metadata: Some(metadata),
    //             }))
    //             .await
    //             .unwrap();
    //         }
    //     });

    //     Ok(Response::new(rx))
    // }
}
