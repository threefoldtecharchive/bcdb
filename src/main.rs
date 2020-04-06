use tonic::transport::Server;
#[macro_use]
extern crate failure;

mod bcdb;
mod sonic;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = bcdb::BcdbService::default();

    Server::builder()
        .add_service(bcdb::BcdbServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
