use tonic::transport::Server;
#[macro_use]
extern crate failure;

#[macro_use]
extern crate log;
//extern crate simple_logger;

mod acl;
mod bcdb;
mod meta;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

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
