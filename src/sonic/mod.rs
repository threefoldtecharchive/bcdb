use failure::Error;
use tokio::prelude::*;

mod protocol;
use protocol::{Protocol, Response, Word};

type Result<T> = std::result::Result<T, Error>;

enum Channel {
    Search,
    Ingest,
    Control,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Search => write!(f, "search"),
            Channel::Ingest => write!(f, "ingest"),
            Channel::Control => write!(f, "control"),
        }
    }
}

struct Ingest {
    p: Protocol,
}

impl Ingest {
    async fn new<A, P>(add: A, password: P) -> Result<Ingest>
    where
        A: tokio::net::ToSocketAddrs,
        P: AsRef<str>,
    {
        let mut p = Protocol::connect(add).await?;
        println!("starting...");
        let response = p
            .run(vec![
                Word::single("START"),
                Word::single(Channel::Ingest.to_string()),
                Word::single(password),
            ])
            .await?;
        match response {
            Response::Started => {
                println!("started");
            }
            _ => bail!("got unexpected response: {:?}", response),
        };
        Ok(Ingest { p: p })
    }

    async fn push<S>(&mut self, collection: S, bucket: S, object: S, text: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let response = self
            .p
            .run(vec![
                Word::single("PUSH"),
                Word::single(collection),
                Word::single(bucket),
                Word::single(object),
                Word::multiple(text),
            ])
            .await?;

        match response {
            Response::Ok => Ok(()),
            _ => bail!("got unexpected response: {:?}", response),
        }
    }

    async fn pop<S>(&mut self, collection: S, bucket: S, object: S, text: S) -> Result<u32>
    where
        S: AsRef<str>,
    {
        let response = self
            .p
            .run(vec![
                Word::single("POP"),
                Word::single(collection),
                Word::single(bucket),
                Word::single(object),
                Word::multiple(text),
            ])
            .await?;

        match response {
            Response::Result(v) => Ok(v),
            _ => bail!("got unexpected response: {:?}", response),
        }
    }

    /**
     * count matches, note that object is only taken into account if bucket is set
     */
    async fn count<S>(&mut self, collection: S, bucket: Option<S>, object: Option<S>) -> Result<u32>
    where
        S: AsRef<str>,
    {
        let mut args = vec![Word::single("COUNT"), Word::single(collection)];
        if let Some(b) = bucket {
            args.push(Word::single(b));

            if let Some(o) = object {
                args.push(Word::single(o));
            }
        }

        let response = self.p.run(args).await?;

        match response {
            Response::Result(v) => Ok(v),
            _ => bail!("got unexpected response: {:?}", response),
        }
    }
}

struct Search {
    p: protocol::Protocol,
}

impl Search {
    async fn new<A, P>(add: A, password: P) -> Result<Search>
    where
        A: tokio::net::ToSocketAddrs,
        P: AsRef<str>,
    {
        let mut p = Protocol::connect(add).await?;
        println!("starting...");
        let response = p
            .run(vec![
                Word::single("START"),
                Word::single(Channel::Search.to_string()),
                Word::single(password),
            ])
            .await?;
        match response {
            Response::Started => {
                println!("started");
            }
            _ => bail!("got unexpected response: {:?}", response),
        };
        Ok(Search { p: p })
    }

    async fn query<S>(
        &mut self,
        collection: S,
        bucket: S,
        terms: S,
        limit: Option<u32>,
        offset: Option<u32>,
        locale: Option<S>,
    ) -> Result<Vec<String>>
    where
        S: AsRef<str>,
    {
        let mut args = vec![
            Word::single("QUERY"),
            Word::single(collection),
            Word::single(bucket),
            Word::multiple(terms),
        ];
        if let Some(l) = limit {
            args.push(Word::single(format!("LIMIT({})", l)));
        }

        if let Some(o) = offset {
            args.push(Word::single(format!("OFFSET({})", o)))
        }

        if let Some(l) = locale {
            args.push(Word::single(format!("LOCALE({})", l.as_ref())))
        }

        let response = self.p.run(args).await?;

        let id = match response {
            Response::Pending(id) => id,
            _ => bail!("got unexpected response: {:?}", response),
        };
        let results = self.p.read().await?;
        if let Response::Event(protocol::Event {
            id: id,
            kind: protocol::EventKind::Query,
            data,
        }) = results
        {
            return Ok(data);
        } else {
            bail!("invalid response from query: {:?}", results);
        }
    }
}

// #[tokio::main]
// async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
//     let mut ingest = Ingest::new("localhost:1491", "SecretPassword").await?;
//     let mut search = Search::new("localhost:1491", "SecretPassword").await?;

//     println!("insert");
//     ingest
//         .pop("testcol", "bucket", "id:123", "muhamad azmy")
//         .await?;

//     let r = search
//         .query("testcol", "bucket", "hello", None, None, None)
//         .await?;
//     println!("{:?}", r);
//     // println!("writing command");
//     // println!("trying to read 1");
//     // protocol.read().await?;

//     // protocol
//     //     .write(vec![
//     //         Word::single("START"),
//     //         Word::single(Channel::Ingest.to_string()),
//     //         Word::single("SecretPassword"),
//     //     ])
//     //     .await?;

//     // println!("tyring to read 2");
//     // protocol.read().await?;

//     Ok(())
// }
