use clap::{App, Arg};
use identity::Identity;
use log::debug;
use std::net::SocketAddr;
use storage::{encrypted::EncryptedStorage, zdb::Zdb};
use tonic::transport::Server;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

mod acl;
mod auth;
mod bcdb;
mod explorer;
mod identity;
mod meta;
mod rest;
mod storage;

const MEAT_DIR: &str = ".bcdb-meta";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let meta_dir = match dirs::home_dir() {
        Some(p) => p.join(".bcdb-meta"),
        None => std::path::PathBuf::from(MEAT_DIR),
    };

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
            Arg::with_name("grpc")
                .help("listen on address for grpc api")
                .long("grpc")
                .short("g")
                .takes_value(true)
                .default_value("0.0.0.0:50051"),
        )
        .arg(
            Arg::with_name("rest")
                .help("listen unix socket for rest api")
                .long("rest")
                .short("r")
                .takes_value(true)
                .default_value("/var/run/bcdb.sock"),
        )
        .arg(
            Arg::with_name("meta")
                .help("directory where metadata is stored")
                .long("meta")
                .short("m")
                .takes_value(true)
                .default_value(meta_dir.to_str().unwrap_or(MEAT_DIR)),
        )
        .arg(
            Arg::with_name("debug")
                .help("enable debug logging")
                .long("debug")
                .short("d")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("id")
                .help("threebot ID for this bcdb instance")
                .long("threebot-id")
                .short("id")
                .required_unless("seed-file")
                .conflicts_with("seed-file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .help("mnemonic of the seed to be used for the identity")
                .long("seed")
                .short("s")
                .takes_value(true)
                .env("SEED")
                .conflicts_with("seed-file")
                .required_unless("seed-file"),
        )
        .arg(
            Arg::with_name("seed-file")
                .help("path to the file containing the mnemonic")
                .long("seed-file")
                .takes_value(true)
                .required_unless("seed")
                .env("seed-file"),
        )
        .arg(
            Arg::with_name("explorer")
                .help("explorer URL for phonebook entries validations")
                .long("explorer")
                .takes_value(true)
                .default_value("https://explorer.devnet.grid.tf/explorer/"),
        )
        .arg(
            Arg::with_name("peers-file")
                .help("path to file with peers list, otherwise use explorer")
                .long("peers-file")
                .takes_value(true)
                .required(false)
                .env("PEERS_FILE"),
        )
        .get_matches();

    let level = if matches.is_present("debug") {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    simple_logger::init_with_level(level).unwrap();

    let identity = if matches.is_present("seed") {
        let bot_id: u32 = matches.value_of("id").unwrap().parse()?;
        Identity::from_mnemonic(bot_id, matches.value_of("seed").unwrap())?
    } else {
        Identity::from_identity_file(matches.value_of("seed-file").unwrap())?
    };

    // for some reason a byte slice does not implement fmt::LowerHex or fmt::UpperHex so manually
    // show the bytes
    info!(
        "Starting server with identity, id: {}, and public-key: {}",
        identity.id(),
        identity.public_key()
    );

    debug!("Using identity private key as symmetric encryption key for zdb data");

    let zdb = Zdb::new(matches.value_of("zdb").unwrap().parse()?);

    // use sqlite meta data factory
    let meta_factory =
        meta::sqlite::SqliteMetaStoreFactory::new(matches.value_of("meta").unwrap())?;

    // the acl_store
    let acl_store = acl::ACLStorage::new(EncryptedStorage::new(
        identity.as_sk_bytes(),
        zdb.collection("acl"),
    ));

    let local_bcdb = bcdb::LocalBcdb::new(
        EncryptedStorage::new(identity.as_sk_bytes(), zdb.collection("objects")),
        meta_factory,
        acl_store.clone(),
    );

    let interceptor = auth::Authenticator::new(matches.value_of("explorer"), identity.id())?;
    let acl_interceptor = interceptor.clone();

    let peers = if matches.is_present("peers-file") {
        bcdb::Either::A(bcdb::PeersFile::new(
            matches.value_of("peers-file").unwrap(),
        )?)
    } else {
        bcdb::Either::B(bcdb::Explorer::new(matches.value_of("explorer"))?)
    };

    // tracker cache peers from the given source, and validate their identity
    let tracker = bcdb::Tracker::new(std::time::Duration::from_secs(20 * 60), 1000, peers);

    //bcdb storage api
    let bcdb_service = bcdb::BcdbService::new(identity.id(), local_bcdb, tracker);

    //acl api
    let acl_service = bcdb::AclService::new(acl_store);

    //identity api
    let identity_service = bcdb::IdentityService::new(identity.clone());

    let grpc_address: SocketAddr = matches.value_of("grpc").unwrap().parse()?;
    let rest_address = matches.value_of("rest").unwrap().into();

    let grpc_port = grpc_address.port();
    tokio::spawn(async move {
        match rest::run(identity, rest_address, grpc_port).await {
            Ok(_) => {}
            Err(err) => {
                error!("failed to start rest api: {}", err);
                std::process::exit(1);
            }
        }
    });

    Server::builder()
        .add_service(bcdb::BcdbServer::with_interceptor(
            bcdb_service,
            move |request| interceptor.authenticate_blocking(request),
        ))
        .add_service(bcdb::AclServer::with_interceptor(
            acl_service,
            move |request| acl_interceptor.authenticate_blocking(request),
        ))
        .add_service(bcdb::IdentityServer::new(identity_service))
        .serve(grpc_address)
        .await?;

    Ok(())
}
