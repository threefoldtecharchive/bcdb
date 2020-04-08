use failure::Error;
use tokio::io::BufStream;
use tokio::net::TcpStream;
use tokio::prelude::*;

type Result<T> = std::result::Result<T, Error>;

//TODO: create proper error type

/// Word wraps string objects for proper string escaping
pub enum Word {
    Single(String),
    Multiple(String),
}

impl Word {
    pub fn single<S>(s: S) -> Word
    where
        S: AsRef<str>,
    {
        Word::Single(s.as_ref().into())
    }

    pub fn multiple<S>(s: S) -> Word
    where
        S: AsRef<str>,
    {
        Word::Multiple(s.as_ref().into())
    }

    pub fn format(self) -> String {
        match self {
            Word::Single(v) => v,
            Word::Multiple(v) => format!("\"{}\"", v),
        }
    }
}

pub struct Protocol {
    s: BufStream<TcpStream>,
}

impl Protocol {
    pub async fn connect<A>(addr: A) -> Result<Protocol>
    where
        A: tokio::net::ToSocketAddrs,
    {
        let s = BufStream::new(TcpStream::connect(addr).await?);
        let mut p = Protocol { s: s };
        let response = p.read().await?;
        match response {
            Response::Connected => {}
            _ => bail!("got an expected response: {:?}", response),
        };

        Ok(p)
    }

    pub async fn run(&mut self, cmd: Vec<Word>) -> Result<Response> {
        for word in cmd {
            self.s.write_all(word.format().as_ref()).await?;
            self.s.write_all(" ".as_ref()).await?;
        }
        self.s.write_all("\r\n".as_ref()).await?;
        self.s.flush().await?;

        self.read().await
    }

    pub async fn read(&mut self) -> Result<Response> {
        let mut s = String::new();
        self.s.read_line(&mut s).await?;
        Response::from(s)
    }
}

#[derive(Debug)]
pub enum EventKind {
    Suggest,
    Query,
}

impl std::str::FromStr for EventKind {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "SUGGEST" => Ok(EventKind::Suggest),
            "QUERY" => Ok(EventKind::Query),
            _ => bail!("unknown event kind: {}", s),
        }
    }
}

#[derive(Debug)]
pub struct Event {
    pub id: String,
    pub kind: EventKind,
    pub data: Vec<String>,
}

#[derive(Debug)]
pub enum Response {
    Ok,
    Connected,
    Started,
    Err(String),
    Result(u32),
    Pending(String),
    Event(Event),
}

impl Response {
    fn from(s: String) -> Result<Response> {
        let mut t = Tokenizer::new(&s);
        let head = match t.next() {
            Some(head) => head,
            None => bail!("failed to parse response command"),
        };
        if head == "OK" {
            return Ok(Response::Ok);
        } else if head == "CONNECTED" {
            // todo: process connection information
            return Ok(Response::Connected);
        } else if head == "ERR" {
            return Ok(Response::Err(t.tail().into()));
        } else if head == "STARTED" {
            // todo: process started information
            return Ok(Response::Started);
        } else if head == "RESULT" {
            let v: u32 = match t.next() {
                Some(v) => v.parse()?,
                None => bail!("result has invalid parameters: {}", s),
            };
            return Ok(Response::Result(v));
        } else if head == "PENDING" {
            let v = match t.next() {
                Some(v) => v,
                None => bail!("pending with no even id"),
            };
            return Ok(Response::Pending(v.into()));
        } else if head == "EVENT" {
            let kind: EventKind = match t.next() {
                Some(k) => k.parse()?,
                None => bail!("event doesn't have a kind"),
            };

            let id = match t.next() {
                Some(id) => id,
                None => bail!("event does not have an id"),
            };

            let mut data: Vec<String> = vec![];
            loop {
                let v = match t.next() {
                    Some(v) => v,
                    None => break,
                };
                data.push(v.into());
            }

            return Ok(Response::Event(Event {
                id: id.into(),
                kind: kind,
                data: data,
            }));
        }

        bail!("unknown response header: {} ({})", head, s)
    }
}

pub struct Tokenizer<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Tokenizer {
        Tokenizer { s: s.trim(), i: 0 }
    }

    pub fn tail(&self) -> &'a str {
        &self.s[self.i..]
    }

    pub fn next(&mut self) -> Option<&'a str> {
        let s = self.i;
        if s >= self.s.len() {
            return None;
        }

        let slice = self.s.as_bytes();

        let mut i = self.i;
        loop {
            if i >= slice.len() {
                break;
            }

            let v = slice[i];

            if v == ' ' as u8 {
                self.i = i + 1;
                return Some(&self.s[s..i]);
            } else if v == '<' as u8 {
                //forward to > but what if we have multiple < ?
                i = match self.s.find('>') {
                    Some(i) => i,
                    None => self.s.len() - 1, // last index
                };
            } else if v == '\\' as u8 {
                i += 1;
            }
            // TODO: handle quoted string
            i += 1;
        }

        self.i = self.s.len();
        Some(&self.s[s..])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tokenize() {
        let mut t = super::Tokenizer::new("hello world");
        let mut r: Vec<String> = vec![];
        loop {
            match t.next() {
                Some(v) => r.push(v.into()),
                None => break,
            };
        }
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], "hello");
        assert_eq!(r[1], "world");
    }

    #[test]
    fn tokenize_with_bracket() {
        let mut t = super::Tokenizer::new("hello <world v1.0>");
        let mut r: Vec<String> = vec![];
        loop {
            match t.next() {
                Some(v) => r.push(v.into()),
                None => break,
            };
        }
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], "hello");
        assert_eq!(r[1], "<world v1.0>");
    }

    #[test]
    fn tokenize_with_skip() {
        let mut t = super::Tokenizer::new("hello world\\ again");
        let mut r: Vec<String> = vec![];
        loop {
            match t.next() {
                Some(v) => r.push(v.into()),
                None => break,
            };
        }
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], "hello");
        assert_eq!(r[1], "world\\ again");
    }
}
