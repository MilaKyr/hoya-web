use crate::data_models::{HoyaPosition, Proxy, ProxyParsingRules, Shop, ShopParsingRules};
use crate::db::in_memory::{InMemoryDB, PriceRange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use time::Date;
use url::Url;

pub mod in_memory;
mod map_json_as_pairs;

#[derive(Debug)]
pub enum Database {
    InMemory(InMemoryDB),
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

impl Database {
    pub fn all_products(&self) -> Vec<DatabaseProduct> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.all_products(),
        }
    }

    pub fn get_product_by(&self, id: u32) -> Option<DatabaseProduct> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_product_by(id),
        }
    }

    pub fn get_positions_for(&self, product: &DatabaseProduct) -> Vec<HoyaPosition> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_positions_for(product),
        }
    }

    pub fn get_prices_for(&self, product: &DatabaseProduct) -> HashMap<Date, f32> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_prices_for(product),
        }
    }

    pub fn save_proxies(&self, proxies: Vec<Proxy>) {
        match self {
            Database::InMemory(in_memory_db) => {
                in_memory_db.set_proxies(proxies);
            }
        }
    }

    pub fn search_with_filter(&self, filter: SearchFilter) -> Vec<DatabaseProduct> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.search_with_filters(filter),
        }
    }

    pub fn get_search_filter(&self) -> SearchFilter {
        let product_filter = match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_product_filter(),
        };
        SearchFilter {
            product: Some(product_filter),
            ..Default::default()
        }
    }

    pub fn get_shop_by(&self, id: u32) -> Option<Shop> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_shop_by(id),
        }
    }

    pub fn set_positions(&self, name: String, hoya_positions: Vec<HoyaPosition>) {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.set_positions(name, hoya_positions),
        }
    }

    pub fn get_positions_all(&self) -> HashMap<String, Vec<HoyaPosition>> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_positions_all(),
        }
    }

    pub fn get_top_shop(&self) -> Option<Shop> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_top_shop(),
        }
    }

    pub fn push_shop_back(&self, shop: &Shop) {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.push_shop_back(shop),
        }
    }

    pub fn get_all_shops(&self) -> Vec<Shop> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_all_shops(),
        }
    }

    pub fn get_shop_parsing_rules(&self, shop: &Shop) -> Option<ShopParsingRules> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_shop_parsing_rules(shop),
        }
    }

    pub fn set_proxies(&self, new_proxies: Vec<Proxy>) {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.set_proxies(new_proxies),
        }
    }

    pub fn get_proxies(&self) -> Vec<Proxy> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_proxies(),
        }
    }

    pub fn get_proxy_parsing_rules(&self) -> HashMap<Url, ProxyParsingRules> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_proxy_parsing_rules(),
        }
    }
}
