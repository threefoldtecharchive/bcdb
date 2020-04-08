use crate::sonic;
use failure::Error;
use generated::bcdb_server::Bcdb;
use generated::*;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

pub use generated::bcdb_server::BcdbServer;

pub mod generated {
    tonic::include_proto!("bcdb"); // The string specified here must match the proto package name
}

//TODO:
// We need to implement a way to Pool sonic connections in case of
// dropped connections, and to make sure we don't block waiting for
// some operations to finish before we use the connection

pub struct BcdbService {
    ingest: sonic::Ingest,
    search: sonic::Search,
}

impl BcdbService {
    pub async fn new(sonic: &str, password: &str) -> Result<BcdbService, Error> {
        Ok(BcdbService {
            ingest: sonic::Ingest::new(sonic, password).await?,
            search: sonic::Search::new(sonic, password).await?,
        })
    }
}

#[tonic::async_trait]
impl Bcdb for BcdbService {
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let request = request.into_inner();
        let tags = match request.metadata {
            Some(metadata) => metadata.tags,
            None => vec![],
        };
        use std::fmt::Write as FmtWrite;
        let mut buf = String::new();

        // we trying to insert some data into sonic
        // since sonic is text indexer, we will just
        // build a big text and send it for indexing
        // we of course needed the object ID first
        // from the data store, but since we don't have
        // this atm. we can now just return a guid or something

        for t in tags.iter() {
            // println!("Tag: {}", t.key);
            let _ = write!(&mut buf, "{}(", t.key);
            if let Some(ref value) = t.value {
                let _ = match value {
                    tag::Value::String(v) => write!(&mut buf, "{}", v),
                    tag::Value::Double(v) => write!(&mut buf, "{}", v),
                    tag::Value::Unsigned(v) => write!(&mut buf, "{}", v),
                    tag::Value::Number(v) => write!(&mut buf, "{}", v),
                };
                let _ = write!(&mut buf, ")\n");
            }
        }
        //println!("data {:?}", std::str::from_utf8(&request.data));

        Ok(Response::new(SetResponse { id: "test".into() }))
        //Err(Status::unimplemented("not implemented yet!"))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        println!("call to get: {}", request.into_inner().id);
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
