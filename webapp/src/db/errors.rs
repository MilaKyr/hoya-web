use thiserror::Error;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("transparent")]
    Relational(#[from] sea_orm::DbErr),
    #[error("unknown product")]
    UnknownProduct,
    #[error("no historic prices found")]
    PricesNotFound,
    #[error("no shops found")]
    ShopNotFound,
    #[error("no parsing rules found")]
    ParsingRulesNotFound,
    #[error("no positions found")]
    NoProductShopPositions,
}
