use crate::db::errors::DBError;
use crate::db::product::DatabaseProduct;
use crate::db::relational::entities;
use crate::db::shop::Shop;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ShopPosition {
    pub shop: Shop,
    pub full_name: String,
    pub price: f32,
    pub url: String,
}

impl ShopPosition {
    pub fn new(shop: Shop, full_name: String, price: f32, url: String) -> Self {
        Self {
            shop,
            full_name,
            price,
            url,
        }
    }

    pub fn try_init(
        position: entities::shopposition::Model,
        shop: Shop,
        product: &DatabaseProduct,
    ) -> Result<Self, DBError> {
        Ok(Self {
            shop,
            full_name: product.name.to_string(),
            price: position.price.to_string().parse()?,
            url: position.url.to_string(),
        })
    }
}
