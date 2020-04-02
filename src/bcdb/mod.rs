use generated::bcdb_server::Bcdb;
use generated::*;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

pub use generated::bcdb_server::BcdbServer;

pub mod generated {
    tonic::include_proto!("bcdb"); // The string specified here must match the proto package name
}

#[derive(Default)]
pub struct BcdbService;

#[tonic::async_trait]
impl Bcdb for BcdbService {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        Err(Status::unimplemented("not implemented yet!"))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        Err(Status::unimplemented("not implemented yet!"))
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
        Err(Status::unimplemented("not implemented yet!"))
    }

    type FindStream = mpsc::Receiver<Result<FindResponse, Status>>;

    async fn find(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<Self::FindStream>, Status> {
        Err(Status::unimplemented("not implemented yet!"))
    }
}
