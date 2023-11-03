use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read with serde: {0}")]
    SerdeError(#[from] serde_json::error::Error),
    #[error("socket address parsing error: {0}")]
    SocketAddressParsingError(#[from] std::net::AddrParseError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}
