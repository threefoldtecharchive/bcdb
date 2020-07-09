use anyhow::Context;
use clap::{App, Arg, SubCommand};
use identity::Identity;
use log::debug;
use std::net::SocketAddr;
use storage::{encrypted::EncryptedStorage, zdb::Zdb};
use tokio::runtime::Builder;
use tonic::transport::Server;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod acl;
mod auth;
mod database;
mod explorer;
mod identity;
mod peer;
mod rest;
mod rpc;
mod storage;

const MEAT_DIR: &str = ".bcdb-meta";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = Builder::default()
        .threaded_scheduler()
        .enable_all()
        .thread_stack_size(10 * 1024 * 1024) //set stack size to 10MB
        .build()
        .unwrap();

    runtime.block_on(async {
        match entry().await {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        };
    });

    Ok(())
}

async fn entry() -> Result<(), Box<dyn std::error::Error>> {
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
                .default_value("/tmp/bcdb.sock"),
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
        .subcommand(
            SubCommand::with_name("rebuild")
                .about("rebuild index from zdb")
                .arg(
                    Arg::with_name("from")
                        .long("from")
                        .short("f")
                        .help("only rebuild index with records after given timestamp")
                        .takes_value(true)
                        .required(false),
                ),
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

    // use sqlite meta data factory, to build a sqlite index
    let index = database::index::SqliteIndexBuilder::new(matches.value_of("meta").unwrap())?
        .build("metadata")
        .await?;

    // intercept the index to also store the metadata in zdb as well
    let index = database::index::MetaInterceptor::new(
        index,
        EncryptedStorage::new(identity.as_sk_bytes(), zdb.collection("metadata")),
    );

    if let Some(matches) = matches.subcommand_matches("rebuild") {
        let mut index = index;
        let from = match matches.value_of("from") {
            Some(s) => Some(
                s.parse()
                    .context("failed to parse 'from' value expecting timestamp")?,
            ),
            None => None,
        };
        index.rebuild(from).await?;
        return Ok(());
    }

    // the acl_store
    let acl_store = acl::ACLStorage::new(EncryptedStorage::new(
        identity.as_sk_bytes(),
        zdb.collection("acl"),
    ));

    let db = database::BcdbDatabase::new(
        EncryptedStorage::new(identity.as_sk_bytes(), zdb.collection("objects")),
        index,
        acl_store.clone(),
    );

    let peers = if matches.is_present("peers-file") {
        peer::Either::A(peer::PeersFile::new(
            matches.value_of("peers-file").unwrap(),
        )?)
    } else {
        peer::Either::B(explorer::Explorer::new(matches.value_of("explorer"))?)
    };

    // tracker cache peers from the given source, and validate their identity
    let tracker = peer::Tracker::new(std::time::Duration::from_secs(20 * 60), 1000, peers);

    let db = peer::Router::new(identity.clone(), db, tracker.clone());

    let interceptor = auth::Authenticator::new(tracker, identity.clone());
    let acl_interceptor = interceptor.clone();

    let bcdb_service = rpc::BcdbService::new(db.clone());

    //acl api
    let acl_service = rpc::AclService::new(acl_store.clone());

    //identity api
    let identity_service = rpc::IdentityService::new(identity.clone());

    let grpc_address: SocketAddr = matches.value_of("grpc").unwrap().parse()?;

    let rest_address = matches.value_of("rest").unwrap().into();
    tokio::spawn(async move {
        match rest::run(db, acl_store, rest_address).await {
            Ok(_) => {}
            Err(err) => {
                error!("failed to start rest api: {}", err);
                std::process::exit(1);
            }
        }
    });

    Server::builder()
        .add_service(rpc::BcdbServer::with_interceptor(
            bcdb_service,
            move |request| interceptor.authenticate_blocking(request),
        ))
        .add_service(rpc::AclServer::with_interceptor(
            acl_service,
            move |request| acl_interceptor.authenticate_blocking(request),
        ))
        .add_service(rpc::IdentityServer::new(identity_service))
        .serve(grpc_address)
        .await?;

    Ok(())
}
