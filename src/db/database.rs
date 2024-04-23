use crate::configuration::{DatabaseSettings, DatabaseType};
use crate::db::errors::DBError;
use crate::db::in_memory::InMemoryDB;
use crate::db::message::Message;
use crate::db::product::DatabaseProduct;
use crate::db::product_alert::ProductAlert;
use crate::db::product_position::ShopPosition;
use crate::db::proxy::Proxy;
use crate::db::proxy_parsing_rules::ProxyParsingRules;
use crate::db::relational::RelationalDB;
use crate::db::search_filter::SearchFilter;
use crate::db::shop::Shop;
use crate::db::shop_parsing_rules::ShopParsingRules;
use crate::errors::AppErrors;
use sea_orm::Database as SeaOrmDB;
use std::collections::HashMap;
use url::Url;

#[derive(Debug)]
pub enum Database {
    InMemory(Box<InMemoryDB>),
    Relational(RelationalDB),
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
    ) -> Result<Vec<ShopPosition>, DBError> {
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

    pub async fn save_positions(&self, positions: Vec<ShopPosition>) -> Result<(), DBError> {
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

    pub async fn register_message(&self, message: Message) -> Result<(), DBError> {
        match self {
            Database::InMemory(db) => db.register_message(message),
            Database::Relational(db) => db.register_message(message).await,
        }
    }

    pub async fn register_alert(&self, alert: ProductAlert) -> Result<(), DBError> {
        match self {
            Database::InMemory(db) => db.register_alert(alert),
            Database::Relational(db) => db.register_alert(alert).await,
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
