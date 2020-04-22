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
    algorithm: String,
    headers: String,
    signature: String,
}

impl FromStr for AuthHeader {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const PREFIX: &str = "Signature ";
        enum Mode {
            Key,
            ValueStart,
            Value,
            Next,
        }
        match s.find(PREFIX) {
            Some(0) => (),
            _ => bail!("header is not starting with `Signature`"),
        };
        let mut mode = Mode::Key;
        let mut key = String::new();
        let mut value = String::new();

        let mut map: HashMap<String, String> = HashMap::new();
        //let mut previous = Mode::Unknown;
        for c in s[PREFIX.len()..].chars() {
            match mode {
                Mode::Key => {
                    if c == '=' {
                        mode = Mode::ValueStart;
                        continue;
                    }

                    key.push(c)
                }
                Mode::ValueStart => {
                    if c != '"' {
                        bail!("invalid value not starting with '\"'");
                    }
                    mode = Mode::Value;
                }
                Mode::Value => {
                    //TODO: skip sequence?
                    if c == '"' {
                        map.insert(key.trim().into(), value.clone());
                        key.clear();
                        value.clear();

                        mode = Mode::Next;
                        continue;
                    }
                    value.push(c)
                }
                Mode::Next => {
                    if c == ',' {
                        mode = Mode::Key
                    }
                }
            }
        }
        Ok(AuthHeader {
            keyId: map
                .get("keyId")
                .ok_or(format_err!("missing KeyId value in Authorization"))?
                .into(),
            headers: map
                .get("headers")
                .ok_or(format_err!("missing headers value in Authorization"))?
                .into(),
            algorithm: map
                .get("algorithm")
                .ok_or(format_err!("missing algorithm value in Authorization"))?
                .into(),
            signature: map
                .get("signature")
                .ok_or(format_err!("missing signature value in Authorization"))?
                .into(),
        })
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

        assert_eq!(auth.algorithm, "hs2019");
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
