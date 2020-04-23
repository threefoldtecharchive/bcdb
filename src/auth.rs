use crate::identity::{PublicKey, Signature};
use failure::Error;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use surf;
use tokio::sync::Mutex;
use tonic::{Request, Status};
use url::Url;

const BASE_URL: &str = "https://explorer.devnet.grid.tf/explorer/users/";

pub struct Authenticator {
    base_url: Url,
    cache: Arc<Mutex<HashMap<u64, PublicKey>>>,
}

#[derive(Deserialize)]
struct User {
    pub pubkey: PublicKey,
}

impl Authenticator {
    pub fn new(base: Option<&str>) -> Result<Authenticator, Error> {
        Ok(Authenticator {
            base_url: match base {
                Some(base) => base,
                None => BASE_URL,
            }
            .parse()?,
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn get_key(&self, id: u64) -> Result<PublicKey, Error> {
        let mut cache = self.cache.lock().await;
        if let Some(key) = cache.get(&id) {
            return Ok(key.clone());
        }

        let url = self.base_url.join(&format!("{}", id))?;
        debug!("getting user info at: {}", url);
        let resp: User = match surf::get(url).recv_json().await {
            Ok(u) => u,
            Err(err) => bail!("failed to get user: {}", err),
        };

        debug!("user key retrieved");
        cache.insert(id, resp.pubkey.clone());

        Ok(resp.pubkey)
    }

    pub fn authenticate_blocking<T>(&self, request: Request<T>) -> Result<Request<T>, Status> {
        futures::executor::block_on(self.authenticate(request))
    }

    pub async fn authenticate<T>(&self, request: Request<T>) -> Result<Request<T>, Status> {
        let meta = request.metadata();

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

        let sig_str = match header.signature_str() {
            Ok(s) => s,
            Err(err) => {
                return Err(Status::unauthenticated(format!(
                    "failed to build signing string: {}",
                    err
                )))
            }
        };

        let key = match self.get_key(header.key_id).await {
            Ok(key) => key,
            Err(err) => {
                return Err(Status::unauthenticated(format!(
                    "could not get user key: {}",
                    err
                )))
            }
        };

        let signature = Signature::from_bytes(&match base64::decode(header.signature) {
            Ok(s) => s,
            Err(err) => {
                return Err(Status::unauthenticated(format!(
                    "invalid signature format expecting base64: {}",
                    err
                )))
            }
        });

        let signature = match signature {
            Ok(s) => s,
            Err(err) => {
                return Err(Status::unauthenticated(format!(
                    "invalid signature bytes: {}",
                    err
                )))
            }
        };

        match key.verify(sig_str.as_bytes(), &signature) {
            Ok(_) => (),
            Err(err) => {
                return Err(Status::unauthenticated(format!(
                    "failed to validate identity: {}",
                    err
                )))
            }
        };

        Ok(request)
    }
}

#[derive(Debug)]
struct AuthHeader {
    key_id: u64,
    algorithm: Option<String>,
    headers: String,
    signature: String,

    created: Option<u64>,
    expires: Option<u64>,
}

impl AuthHeader {
    fn valid(&self) -> Result<(), Error> {
        if self.headers.trim() == "" {
            bail!("invalid headers can not be empty");
        }
        use std::time::SystemTime;

        match self.created {
            Some(v) => {
                if v > SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs()
                {
                    bail!("(created) argument is in the future")
                }
            }
            None => (),
        };

        match self.expires {
            Some(v) => {
                if v < SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs()
                {
                    bail!("(expired) argument is in the past")
                }
            }
            None => (),
        };
        Ok(())
    }

    pub fn signature_str(&self) -> Result<String, Error> {
        self.valid()?;
        use std::fmt::Write;

        let mut sig_str = String::new();
        let headers = self.headers.split_ascii_whitespace();
        for header in headers {
            let value = match header {
                "(created)" => match self.created {
                    Some(v) => format!("{}", v),
                    None => bail!("(created) argument is not set"),
                },
                "(expires)" => match self.expires {
                    Some(v) => format!("{}", v),
                    None => bail!("(expired) argument is not set"),
                },
                "(key-id)" => format!("{}", self.key_id),
                _ => bail!("unknown signature string argument '{}'", header),
            };
            if sig_str.len() > 0 {
                sig_str.push('\n');
            }

            write!(sig_str, "{}: {}", header, value)?;
        }

        Ok(sig_str)
    }
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
            key_id: map
                .remove("keyId")
                .ok_or(format_err!("missing KeyId value in Authorization"))?
                .parse()
                .or_else(|v| bail!("invalid key-id format"))?,
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
            Ok(key) => println!("pubkey: {:?}", key),
            Err(err) => panic!(err), //assert_eq!(true, false, "failed to get user id: {}", err),
        };

        assert_eq!(auth.cache.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn get_user_blocking() {
        let auth = Authenticator::new(None).unwrap();
        match futures::executor::block_on(auth.get_key(1)) {
            Ok(key) => println!("pubkey: {:?}", key),
            Err(err) => panic!(err), //assert_eq!(true, false, "failed to get user id: {}", err),
        };
    }

    #[test]
    fn auth_header_parse() {
        let auth: AuthHeader = "Signature keyId=\"10\",algorithm=\"hs2019\",
        headers=\"(request-target) (created) host digest content-length\",
        signature=\"Base64(RSA-SHA512(signing string))\""
            .parse()
            .unwrap();

        assert_eq!(auth.algorithm, Some("hs2019".into()));
        assert_eq!(auth.key_id, 10);
        assert_eq!(auth.signature, "Base64(RSA-SHA512(signing string))");
        assert_eq!(
            auth.headers,
            "(request-target) (created) host digest content-length"
        )
    }
    #[test]
    fn auth_header_parse_invalid_key_id() {
        let auth: Result<AuthHeader, Error> = "Signature keyId=\"bad\",algorithm=\"hs2019\",
        headers=\"(request-target) (created) host digest content-length\",signature=\"some signature\""
            .parse();

        assert_eq!(auth.is_err(), true);
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
