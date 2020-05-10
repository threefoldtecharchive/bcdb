use super::generated::identity_client::IdentityClient;
use super::generated::{SignRequest, SignResponse};
use crate::identity::PublicKey;
use async_trait::async_trait;
use failure::Error;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::path::Path;
use url::Url;

type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Clone)]
pub struct Peer {
    id: u32,

    #[serde(default)]
    name: String,

    #[serde(default)]
    email: String,

    #[serde(rename = "pubkey")]
    key: PublicKey,

    host: String,

    #[serde(default)]
    description: String,
}

impl Peer {
    pub async fn connect(&self) -> Result<IdentityClient<tonic::transport::Channel>> {
        Ok(IdentityClient::connect(self.host.clone()).await?)
    }

    async fn veirfy(&self) -> Result<()> {
        // this need to make a grpc call to the peer
        // identity grpc service.
        // and validate it indeed owns the sk associated
        // with this pk

        let mut client = self.connect().await?;
        let req = SignRequest {
            message: "random message goes here".into(),
        };
        let resp = client.sign(req).await?.into_inner();
        println!("sig: {}", resp.signature);

        unimplemented!();
    }
}

#[async_trait]
pub trait PeersList {
    async fn get(&self, id: u32) -> Result<Peer>;
}

pub struct PeersFile {
    peers: HashMap<u32, Peer>,
}

impl PeersFile {
    /// creates  new peers list from file. the file consists of
    /// peers serialized as json objects
    /// example:
    ///   {id: 1, "pubkey": "<public key 1>", host: "host1"}
    ///   {id: 2, "pubkey": "<public key 2>", host: "host2"}
    pub fn new<P: AsRef<Path>>(path: P) -> Result<PeersFile> {
        let file = File::open(path)?;
        PeersFile::try_from(file)
    }
}

impl TryFrom<File> for PeersFile {
    type Error = Error;
    fn try_from(f: File) -> Result<Self> {
        let peers = serde_json::Deserializer::from_reader(f).into_iter::<Peer>();
        let mut map = HashMap::new();
        for peer in peers {
            let peer = peer?;
            map.insert(peer.id, peer);
        }

        Ok(PeersFile { peers: map })
    }
}

#[async_trait]
impl PeersList for PeersFile {
    async fn get(&self, id: u32) -> Result<Peer> {
        match self.peers.get(&id) {
            Some(peer) => Ok(peer.clone()),
            None => bail!("peer not found"),
        }
    }
}

pub struct Explorer {
    base: Url,
}

impl Explorer {
    pub fn new<U: Into<Url>>(base: U) -> Explorer {
        Explorer { base: base.into() }
    }
}

#[async_trait]
impl PeersList for Explorer {
    async fn get(&self, id: u32) -> Result<Peer> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn peers_file() {
        let peers = "
        {\"id\": 1, \"host\": \"host1\", \"pubkey\": \"34c77fdf6c5ef24d5a6981be06f9109ba83b7e306cfad8141ce5f572b647cbeb\"}
        {\"id\": 2, \"host\": \"host2\", \"pubkey\": \"8d0ba0d199426a71d5cb933406ab3296db5441384a5c5a39f4435130cfb688dc\"}
        ";
        const fname: &str = "/tmp/peers.file.test"; //TODO: use tempfile
        let mut file = File::create(fname).unwrap();
        use std::io::Write;

        file.write_all(peers.as_bytes()).unwrap();
        drop(file);
        let file = File::open(fname).unwrap();
        let peers_file = PeersFile::try_from(file).unwrap();
        let peer = peers_file.get(1).await.unwrap();
        assert_eq!(peer.host, "host1");

        std::fs::remove_file(fname);
    }
}
