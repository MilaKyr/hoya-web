use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read with serde: {0}")]
    SerdeError(#[from] serde_json::error::Error),
    #[error("socket address parsing error")]
    SocketAddressParsingError(#[from] std::net::AddrParseError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}
