use crate::db::errors::DBError;
use crate::parser::errors::ParserError;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Proxy {
    pub ip: String,
    pub port: u16,
    pub https: bool,
}

impl Display for Proxy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let protocol = if self.https { "https" } else { "http" };
        write!(f, "{}://{}:{}", protocol, self.ip, self.port)
    }
}

impl Proxy {
    pub fn dummy(ip: &str) -> Self {
        Self {
            ip: ip.to_string(),
            port: 1,
            https: false,
        }
    }
}

impl TryFrom<Vec<(&String, &String)>> for Proxy {
    type Error = ParserError;

    fn try_from(row: Vec<(&String, &String)>) -> Result<Self, Self::Error> {
        let (mut ip, mut port, mut https) = (None, None, None);
        for (name, value) in row.into_iter() {
            match name.as_str() {
                "IP Address" => ip = Some(value.to_string()),
                "Port" => port = value.parse::<u16>().ok(),
                "Https" => https = Some(value == "yes"),
                _ => {}
            }
        }
        Ok(Self {
            ip: ip.ok_or(ParserError::NotAProxyRow)?,
            port: port.ok_or(ParserError::NotAProxyRow)?,
            https: https.ok_or(ParserError::NotAProxyRow)?,
        })
    }
}

impl TryFrom<Url> for Proxy {
    type Error = DBError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        Ok(Self {
            ip: url.host().ok_or(DBError::UrlParseError)?.to_string(),
            port: url.port().ok_or(DBError::UrlParseError)?,
            https: url.scheme() == "https",
        })
    }
}
