use failure::Error;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Status};

const BASE_URL: &str = "https://explorer.devnet.grid.tf/explorer/users/";

pub struct Authenticator {
    client: Client,
    base_url: Url,
    cache: Arc<Mutex<HashMap<u64, String>>>,
}

#[derive(Deserialize)]
struct User {
    pub pubkey: String,
}

impl Authenticator {
    pub fn new(base: Option<&str>) -> Result<Authenticator, Error> {
        Ok(Authenticator {
            base_url: match base {
                Some(base) => base,
                None => BASE_URL,
            }
            .parse()?,
            client: Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn get_key(&self, id: u64) -> Result<String, Error> {
        let mut cache = self.cache.lock().await;
        if let Some(key) = cache.get(&id) {
            return Ok(key.into());
        }

        let url = self.base_url.join(&format!("{}", id))?;
        let resp: User = self.client.get(url).send().await?.json().await?;
        cache.insert(id, resp.pubkey.clone());

        Ok(resp.pubkey)
    }

    pub async fn authenticate<T>(&self, request: Request<T>) -> Result<Request<T>, Status> {
        let meta = request.metadata();
        for p in meta.iter() {
            println!("{:?}", p);
        }

        let header: AuthHeader = match meta.get("authorization") {
            None => return Err(Status::unauthenticated("missing authorization header")),
            Some(v) => match v.to_str().unwrap().parse() {
                Ok(header) => header,
                Err(err) => {
                    return Err(Status::unauthenticated(format!(
                        "invalid auth header: {}",
                        err
                    )))
                }
            },
        };

        debug!("Auth header: {:?}", header);

        Ok(request)
    }
}

#[derive(Debug)]
struct AuthHeader {
    keyId: String,
    algorithm: Option<String>,
    headers: String,
    signature: String,

    created: Option<u64>,
    expires: Option<u64>,
}

impl FromStr for AuthHeader {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const PREFIX: &str = "Signature ";
        enum Scan {
            Key,
            ValueStart,
            Value,
            Next,
        }
        match s.find(PREFIX) {
            Some(0) => (),
            _ => bail!("header is not starting with `Signature`"),
        };
        let mut mode = Scan::Key;
        let mut key = String::new();
        let mut value = String::new();

        let mut map: HashMap<String, String> = HashMap::new();
        //let mut previous = Mode::Unknown;
        for c in s[PREFIX.len()..].chars() {
            match mode {
                Scan::Key => {
                    if c == '=' {
                        mode = Scan::ValueStart;
                        continue;
                    }

                    key.push(c)
                }
                Scan::ValueStart => {
                    if c != '"' {
                        bail!("invalid value not starting with '\"'");
                    }
                    mode = Scan::Value;
                }
                Scan::Value => {
                    //TODO: skip sequence?
                    if c == '"' {
                        map.insert(key.trim().into(), value.clone());
                        key.clear();
                        value.clear();

                        mode = Scan::Next;
                        continue;
                    }
                    value.push(c)
                }
                Scan::Next => {
                    if c == ',' {
                        mode = Scan::Key
                    }
                }
            }
        }

        let header = AuthHeader {
            keyId: map
                .remove("keyId")
                .ok_or(format_err!("missing KeyId value in Authorization"))?,
            headers: map.remove("headers").unwrap_or("(created)".into()),
            algorithm: map.remove("algorithm"),
            signature: map
                .remove("signature")
                .ok_or(format_err!("missing signature value in Authorization"))?,
            created: match map.remove("created") {
                Some(v) => Some(v.parse()?),
                None => None,
            },
            expires: match map.remove("expires") {
                Some(v) => Some(v.parse()?),
                None => None,
            },
        };

        if map.len() > 0 {
            bail!("authorization header has unknown arguments");
        }

        Ok(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_user_key() {
        let auth = Authenticator::new(None).unwrap();
        match auth.get_key(1).await {
            Ok(key) => println!("pubkey: {}", key),
            Err(err) => panic!(err), //assert_eq!(true, false, "failed to get user id: {}", err),
        };

        assert_eq!(auth.cache.lock().await.len(), 1);
    }
    #[test]
    fn auth_header_parse() {
        let auth: AuthHeader = "Signature keyId=\"rsa-key-1\",algorithm=\"hs2019\",
        headers=\"(request-target) (created) host digest content-length\",
        signature=\"Base64(RSA-SHA512(signing string))\""
            .parse()
            .unwrap();

        assert_eq!(auth.algorithm, Some("hs2019".into()));
        assert_eq!(auth.keyId, "rsa-key-1");
        assert_eq!(auth.signature, "Base64(RSA-SHA512(signing string))");
        assert_eq!(
            auth.headers,
            "(request-target) (created) host digest content-length"
        )
    }
    #[test]
    fn auth_header_parse_missing_value() {
        let auth: Result<AuthHeader, Error> = "Signature keyId=\"rsa-key-1\",algorithm=\"hs2019\",
        headers=\"(request-target) (created) host digest content-length\""
            .parse();

        assert_eq!(auth.is_err(), true);
        auth.expect_err("missing signature value in Authorization");
    }
}
