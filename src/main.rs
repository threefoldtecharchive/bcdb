use clap::{App, Arg};
use log::debug;
use tonic::transport::Server;

use std::fs::File;
use std::io::Read;

use identity::Identity;
use storage::{encrypted::EncryptedStorage, zdb::Zdb};

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

mod acl;
mod auth;
mod bcdb;
mod identity;
mod meta;
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
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .help("mnemonic of the seed to be used for the identity")
                .long("seed")
                .short("s")
                .takes_value(true)
                .env("SEED")
                .conflicts_with("seed_file")
                .required_unless("seed_file"),
        )
        .arg(
            Arg::with_name("seed_file")
                .help("path to the file containing the mnemonic")
                .long("seed-file")
                .takes_value(true)
                .required_unless("seed")
                .env("SEED_FILE"),
        )
        .arg(
            Arg::with_name("explorer")
                .help("explorer URL for phonebook entries validations")
                .long("explorer")
                .takes_value(true)
                .default_value("https://explorer.devnet.grid.tf/explorer/"),
        )
        .get_matches();

    let level = if matches.is_present("debug") {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    simple_logger::init_with_level(level).unwrap();
    let bot_id: u32 = matches.value_of("id").unwrap().parse()?;

    let identity = if matches.is_present("seed") {
        Identity::from_mnemonic(bot_id, matches.value_of("seed").unwrap())?
    } else {
        let mut file = File::open(matches.value_of("seed_file").unwrap())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Identity::from_mnemonic(bot_id, content)?
    };

    // for some reason a byte slice does not implement fmt::LowerHex or fmt::UpperHex so manually
    // show the bytes
    debug!(
        "Starting server with identity, public key {}",
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

    //bcdb storage api
    let bcdb_service = bcdb::BcdbService::new(identity.id(), local_bcdb);

    //acl api
    let acl_service = bcdb::AclService::new(acl_store);

    //identity api
    let identity_service = bcdb::IdentityService::new(identity);

    let addr = matches.value_of("listen").unwrap().parse()?;

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
        .serve(addr)
        .await?;

    Ok(())
}
