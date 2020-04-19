use clap::{App, Arg};
use tonic::transport::Server;

#[macro_use]
extern crate failure;

mod acl;
mod bcdb;
mod meta;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("bcdb")
        .arg(
            Arg::with_name("zdb")
                .help("local zdb port")
                .long("zdb")
                .short("z")
                .takes_value(true)
                .default_value("9900"),
        )
        .arg(
            Arg::with_name("listen")
                .help("listen on address")
                .long("listen")
                .short("l")
                .takes_value(true)
                .default_value("[::1]:50051"),
        )
        .arg(
            Arg::with_name("meta")
                .help("directory where metadata is stored")
                .long("meta")
                .short("m")
                .takes_value(true)
                .default_value("meta"),
        )
        .arg(
            Arg::with_name("debug")
                .help("enable debug logging")
                .long("debug")
                .short("d")
                .takes_value(false),
        )
        .get_matches();

    let level = if matches.is_present("debug") {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    simple_logger::init_with_level(level).unwrap();

    // use zdb storage implementation (namespace objects)
    let object_store =
        storage::zdb::Zdb::new(matches.value_of("zdb").unwrap().parse()?).collection("objects");
    // use sqlite meta data factory
    let meta_factory =
        meta::sqlite::SqliteMetaStoreFactory::new(matches.value_of("meta").unwrap())?;

    let bcdb_service = bcdb::BcdbService::new(object_store, meta_factory);

    let acl_store =
        storage::zdb::Zdb::new(matches.value_of("zdb").unwrap().parse()?).collection("acl");
    let acl_service = bcdb::AclService::new(acl_store);

    let addr = matches.value_of("listen").unwrap().parse()?;

    Server::builder()
        .add_service(bcdb::BcdbServer::new(bcdb_service))
        .add_service(bcdb::AclServer::new(acl_service))
        .serve(addr)
        .await?;

    Ok(())
}
