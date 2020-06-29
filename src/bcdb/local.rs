use super::generated::bcdb_server::Bcdb as BcdbServiceTrait;
use super::generated::*;
use super::FailureExt;
use crate::acl::*;
use crate::auth::MetadataMapExt;
use crate::database::{Database, Meta};
use tokio::stream::StreamExt;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

const TAG_COLLECTION: &str = ":collection";
const TAG_ACL: &str = ":acl";
const TAG_SIZE: &str = ":size";
const TAG_CREATED: &str = ":created";
const TAG_UPDATED: &str = ":updated";
const TAG_DELETED: &str = ":deleted";

//TODO: use generics for both object store type and meta factory type.
pub struct LocalBcdb<D>
where
    D: Database,
{
    db: D,
}

impl<D> LocalBcdb<D>
where
    D: Database + Clone,
{
    pub fn new(db: D) -> Self {
        LocalBcdb { db: db }
    }

    // fn build_pb_meta<C>(collection: C, meta: database::Meta) -> Metadata
    // where
    //     C: Into<String>,
    // {
    //     let mut tags = vec![];
    //     let mut acl = None;
    //     for (k, v) in meta {
    //         if k == TAG_ACL {
    //             acl = Some(AclRef {
    //                 acl: v.parse().unwrap_or(0),
    //             });
    //         }
    //         tags.push(Tag { key: k, value: v })
    //     }

    //     Metadata {
    //         acl: acl,
    //         tags: tags,
    //         collection: collection.into(),
    //     }
    // }

    fn build_meta(metadata: Meta) -> Metadata {
        //build metadata for storage
        let collection = metadata.collection().unwrap_or_default();
        let acl = metadata.acl().map(|acl| AclRef { acl });
        Metadata {
            tags: metadata.inner(),
            collection: collection,
            acl: acl,
        }
    }

    // async fn get_metadata(&self, collection: Option<&str>, id: u32) -> Result<Metadata, Status> {
    //     let mut meta = self.meta.clone();
    //     let info = match meta.get(id).await {
    //         Ok(info) => info,
    //         Err(err) => return Err(err.status()),
    //     };

    //     let col = info.get(TAG_COLLECTION);
    //     if let Some(collection) = collection {
    //         match col {
    //             Some(v) if v == collection => {}
    //             _ => return Err(Status::not_found("object not found")),
    //         };
    //     }

    //     let col: String = match col {
    //         Some(v) => v.into(),
    //         None => ":unknown".into(),
    //     };

    //     Ok(Self::build_pb_meta(col, info))
    // }
}

#[tonic::async_trait]
impl<D> BcdbServiceTrait for LocalBcdb<D>
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
            .set(
                context,
                metadata.collection,
                data,
                Meta::from(metadata.tags),
                acl,
            )
            .await
            .map_err(|e| e.status())?;

        Ok(Response::new(SetResponse { id }))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();
        let id = request.id;

        let mut db = self.db.clone();
        let object = db.get(ctx, id).await.map_err(|e| e.status())?;

        if !object.meta.is_collection(request.collection) {
            return Err(Status::not_found("object not found"));
        }

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
        let object = db.get(ctx, id).await.map_err(|e| e.status())?;

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
        db.delete(ctx, id).await.map_err(|e| e.status())?;

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
        let id = db
            .update(
                ctx,
                id,
                metadata.collection,
                data.map(|d| d.data),
                Meta::from(metadata.tags),
                acl,
            )
            .await
            .map_err(|e| e.status())?;

        Ok(Response::new(UpdateResponse {}))
    }

    type ListStream = super::ListStream;

    async fn list(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<super::ListStream>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();

        let mut db = self.db.clone();

        let mut results = db
            .list(ctx, Meta::from(request.tags), Some(request.collection))
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

    type FindStream = super::FindStream;

    async fn find(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::FindStream>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();

        let mut db = self.db.clone();

        let mut results = db
            .find(ctx, Meta::from(request.tags), Some(request.collection))
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
