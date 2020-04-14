use tonic::transport::Server;
#[macro_use]
extern crate failure;

mod acl;
mod bcdb;
mod meta;
mod sonic;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let bcdb_service = bcdb::BcdbService::default();
    let acl_service = bcdb::AclService::default();

    Server::builder()
        .add_service(bcdb::BcdbServer::new(bcdb_service))
        .add_service(bcdb::AclServer::new(acl_service))
        .serve(addr)
        .await?;

    Ok(())
}
