use crate::db::errors::DBError;
use crate::db::in_memory::{InMemoryDB, PriceRange, ShopParsingRules};
use crate::db::relational::entities;
use crate::db::relational::RelationalDB;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::{Database as SeaOrmDB, EntityTrait, FromQueryResult, ModelTrait, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use url::Url;

mod errors;
pub mod in_memory;
mod map_json_as_pairs;
mod relational;

use crate::configuration::{DatabaseSettings, DatabaseType};
use crate::db::relational::entities::proxyparsingrules::Model;
use crate::errors::AppErrors;
use crate::parser::errors::ParserError;
pub use errors::DBError as DatabaseError;

#[derive(Debug)]
pub enum Database {
    InMemory(Box<InMemoryDB>),
    Relational(RelationalDB),
}
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProxyParsingRules {
    pub table_lookup: String,
    pub head_lookup: String,
    pub row_lookup: String,
    pub data_lookup: String,
}

impl From<entities::proxyparsingrules::Model> for ProxyParsingRules {
    fn from(rules: Model) -> Self {
        Self {
            table_lookup: rules.table_name.to_string(),
            head_lookup: rules.head.to_string(),
            row_lookup: rules.row.to_string(),
            data_lookup: rules.data.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Proxy {
    pub ip: String,
    pub port: u16,
    pub https: bool,
}

impl Display for Proxy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let protocol = if self.https { "https" } else { "http" };
        write!(f, "{}://{}:{}", protocol, self.ip, self.port)
    }
}

impl Proxy {
    pub fn dummy(ip: &str) -> Self {
        Self {
            ip: ip.to_string(),
            port: 1,
            https: false,
        }
    }
}

impl TryFrom<Vec<(&String, &String)>> for Proxy {
    type Error = ParserError;

    fn try_from(row: Vec<(&String, &String)>) -> Result<Self, Self::Error> {
        let (mut ip, mut port, mut https) = (None, None, None);
        for (name, value) in row.into_iter() {
            match name.as_str() {
                "IP Address" => ip = Some(value.to_string()),
                "Port" => port = value.parse::<u16>().ok(),
                "Https" => https = Some(value == "yes"),
                _ => {}
            }
        }
        Ok(Self {
            ip: ip.ok_or(ParserError::NotAProxyRow)?,
            port: port.ok_or(ParserError::NotAProxyRow)?,
            https: https.ok_or(ParserError::NotAProxyRow)?,
        })
    }
}

impl TryFrom<Url> for Proxy {
    type Error = DBError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        Ok(Self {
            ip: url.host().ok_or(DBError::UrlParseError)?.to_string(),
            port: url.port().ok_or(DBError::UrlParseError)?,
            https: url.scheme() == "https",
        })
    }
}

impl Database {
    pub async fn try_from(settings: &DatabaseSettings) -> Result<Self, AppErrors> {
        settings.is_valid()?;
        match settings.db_type {
            DatabaseType::InMemory => {
                let file_path = settings.path_unchecked();
                let db = InMemoryDB::try_from(file_path)?;
                Ok(Self::InMemory(Box::new(db)))
            }
            DatabaseType::Relational => {
                let connection_settings = settings.relational_connection_unchecked();
                let connection = SeaOrmDB::connect(connection_settings)
                    .await
                    .map_err(|e| AppErrors::DatabaseError(DBError::Relational(e)))?;
                let db = RelationalDB::init(connection);
                Ok(Self::Relational(db))
            }
        }
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

    pub async fn get_positions_for(
        &self,
        product: &DatabaseProduct,
    ) -> Result<Vec<HoyaPosition>, DBError> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_positions_for(product),
            Database::Relational(db) => db.get_positions_for(product).await,
        }
    }

    pub async fn get_prices_for(
        &self,
        product: &DatabaseProduct,
    ) -> Result<Vec<(String, f32)>, DBError> {
        let mut prices = match self {
            Database::InMemory(db) => db.get_prices_for(product)?,
            Database::Relational(db) => db.get_prices_for(product).await?,
        };
        prices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(prices
            .into_iter()
            .map(|(date, price)| (date.to_string(), price))
            .collect())
    }

    pub async fn search_with_filter(
        &self,
        filter: SearchFilter,
    ) -> Result<Vec<DatabaseProduct>, DBError> {
        match self {
            Database::InMemory(db) => db.search_with_filter(filter),
            Database::Relational(db) => db.search_with_filter(filter).await,
        }
    }

    pub async fn get_search_filter(&self) -> Result<SearchFilter, DBError> {
        let product_filter = match self {
            Database::InMemory(db) => db.get_product_filter(),
            Database::Relational(db) => db.get_product_filter().await,
        }?;
        Ok(SearchFilter {
            product: Some(product_filter),
            ..Default::default()
        })
    }

    pub async fn save_positions(&self, positions: Vec<HoyaPosition>) -> Result<(), DBError> {
        match self {
            Database::InMemory(db) => db.save_positions(positions),
            Database::Relational(db) => db.save_positions(positions).await,
        }
    }

    pub async fn get_top_shop(&self) -> Result<Shop, DBError> {
        match self {
            Database::InMemory(db) => db.get_top_shop(),
            Database::Relational(db) => db.get_top_shop().await,
        }
    }

    pub async fn push_shop_back(&self, shop: &Shop) -> Result<(), DBError> {
        match self {
            Database::InMemory(db) => db.push_shop_back(shop),
            Database::Relational(db) => db.push_shop_back(shop).await,
        }
    }

    pub async fn get_shop_parsing_rules(&self, shop: &Shop) -> Result<ShopParsingRules, DBError> {
        match self {
            Database::InMemory(db) => db.get_shop_parsing_rules(shop),
            Database::Relational(db) => db.get_shop_parsing_rules(shop).await,
        }
    }

    pub async fn save_proxies(&self, new_proxies: Vec<Proxy>) -> Result<(), DBError> {
        match self {
            Database::InMemory(db) => db.save_proxies(new_proxies),
            Database::Relational(db) => db.save_proxies(new_proxies).await,
        }
    }

    pub async fn get_proxies(&self) -> Result<Vec<Proxy>, DBError> {
        match self {
            Database::InMemory(db) => db.get_proxies(),
            Database::Relational(db) => db.get_proxies().await,
        }
    }

    pub async fn get_proxy_parsing_rules(
        &self,
    ) -> Result<HashMap<Url, ProxyParsingRules>, DBError> {
        match self {
            Database::InMemory(db) => db.get_proxy_parsing_rules(),
            Database::Relational(db) => db.get_proxy_parsing_rules().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_http_to_string_works() {
        let proxy = Proxy {
            ip: "127.0.0.1".to_string(),
            port: 80,
            https: false,
        };
        let proxy_url = proxy.to_string();
        let expected_url = "http://127.0.0.1:80".to_string();
        assert_eq!(proxy_url, expected_url);
    }

    #[test]
    fn proxy_https_to_string_works() {
        let proxy = Proxy {
            ip: "127.0.0.1".to_string(),
            port: 80,
            https: true,
        };
        let proxy_url = proxy.to_string();
        let expected_url = "https://127.0.0.1:80".to_string();
        assert_eq!(proxy_url, expected_url);
    }

    #[test]
    fn proxy_from_row_http_works() {
        let proxy = Proxy::try_from(vec![
            (&"IP Address".to_string(), &"127.0.0.1".to_string()),
            (&"Port".to_string(), &"6464".to_string()),
            (&"Https".to_string(), &"no".to_string()),
        ])
        .expect("Failed to create proxy");
        let proxy_url = proxy.to_string();
        let expected_url = "http://127.0.0.1:6464".to_string();
        assert_eq!(proxy_url, expected_url);
    }

    #[test]
    fn proxy_from_row_https_works() {
        let proxy = Proxy::try_from(vec![
            (&"IP Address".to_string(), &"127.0.0.1".to_string()),
            (&"Port".to_string(), &"6464".to_string()),
            (&"Https".to_string(), &"yes".to_string()),
        ])
        .expect("Failed to create proxy");
        let proxy_url = proxy.to_string();
        let expected_url = "https://127.0.0.1:6464".to_string();
        assert_eq!(proxy_url, expected_url);
    }
}
