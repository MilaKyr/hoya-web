use crate::parser::errors::ParserError;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read with serde: {0}")]
    SerdeError(#[from] serde_json::error::Error),
    #[error("socket address parsing error: {0}")]
    SocketAddressParsingError(#[from] std::net::AddrParseError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    AppErrors(AppErrors),
}

#[derive(Error, Debug)]
pub enum AppErrors {
    #[error("failed parsing with: {0}")]
    ParserError(#[from] ParserError),
    #[error("failed to parse string as url: {0}")]
    UrlParseError(#[from] url::ParseError),
}
