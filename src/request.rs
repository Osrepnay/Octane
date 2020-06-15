use crate::constants::*;
use crate::util::Spliterator;
use std::cfg;
use std::collections::HashMap;
use std::ops::Deref;
use std::str;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RequestMethod<'a> {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
    Other(&'a str),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HttpVersion {
    Http11,
    Http10,
    Http02,
    Http09,
    HttpInvalid,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RequestLine<'a> {
    pub method: RequestMethod<'a>,
    pub path: &'a str,
    pub version: HttpVersion,
}

impl<'a> RequestLine<'a> {
    pub fn parse(request_line: &'a str) -> Option<Self> {
        let mut toks = request_line.split(SP);
        let method = toks.next()?;
        let path = toks.next()?;
        let version = toks.next()?;
        let (first, ver) = version.split_at(5);
        let enum_ver = match ver {
            "1.1" => HttpVersion::Http11,
            "1.0" => HttpVersion::Http10,
            "2.0" => HttpVersion::Http02,
            "0.9" => HttpVersion::Http09,
            _ => HttpVersion::HttpInvalid,
        };

        if cfg!(feature = "faithful") && (first != "HTTP/" || toks.next().is_some()) {
            return None;
        }
        let request_method = match method {
            "POST" => RequestMethod::Post,
            "GET" => RequestMethod::Get,
            "DELETE" => RequestMethod::Delete,
            "PUT" => RequestMethod::Put,
            "OPTIONS" => RequestMethod::Options,
            "HEAD" => RequestMethod::Head,
            "TRACE" => RequestMethod::Trace,
            "CONNECT" => RequestMethod::Connect,
            _ => RequestMethod::Other(method),
        };
        Some(Self {
            method: request_method,
            path,
            version: enum_ver,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Header<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> Header<'a> {
    pub fn parse(header: &'a str) -> Option<Self> {
        let mut toks = header.splitn(2, ':');
        let name = toks.next()?;
        if name.is_empty() {
            return None;
        }
        if cfg!(feature = "faithful") {
            for c in name.chars() {
                TOKEN_CHARS.get(&c)?;
            }
        }
        let value = toks.next()?.trim_start_matches(|c| c == SP || c == HT);
        if cfg!(feature = "faithful") && value.chars().any(is_ctl) {
            return None;
        }
        Some(Self { name, value })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request<'a> {
    pub request_line: RequestLine<'a>,
    pub headers: HashMap<String, String>,
    pub body: &'a [u8],
    #[cfg(feature = "raw_headers")]
    pub raw_headers: Vec<Header<'a>>,
    #[cfg(feature = "cookies")]
    pub cookies: Cookies,
}

impl<'a> Request<'a> {
    pub fn parse(request: &'a [u8]) -> Option<Self> {
        let mut toks = Spliterator::new(request, B_CRLF);
        toks.skip_empty();
        let line = toks.next().and_then(|v| match str::from_utf8(v) {
            Ok(s) => RequestLine::parse(s),
            Err(_) => None,
        })?;
        let mut headers: HashMap<String, String> = HashMap::new();
        #[cfg(feature = "raw_headers")]
        let mut raw_headers: Vec<Header> = Vec::new();
        let mut found_empty: bool = false;
        for tok in toks.by_ref() {
            if tok.is_empty() {
                if cfg!(feature = "faithful") {
                    if toks.finished {
                        return None;
                    }
                    found_empty = true;
                }
                break;
            }
            let parsed = Header::parse(match str::from_utf8(tok) {
                Ok(s) => s,
                Err(_) => return None,
            })?;
            headers
                .entry(parsed.name.to_ascii_lowercase())
                .and_modify(|v| *v = format!("{}, {}", v, parsed.value))
                .or_insert_with(|| parsed.value.to_owned());
            #[cfg(feature = "raw_headers")]
            raw_headers.push(parsed);
        }
        if cfg!(feature = "faithful") && !found_empty {
            return None;
        }
        let body = toks.string;
        #[cfg(feature = "cookies")]
        let cookies: Cookies;
        #[cfg(feature = "cookies")]
        if let Some(v) = headers.get("cookie") {
            cookies = Cookies::parse(v);
        } else {
            cookies = Default::default();
        }
        Some(Self {
            request_line: line,
            headers,
            #[cfg(feature = "raw_headers")]
            raw_headers,
            #[cfg(feature = "cookies")]
            cookies,
            body,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct KeepAlive {
    pub timeout: Option<u64>,
    pub max: Option<u64>,
}

impl KeepAlive {
    pub fn parse(header: &str) -> Self {
        let mut ret = Self {
            timeout: None,
            max: None,
        };
        for tok in header.split(',') {
            let trimmed = tok.trim();
            let eq_ind = match trimmed.find('=') {
                Some(v) => v,
                None => continue,
            };
            let (name, val_str) = trimmed.split_at(eq_ind);
            let val: u64 = match (&val_str[1..]).parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            match name {
                "timeout" => ret.timeout = Some(val),
                "max" => ret.max = Some(val),
                _ => continue,
            };
        }
        ret
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cookies {
    pub cookies: HashMap<String, String>,
}

impl Deref for Cookies {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.cookies
    }
}

impl Default for Cookies {
    fn default() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }
}

impl Cookies {
    pub fn parse(header: &str) -> Self {
        let mut hashmap: HashMap<String, String> = HashMap::new();
        for tok in header.split("; ") {
            let eq_ind = match tok.find('=') {
                Some(v) => v,
                None => continue,
            };
            let (first, second) = tok.split_at(eq_ind);
            hashmap.insert(first.to_owned(), second[1..].to_owned());
        }
        Self { cookies: hashmap }
    }
}
