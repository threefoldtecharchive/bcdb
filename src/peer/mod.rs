use crate::identity::PublicKey;
use anyhow::Error;
use async_trait::async_trait;
use lru_time_cache::LruCache;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

mod router;
pub use router::Router;

type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Clone)]
pub struct Peer {
    pub id: u32,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub email: String,

    #[serde(rename = "pubkey")]
    pub key: PublicKey,

    pub host: String,

    #[serde(default)]
    pub description: String,
}

impl Peer {
    pub async fn connect(&self) -> Result<tonic::transport::Channel> {
        debug!("connecting to peer: {}", self.host);
        let con = tonic::transport::Endpoint::new(self.host.clone())?
            .connect()
            .await?;

        Ok(con)
    }
}

#[cfg(test)]
impl Peer {
    pub fn new(id: &crate::identity::Identity) -> Self {
        Peer {
            id: id.id(),
            name: "".into(),
            email: "".into(),
            key: id.public_key(),
            host: "".into(),
            description: "".into(),
        }
    }
}

#[async_trait]
pub trait PeersList: Sync + Send + 'static {
    async fn get(&self, id: u32) -> Result<Peer>;
}

#[derive(Clone)]
pub struct PeersFile {
    peers: Arc<HashMap<u32, Peer>>,
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

        Ok(PeersFile {
            peers: Arc::new(map),
        })
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

#[async_trait]
impl PeersList for HashMap<u32, Peer> {
    async fn get(&self, id: u32) -> Result<Peer> {
        match Self::get(self, &id) {
            None => bail!("peer not found"),
            Some(peer) => Ok(peer.clone()),
        }
    }
}

// TODO: Better name?
/// Tracker tracks peers using PeerList as a source for ip address
/// and public key. Then it does identity check and cache this identity
/// for quick access. Tracker is responsible of making sure peer is valid
/// before it's used by the system
#[derive(Clone)]
pub struct Tracker<L>
where
    L: PeersList,
{
    list: L,
    cache: Arc<Mutex<LruCache<u32, Peer>>>,
}

impl<L> Tracker<L>
where
    L: PeersList,
{
    pub fn new(ttl: Duration, cap: usize, list: L) -> Self {
        let lru = LruCache::with_expiry_duration_and_capacity(ttl, cap);

        Tracker {
            list,
            cache: Arc::new(Mutex::new(lru)),
        }
    }
}

#[async_trait]
impl<L> PeersList for Tracker<L>
where
    L: PeersList,
{
    async fn get(&self, id: u32) -> Result<Peer> {
        let mut cache = self.cache.lock().await;
        if let Some(peer) = cache.get(&id) {
            return Ok(peer.clone());
        }
        // NOTICE:
        // both `get` and `verify` might take long time
        // to finish, but we are holding a lock to the cache
        // which means calls to Tracker.get() will block to
        // other calls even if it tries to get a peer that is
        // already in the cache.
        // if we drop the lock to allow other calls to go through
        // we might end up doing multiple calls to the possibly expensive
        // get and verify.
        let peer = self.list.get(id).await?;

        // TODO:
        // implement peer.verify().await?;
        cache.insert(id, peer.clone());

        Ok(peer)
    }
}

#[derive(Clone)]
pub enum Either<A, B>
where
    A: PeersList,
    B: PeersList,
{
    A(A),
    B(B),
}

#[async_trait]
impl<A, B> PeersList for Either<A, B>
where
    A: PeersList,
    B: PeersList,
{
    async fn get(&self, id: u32) -> Result<Peer> {
        match self {
            Either::A(ref a) => a.get(id).await,
            Either::B(ref b) => b.get(id).await,
        }
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

        std::fs::remove_file(fname).unwrap();
    }
}
