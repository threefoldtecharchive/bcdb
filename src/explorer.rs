use crate::peer::{Peer, PeersList};
use anyhow::Result;
use async_trait::async_trait;
use surf;
use url::Url;

const BASE_URL: &str = "https://explorer.devnet.grid.tf/explorer/";

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
}

#[async_trait]
impl PeersList for Explorer {
    async fn get(&self, id: u32) -> Result<Peer> {
        let url = self.base.join(&format!("users/{}", id))?;
        let peer: Peer = match surf::get(url).recv_json().await {
            Ok(u) => u,
            Err(_) => bail!("failed to get user '{}'", id),
        };

        Ok(peer)
    }
}
