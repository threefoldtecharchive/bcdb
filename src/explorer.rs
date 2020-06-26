use crate::bcdb::generated::identity_client::IdentityClient;
use crate::bcdb::generated::SignRequest;
use crate::identity::{PublicKey, Signature};
use anyhow::Result;
use serde::Deserialize;
use surf;
use url::Url;

const BASE_URL: &str = "https://explorer.devnet.grid.tf/explorer/";

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

    async fn verify(&self) -> Result<()> {
        // this need to make a grpc call to the peer
        // identity grpc service.
        // and validate it indeed owns the sk associated
        // with this pk

        let con = self.connect().await?;

        use rand::distributions::Standard;
        use rand::{thread_rng, Rng};
        let nonce: Vec<u8> = thread_rng().sample_iter(&Standard).take(215).collect();

        let mut client = IdentityClient::new(con);
        let request = SignRequest {
            message: nonce.clone(),
        };
        let response = client.sign(request).await?.into_inner();
        let signature = Signature::from_bytes(&response.signature)?;

        self.key.verify(&nonce, &signature)?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Explorer {
    base: Url,
}

impl Explorer {
    pub fn new(base: Option<&str>) -> Result<Explorer> {
        let base: Url = match base {
            Some(base) => {
                // we need to make sure that the url always end up in /
                if base.ends_with('/') {
                    base.parse()?
                } else {
                    format!("{}/", base).parse()?
                }
            }
            None => BASE_URL.parse()?,
        };

        Ok(Explorer { base })
    }

    pub async fn get(&self, id: u32) -> Result<Peer> {
        let url = self.base.join(&format!("users/{}", id))?;
        debug!("explorer: getting user info at: {}", url);
        let peer: Peer = match surf::get(url).recv_json().await {
            Ok(u) => u,
            Err(err) => bail!("failed to get user: {}", err),
        };

        Ok(peer)
    }
}
