use generated::bcdb_server::Bcdb;
use generated::*;
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;
use tonic::{Request, Response, Status};

use crate::storage::{zdb::Zdb, Storage};

pub use generated::bcdb_server::BcdbServer;

pub mod generated {
    tonic::include_proto!("bcdb"); // The string specified here must match the proto package name
}

#[derive(Default)]
pub struct BcdbService {
    db: Zdb,
}

#[tonic::async_trait]
impl Bcdb for BcdbService {
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
        // the problem with this clone is that it moves a ref to self in the closure,
        // which forces us to mess with the lifetimes. Since the clone actually opens a new
        // connection, it is not cheap and blocks the reactor. Need to find a way to solve this.
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
