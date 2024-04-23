use crate::db::relational::entities;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Shop {
    pub id: u32,
    pub logo: String,
    pub name: String,
    pub url: String,
}

impl Shop {
    pub fn dummy() -> Self {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        Self {
            logo: "../public/img/home_icon.png".to_string(),
            name: rand_string,
            ..Default::default()
        }
    }
}

impl From<entities::shop::Model> for Shop {
    fn from(shop: entities::shop::Model) -> Self {
        Self {
            id: shop.id as u32,
            logo: shop.logo,
            name: shop.name,
            url: shop.url,
        }
    }
}
