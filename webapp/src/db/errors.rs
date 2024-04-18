use thiserror::Error;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("transparent")]
    Relational(#[from] sea_orm::DbErr),
    #[error("transparent")]
    InMemoryError(#[from] InMemoryError),
    #[error("unknown product")]
    UnknownProduct,
    #[error("no historic prices found")]
    PricesNotFound,
    #[error("shop not found")]
    ShopNotFound,
    #[error("no parsing rules found")]
    ParsingRulesNotFound,
    #[error("no positions found")]
    NoProductShopPositions,
    #[error("transparent")]
    PriceError(#[from] std::num::ParseFloatError),
    #[error("transparent")]
    DateParseError(#[from] time::error::Parse),
    #[error("failed to parse string as url")]
    UrlParseError,
    #[error("either date or time is none")]
    DatetimeError,
    #[error("transparent")]
    NotAFloat(#[from] rust_decimal::Error),
}

#[derive(Error, Debug)]
pub enum InMemoryError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("failed to read with serde: {0}")]
    SerdeError(#[from] serde_json::error::Error),
}
