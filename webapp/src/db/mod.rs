use crate::data_models::{Proxy, ProxyParsingRules, ShopParsingRules};
use crate::db::in_memory::{InMemoryDB, PriceRange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use sea_orm::DatabaseConnection;
use url::{Url};
use crate::db::relational::entities;
use crate::db::errors::DBError;
use crate::db::relational::RelationalDB;

pub mod in_memory;
mod map_json_as_pairs;
mod relational;
mod errors;

pub use errors::DBError as DatabaseError;

#[derive(Debug)]
pub enum Database {
    InMemory(InMemoryDB),
    Relational(RelationalDB),
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SearchFilter {
    product: Option<ProductFilter>,
    query: SearchQuery,
}

impl SearchFilter {
    pub fn contains_query(&self) -> bool {
        !self.query.0.is_empty()
    }
    pub fn query(&self) -> Result<SearchQuery, Box<dyn Error>> {
        if self.contains_query() {
            return Ok(self.query.cleaned());
        }
        Err(Box::from("Query is empty".to_string()))
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SearchQuery(String);

impl SearchQuery {
    pub fn cleaned(&self) -> Self {
        let mut query = self.0.clone();
        query = query.trim().to_lowercase();
        query = query
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect::<String>();
        SearchQuery(query)
    }
}

impl Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DatabaseProduct {
    pub name: String,
    pub id: u32,
}

impl From<entities::product::Model> for DatabaseProduct {
    fn from(prod: entities::product::Model) -> Self {
        Self {
            name: prod.name.to_string(),
            id: prod.id as u32,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HoyaPosition {
    pub shop: Shop,
    pub full_name: String,
    pub price: f32,
    pub url: String,
}

impl HoyaPosition {
    pub fn new(shop: Shop, full_name: String, price: f32, url: String) -> Self {
        Self {
            shop,
            full_name,
            price,
            url,
        }
    }

    pub fn init(position: entities::shopposition::Model, shop: Shop, product: &DatabaseProduct) -> Self {
        Self {
            shop,
            full_name: product.name.to_string(),
            price: position.price.unwrap_or(sea_orm::entity::prelude::Decimal::new (0, 1)).to_string().parse::<f32>().unwrap(),
            url: position.url.to_string(),
        }
    }
}


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

impl Database {

    pub fn new_im_memory() -> Self {
        Self::InMemory(InMemoryDB::init())
    }

    pub fn new_relational(conn: DatabaseConnection) -> Self {
        Self::Relational(RelationalDB::init(conn))
    }

    pub async fn all_products(&self) -> Result<Vec<DatabaseProduct>, DBError> {
        match self {
            Database::InMemory(db) => db.all_products(),
            Database::Relational(db) => db.all_products().await,
        }
    }

    pub async fn get_product_by(&self, id: u32) -> Result<DatabaseProduct, DBError> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_product_by(id),
            Database::Relational(db) => db.get_product_by(id).await,
        }
    }

    pub async fn get_positions_for(&self, product: &DatabaseProduct) -> Result<Vec<HoyaPosition>, DBError> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_positions_for(product),
            Database::Relational(db) => db.get_positions_for(product).await,
        }
    }

    pub async fn get_prices_for(&self, product: &DatabaseProduct) -> Result<Vec<(String, f32)>, DBError> {
        let mut prices = match self {
            Database::InMemory(db) => db.get_prices_for(product)?,
            Database::Relational(db) => db.get_prices_for(product).await?
        };
        prices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(prices.into_iter().map(|(date, price)| (date.to_string(), price)).collect())
    }

    pub async fn save_proxies(&self, proxies: Vec<Proxy>) {
        match self {
            Database::InMemory(db) => db.set_proxies(proxies),
            Database::Relational(_db) => todo!()
        }
    }

    pub async fn search_with_filter(&self, filter: SearchFilter) -> Result<Vec<DatabaseProduct>, DBError> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.search_with_filters(filter),
            Database::Relational(_db) => todo!()
        }
    }

    pub async fn get_search_filter(&self) -> SearchFilter {
        let product_filter = match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_product_filter(),
            Database::Relational(_db) => todo!()
        };
        SearchFilter {
            product: Some(product_filter),
            ..Default::default()
        }
    }

    pub async fn get_shop_by(&self, id: u32) -> Option<Shop> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_shop_by(id),
            Database::Relational(_db) => todo!()
        }
    }

    pub async fn set_positions(&self, name: String, hoya_positions: Vec<HoyaPosition>) {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.set_positions(name, hoya_positions),
            Database::Relational(_db) => todo!()
        }
    }

    pub async fn get_positions_all(&self) -> HashMap<String, Vec<HoyaPosition>> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_positions_all(),
            Database::Relational(_db) => todo!()
        }
    }

    pub async fn get_top_shop(&self) -> Option<Shop> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_top_shop(),
            Database::Relational(_db) => todo!()
        }
    }

    pub async fn push_shop_back(&self, shop: &Shop) {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.push_shop_back(shop),
            Database::Relational(_db) => todo!()
        }
    }


    pub async fn get_shop_parsing_rules(&self, shop: &Shop) -> Option<ShopParsingRules> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_shop_parsing_rules(shop),
            Database::Relational(_db) => todo!()
        }
    }

    pub async fn set_proxies(&self, new_proxies: Vec<Proxy>) {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.set_proxies(new_proxies),
            Database::Relational(_db) => todo!()
        }
    }

    pub async fn get_proxies(&self) -> Vec<Proxy> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_proxies(),
            Database::Relational(_db) => todo!()
        }
    }

    pub async fn get_proxy_parsing_rules(&self) -> HashMap<Url, ProxyParsingRules> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_proxy_parsing_rules(),
            Database::Relational(_db) => todo!()
        }
    }
}
