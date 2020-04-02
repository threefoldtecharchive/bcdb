use generated::bcdb_server::Bcdb;
use generated::*;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

pub use generated::bcdb_server::BcdbServer;

pub mod generated {
    tonic::include_proto!("bcdb"); // The string specified here must match the proto package name
}

#[derive(Default)]
pub struct BcdbService;

#[tonic::async_trait]
impl Bcdb for BcdbService {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let request = request.into_inner();
        let tags = match request.metadata {
            Some(metadata) => metadata.tags,
            None => vec![],
        };

        for t in tags.iter() {
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

        println!("data {:?}", std::str::from_utf8(&request.data));

        Ok(Response::new(SetResponse { id: "test".into() }))
        //Err(Status::unimplemented("not implemented yet!"))
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
        let (mut tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            for i in 0..3 {
                tx.send(Ok(ListResponse {
                    id: format!("{}", i),
                }))
                .await
                .unwrap();
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
