use thiserror::Error;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("transparent")]
    Relational(#[from] sea_orm::DbErr),
    #[error("unknown product")]
    UnknownProduct,
    #[error("no historic prices found")]
    PricesNotFound,
}