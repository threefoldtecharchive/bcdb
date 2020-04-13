use generated::acl_server::Acl as AclServiceTrait;
use generated::bcdb_server::Bcdb as BcdbServiceTrait;

use generated::*;
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tonic::{Code, Request, Response, Status};

use crate::storage::{zdb::Collection, zdb::Zdb, Storage};

pub use generated::acl_server::AclServer;
pub use generated::bcdb_server::BcdbServer;

pub mod generated {
    tonic::include_proto!("bcdb"); // The string specified here must match the proto package name
}

trait FailureExt {
    fn status(&self) -> Status;
}

impl FailureExt for failure::Error {
    fn status(&self) -> Status {
        Status::new(Code::Internal, format!("{}", self))
    }
}

#[derive(Default)]
pub struct BcdbService {
    db: Zdb,
}

#[tonic::async_trait]
impl BcdbServiceTrait for BcdbService {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let request = request.into_inner();

        if let Some(ref metadata) = request.metadata {
            for t in metadata.tags.iter() {
                println!("Tag: {}", t.key);
                if let Some(ref value) = t.value {
                    match value {
                        tag::Value::String(v) => println!("\tstring: {}", v),
                        tag::Value::Double(v) => println!("\tdouble: {}", v),
                        tag::Value::Unsigned(v) => println!("\tunsigned: {}", v),
                        tag::Value::Number(v) => println!("\tnumber: {}", v),
                    }
                }
            }
        }

        // TODO: create from impl for Tonic status for StorageError
        let mut db = self.db.clone();
        let handle =
            spawn_blocking(move || db.set(None, &request.data).expect("failed to set data"));
        // TODO: proper error
        let id = handle.await.expect("failed to run blocking task");

        Ok(Response::new(SetResponse { id }))
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

        let acl = ACL {
            perm: match request.perm.parse() {
                Ok(perm) => perm,
                Err(err) => return Err(err.status()),
            },
            users: request.users,
        };

        let mut store = self.store.clone();
        match store.create(&acl) {
            Ok(k) => Ok(Response::new(AclCreateResponse { key: k })),
            Err(err) => Err(err.status()),
        }
    }
    async fn list(
        &self,
        request: Request<AclListRequest>,
    ) -> Result<Response<AclListResponse>, Status> {
        Err(Status::unimplemented("not implmeneted"))
    }
    async fn set(
        &self,
        request: Request<AclSetRequest>,
    ) -> Result<Response<AclSetResponse>, Status> {
        Err(Status::unimplemented("not implmeneted"))
    }
    async fn grant(
        &self,
        request: Request<AclUsersRequest>,
    ) -> Result<Response<AclUsersResponse>, Status> {
        Err(Status::unimplemented("not implmeneted"))
    }
    async fn revoke(
        &self,
        request: Request<AclUsersRequest>,
    ) -> Result<Response<AclUsersResponse>, Status> {
        Err(Status::unimplemented("not implmeneted"))
    }
}
