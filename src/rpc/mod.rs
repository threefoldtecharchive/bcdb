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

    async fn head(&self, request: Request<GetRequest>) -> Result<Response<HeadResponse>, Status> {
        let ctx = request.metadata().context();
        let request = request.into_inner();
        let id = request.id;

        let mut db = self.db.clone();
        let object = db
            .head(ctx, id, request.collection)
            .await
            .map_err(|e| e.status())?;

        Ok(Response::new(HeadResponse {
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

#[cfg(test)]
mod rpc_tests {
    use super::generated::*;
    use super::BcdbService;
    use crate::database::data::database_tests::get_in_memory_db;
    use crate::database::Database;
    use crate::database::{Authorization, Context};
    use crate::rpc::generated::bcdb_server::Bcdb;
    use std::collections::HashMap;
    use tonic::Request;

    #[tokio::test]
    async fn rpc_set_owner() {
        let mut db = get_in_memory_db();
        let rpc = BcdbService::new(db.clone());
        let mut tags = HashMap::default();
        tags.insert("tag".into(), "value".into());

        let data: Vec<u8> = "hello world".into();
        let mut request = Request::new(SetRequest {
            data: data.clone(),
            metadata: Some(Metadata {
                acl: Some(AclRef { acl: 3 }),
                collection: "test".into(),
                tags: tags,
            }),
        });

        // set required context on request
        Context::default()
            .with_auth(Authorization::Owner)
            .into_metadata(request.metadata_mut());

        let result = rpc.set(request).await;
        assert_eq!(result.is_ok(), true);
        let id = result.unwrap().into_inner().id;

        let object = db
            .fetch(Context::default().with_auth(Authorization::Owner), id)
            .await
            .unwrap();

        assert_eq!(object.data.unwrap(), data);
        assert_eq!(object.meta.is_collection("test"), true);
        assert_eq!(object.meta.acl(), Some(3));
        assert_eq!(object.meta.get("tag").unwrap(), "value");
    }

    #[tokio::test]
    async fn rpc_set_no_owner() {
        let db = get_in_memory_db();
        let rpc = BcdbService::new(db);
        let mut tags = HashMap::default();
        tags.insert("tag".into(), "value".into());

        let data: Vec<u8> = "hello world".into();
        let mut request = Request::new(SetRequest {
            data: data.clone(),
            metadata: Some(Metadata {
                acl: None,
                collection: "test".into(),
                tags: tags,
            }),
        });

        // set required context on request
        Context::default()
            .with_auth(Authorization::User(5))
            .into_metadata(request.metadata_mut());

        let result = rpc.set(request).await;
        assert_eq!(result.is_err(), true);
        let status = result.err().unwrap();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn rpc_get() {
        let mut db = get_in_memory_db();
        let data: Vec<u8> = "hello world".into();
        let mut tags = HashMap::default();
        tags.insert("tag".into(), "value".into());

        let id = db
            .set(
                Context::default().with_auth(Authorization::Owner),
                "test".into(),
                data.clone(),
                tags,
                None,
            )
            .await
            .unwrap();

        let rpc = BcdbService::new(db);

        let mut request = Request::new(GetRequest {
            id: id,
            collection: "test".into(),
        });

        // set required context on request
        Context::default()
            .with_auth(Authorization::Owner)
            .into_metadata(request.metadata_mut());

        let result = rpc.get(request).await;
        assert_eq!(result.is_ok(), true);
        let object = result.unwrap().into_inner();

        assert_eq!(object.data, data);
        let metadata = object.metadata.unwrap();
        assert_eq!(metadata.collection, "test");
        assert_eq!(metadata.acl, None);
        assert_eq!(metadata.tags.get("tag").unwrap(), "value");
    }

    #[tokio::test]
    async fn rpc_fetch() {
        let mut db = get_in_memory_db();
        let data: Vec<u8> = "hello world".into();
        let mut tags = HashMap::default();
        tags.insert("tag".into(), "value".into());

        let id = db
            .set(
                Context::default().with_auth(Authorization::Owner),
                "test".into(),
                data.clone(),
                tags,
                None,
            )
            .await
            .unwrap();

        let rpc = BcdbService::new(db);

        let mut request = Request::new(GetRequest {
            id: id,
            collection: "wrong".into(),
        });

        // set required context on request
        Context::default()
            .with_auth(Authorization::Owner)
            .into_metadata(request.metadata_mut());

        let result = rpc.get(request).await;
        assert_eq!(result.is_err(), true);
        let status = result.err().unwrap();
        assert_eq!(status.code(), tonic::Code::NotFound);

        let mut request = Request::new(FetchRequest { id: id });

        // set required context on request
        Context::default()
            .with_auth(Authorization::Owner)
            .into_metadata(request.metadata_mut());

        let result = rpc.fetch(request).await;

        let object = result.unwrap().into_inner();

        assert_eq!(object.data, data);
        let metadata = object.metadata.unwrap();
        assert_eq!(metadata.collection, "test");
        assert_eq!(metadata.acl, None);
        assert_eq!(metadata.tags.get("tag").unwrap(), "value");
    }

    #[tokio::test]
    async fn rpc_delete() {
        let mut db = get_in_memory_db();
        let data: Vec<u8> = "hello world".into();
        let mut tags = HashMap::default();
        tags.insert("tag".into(), "value".into());

        let id = db
            .set(
                Context::default().with_auth(Authorization::Owner),
                "test".into(),
                data.clone(),
                tags,
                None,
            )
            .await
            .unwrap();

        let rpc = BcdbService::new(db);

        let mut request = Request::new(DeleteRequest {
            id: id,
            collection: "test".into(),
        });

        // set required context on request
        Context::default()
            .with_auth(Authorization::Owner)
            .into_metadata(request.metadata_mut());

        let result = rpc.delete(request).await;
        assert_eq!(result.is_ok(), true);

        let mut request = Request::new(FetchRequest { id: id });

        // set required context on request
        Context::default()
            .with_auth(Authorization::Owner)
            .into_metadata(request.metadata_mut());

        let result = rpc.fetch(request).await;

        assert_eq!(result.is_err(), true);
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn rpc_update() {
        let mut db = get_in_memory_db();
        let data: Vec<u8> = "hello world".into();
        let mut tags = HashMap::default();
        tags.insert("tag".into(), "value".into());

        let id = db
            .set(
                Context::default().with_auth(Authorization::Owner),
                "test".into(),
                data.clone(),
                tags,
                None,
            )
            .await
            .unwrap();

        let rpc = BcdbService::new(db);
        let new_data: Vec<u8> = "hello new world".into();
        //update data and acl
        let mut request = Request::new(UpdateRequest {
            id: id,
            metadata: Some(Metadata {
                collection: "test".into(),
                tags: HashMap::default(),
                acl: Some(AclRef { acl: 3 }),
            }),
            data: Some(update_request::UpdateData {
                data: new_data.clone(),
            }),
        });

        // set required context on request
        Context::default()
            .with_auth(Authorization::Owner)
            .into_metadata(request.metadata_mut());

        let result = rpc.update(request).await;
        assert_eq!(result.is_ok(), true);

        let mut request = Request::new(FetchRequest { id: id });

        // set required context on request
        Context::default()
            .with_auth(Authorization::Owner)
            .into_metadata(request.metadata_mut());

        let result = rpc.fetch(request).await;

        let object = result.unwrap().into_inner();

        assert_eq!(object.data, new_data);
        let metadata = object.metadata.unwrap();
        assert_eq!(metadata.collection, "test");
        assert_eq!(metadata.acl, Some(AclRef { acl: 3 }));
        assert_eq!(metadata.tags.get("tag").unwrap(), "value");
    }

    #[tokio::test]
    async fn rpc_find() {
        let mut db = get_in_memory_db();
        let data: Vec<u8> = "hello world".into();
        let mut tags = HashMap::default();
        tags.insert("common".into(), "value".into());
        tags.insert("name".into(), "object-1".into());

        let id_1 = db
            .set(
                Context::default().with_auth(Authorization::Owner),
                "test".into(),
                data.clone(),
                tags,
                Some(3),
            )
            .await
            .unwrap();

        let mut tags = HashMap::default();
        tags.insert("common".into(), "value".into());
        tags.insert("name".into(), "object-2".into());

        let id_2 = db
            .set(
                Context::default().with_auth(Authorization::Owner),
                "test".into(),
                data.clone(),
                tags,
                None,
            )
            .await
            .unwrap();

        let rpc = BcdbService::new(db);

        let mut query = HashMap::new();
        query.insert("common".into(), "value".into());
        let mut request = Request::new(QueryRequest {
            collection: "test".into(),
            tags: query,
        });

        // set required context on request
        Context::default()
            .with_auth(Authorization::Owner)
            .into_metadata(request.metadata_mut());

        let result = rpc.find(request).await;
        assert_eq!(result.is_ok(), true);

        let mut stream = result.unwrap().into_inner();

        let mut results: HashMap<u32, FindResponse> = HashMap::new();
        while let Some(result) = stream.recv().await {
            let result = result.unwrap();
            results.insert(result.id, result);
        }

        assert_eq!(results.len(), 2);
        let obj1 = results.get(&id_1).unwrap().clone();
        let metadata = obj1.metadata.unwrap();
        assert_eq!(metadata.collection, "test");
        assert_eq!(metadata.acl, Some(AclRef { acl: 3 }));
        assert_eq!(metadata.tags.get("common").unwrap(), "value");
        assert_eq!(metadata.tags.get("name").unwrap(), "object-1");

        let obj2 = results.get(&id_2).unwrap().clone();
        let metadata = obj2.metadata.unwrap();
        assert_eq!(metadata.collection, "test");
        assert_eq!(metadata.acl, None);
        assert_eq!(metadata.tags.get("common").unwrap(), "value");
        assert_eq!(metadata.tags.get("name").unwrap(), "object-2");
    }
}
