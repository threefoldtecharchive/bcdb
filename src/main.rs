use tonic::transport::Server;
#[macro_use]
extern crate failure;

mod bcdb;
mod sonic;

use clap::{App, Arg, SubCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("bcdb")
        .version("1.0")
        .about("")
        .arg(
            Arg::with_name("sonic")
                .long("sonic")
                .default_value("localhost:1491")
                .takes_value(true)
                .help("address of sonic"),
        )
        .arg(
            Arg::with_name("sonic-password")
                .long("sonic-password")
                .takes_value(true)
                .default_value("SecretPassword"),
        )
        .get_matches();

    let addr = "0.0.0.0:50051".parse()?;
    let service = bcdb::BcdbService::new(
        matches.value_of("sonic").unwrap(),
        matches.value_of("sonic-password").unwrap(),
    )
    .await?;

    Server::builder()
        .add_service(bcdb::BcdbServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
