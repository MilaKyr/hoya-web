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
}

#[derive(Error, Debug)]
pub enum InMemoryError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("failed to read with serde: {0}")]
    SerdeError(#[from] serde_json::error::Error),
}
