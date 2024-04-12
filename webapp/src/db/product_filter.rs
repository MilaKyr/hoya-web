use crate::db::in_memory::PriceRange;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, FromQueryResult)]
pub struct ProductFilter {
    pub price_min: Option<f32>,
    pub price_max: Option<f32>,
}

impl From<PriceRange> for ProductFilter {
    fn from(price_range: PriceRange) -> Self {
        ProductFilter {
            price_min: Some(price_range.min),
            price_max: Some(price_range.max),
        }
    }
}
