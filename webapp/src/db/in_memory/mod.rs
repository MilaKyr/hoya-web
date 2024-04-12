mod map_json_as_pairs;

use crate::db::errors::{DBError, InMemoryError};
use crate::db::product::DatabaseProduct;
use crate::db::product_filter::ProductFilter;
use crate::db::product_position::ShopPosition;
use crate::db::proxy::Proxy;
use crate::db::proxy_parsing_rules::ProxyParsingRules;
use crate::db::search_filter::SearchFilter;
use crate::db::shop::Shop;
use crate::db::shop_parsing_rules::ShopParsingRules;
use crate::db::SearchQuery;
use map_json_as_pairs::map_as_pairs;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::str::FromStr;
use std::sync::RwLock;
use time::Date;
use url::Url;

pub type ProductName = String;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct PriceRange {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FileStructure {
    pub proxies: Vec<Proxy>,
    pub shops: Vec<Shop>,
    #[serde(with = "map_as_pairs")]
    pub shops_parsing_rules: HashMap<Shop, ShopParsingRules>,
    #[serde(with = "map_as_pairs")]
    pub pictures: HashMap<ProductName, String>,
    #[serde(with = "map_as_pairs")]
    pub positions: HashMap<String, Vec<ShopPosition>>,
    #[serde(with = "map_as_pairs")]
    pub proxy_parsing_rules: HashMap<String, ProxyParsingRules>,
}

#[derive(Debug, Default)]
pub struct InMemoryDB {
    pub proxies: RwLock<Vec<Proxy>>,
    pub shops: RwLock<VecDeque<Shop>>,
    pub shops_parsing_rules: RwLock<HashMap<Shop, ShopParsingRules>>,
    pub pictures: RwLock<HashMap<ProductName, String>>,
    pub positions: RwLock<HashMap<ProductName, Vec<ShopPosition>>>,
    pub products: RwLock<Vec<DatabaseProduct>>,
    pub historic_prices: RwLock<HashMap<ProductName, HashMap<Date, f32>>>,
    pub proxy_parsing_rules: RwLock<HashMap<Url, ProxyParsingRules>>,
    pub price_range: PriceRange,
}

impl TryFrom<String> for InMemoryDB {
    type Error = DBError;

    fn try_from(file_path: String) -> Result<Self, Self::Error> {
        let data = fs::read_to_string(file_path)
            .map_err(|e| DBError::InMemoryError(InMemoryError::IoError(e)))?;
        let db: FileStructure = serde_json::from_str(&data)
            .map_err(|e| DBError::InMemoryError(InMemoryError::SerdeError(e)))?;
        let mut proxy_parsing_rules = HashMap::new();
        for (url, rules) in db.proxy_parsing_rules.iter() {
            if let Ok(url) = Url::from_str(url) {
                proxy_parsing_rules.insert(url, rules.to_owned());
            }
        }
        let positions = db.positions.values().flatten();
        let mut prices: Vec<_> = positions.into_iter().map(|pos| pos.price).collect();
        prices.sort_by(|a, b| a.partial_cmp(b).expect("Tried to compare a NaN"));
        Ok(Self {
            proxies: RwLock::new(db.proxies),
            shops: RwLock::new(VecDeque::from(db.shops)),
            shops_parsing_rules: RwLock::new(db.shops_parsing_rules),
            pictures: RwLock::new(db.pictures),
            positions: RwLock::new(db.positions),
            products: RwLock::new(vec![]), // TODO
            historic_prices: Default::default(),
            proxy_parsing_rules: RwLock::new(proxy_parsing_rules),
            price_range: PriceRange {
                min: prices.first().copied().unwrap_or_default(),
                max: prices.last().copied().unwrap_or_default(),
            },
        })
    }
}

impl InMemoryDB {
    pub fn save_positions(&self, positions: Vec<ShopPosition>) -> Result<(), DBError> {
        if let Some(pos1) = positions.first() {
            let mut all_positions = self.positions.write().unwrap();
            (*all_positions).insert(pos1.shop.name.to_string(), positions);
            return Ok(());
        }
        Err(DBError::NoProductShopPositions)
    }

    pub fn get_positions_all(&self) -> HashMap<ProductName, Vec<ShopPosition>> {
        let positions = self.positions.read().unwrap();
        positions.clone()
    }

    pub fn all_products(&self) -> Result<Vec<DatabaseProduct>, DBError> {
        let products = self.products.read().unwrap();
        Ok(products.clone())
    }
    pub fn get_shop_by(&self, id: u32) -> Option<Shop> {
        let shops = self.shops.read().unwrap();
        shops.get(id as usize).cloned()
    }

    fn search(
        &self,
        products: Vec<DatabaseProduct>,
        filter: &Option<ProductFilter>,
        query: SearchQuery,
    ) -> Result<Vec<DatabaseProduct>, DBError> {
        let mut selected = vec![];

        for product in products.iter() {
            let positions = self.get_positions_for(product)?;

            if positions.iter().any(|pos| {
                (pos.price
                    >= filter
                        .as_ref()
                        .and_then(|prod| prod.price_min)
                        .unwrap_or(0.0)
                    || pos.price
                        <= filter
                            .as_ref()
                            .and_then(|prod| prod.price_max)
                            .unwrap_or(f32::MAX))
                    && pos.full_name.contains(&query.to_string())
            }) {
                selected.push(product.clone());
            }
        }
        Ok(selected)
    }
    pub fn search_with_filter(
        &self,
        filter: SearchFilter,
    ) -> Result<Vec<DatabaseProduct>, DBError> {
        let all_products = self.products.read().unwrap().to_owned();
        if !filter.contains_query() {
            return Ok(all_products);
        }
        let query = filter.query().expect("Query cannot be empty");
        self.search(all_products, &filter.product, query)
    }

    pub fn get_product_filter(&self) -> Result<ProductFilter, DBError> {
        Ok(self.price_range.into())
    }

    pub fn get_product_by(&self, id: u32) -> Result<DatabaseProduct, DBError> {
        let products = self.products.read().unwrap();
        products
            .get(id as usize)
            .cloned()
            .ok_or(DBError::UnknownProduct)
    }

    pub fn get_positions_for(
        &self,
        product: &DatabaseProduct,
    ) -> Result<Vec<ShopPosition>, DBError> {
        let positions = self.positions.read().unwrap();
        positions
            .get(&product.name)
            .cloned()
            .ok_or(DBError::UnknownProduct)
    }

    pub fn get_prices_for(&self, product: &DatabaseProduct) -> Result<Vec<(Date, f32)>, DBError> {
        let positions = self.historic_prices.read().unwrap();
        let prices = positions.get(&product.name).cloned().unwrap_or_default();
        Ok(prices.into_iter().collect())
    }

    pub fn get_top_shop(&self) -> Result<Shop, DBError> {
        let mut shops = self.shops.write().unwrap();
        shops.pop_front().ok_or(DBError::ShopNotFound)
    }

    pub fn push_shop_back(&self, shop: &Shop) -> Result<(), DBError> {
        let mut shops = self.shops.write().unwrap();
        shops.push_back(shop.clone());
        Ok(())
    }

    pub fn get_all_shops(&self) -> Vec<Shop> {
        let shops = self.shops.read().unwrap();
        shops.clone().into_iter().collect::<Vec<Shop>>()
    }

    pub fn get_shop_parsing_rules(&self, shop: &Shop) -> Result<ShopParsingRules, DBError> {
        let shops_parsing_rules = self.shops_parsing_rules.read().unwrap();
        shops_parsing_rules
            .get(shop)
            .cloned()
            .ok_or(DBError::ParsingRulesNotFound)
    }

    pub fn save_proxies(&self, new_proxies: Vec<Proxy>) -> Result<(), DBError> {
        let mut proxies = self.proxies.write().unwrap();
        *proxies = new_proxies;
        Ok(())
    }

    pub fn get_proxies(&self) -> Result<Vec<Proxy>, DBError> {
        let proxies = self.proxies.read().unwrap();
        Ok(proxies.clone())
    }

    pub fn get_proxy_parsing_rules(&self) -> Result<HashMap<Url, ProxyParsingRules>, DBError> {
        let proxy_parsing_rules = self.proxy_parsing_rules.read().unwrap();
        Ok(proxy_parsing_rules.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_shop(name: &str) -> Shop {
        Shop {
            logo: "path/to/file".to_string(),
            name: name.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn set_get_positions_all_works() {
        let db = InMemoryDB::default();
        let name = "test shop";
        let shop = create_test_shop(name);
        let hoya_positions = vec![ShopPosition::new(
            shop,
            "full name".to_string(),
            1.2,
            "https://example.com".to_string(),
        )];

        db.save_positions(hoya_positions.clone())
            .expect("Failed to save positions");

        let mut expected_result = HashMap::new();
        expected_result.insert(name.to_string(), hoya_positions);
        let result = db.get_positions_all();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn set_get_proxies_works() {
        let expected_result = vec![Proxy::dummy("a"), Proxy::dummy("b"), Proxy::dummy("c")];
        let db = InMemoryDB::default();
        db.save_proxies(expected_result.clone())
            .expect("Failed to save proxies");

        let result = db.get_proxies().expect("Failed to get proxies");
        assert_eq!(result, expected_result);
    }

    #[test]
    fn get_proxy_parsing_rules_work() {
        let url = Url::from_str("https://example.com").expect("Failed to create url");
        let rules = ProxyParsingRules::default();

        let mut expected_result = HashMap::new();
        expected_result.insert(url.clone(), rules.clone());

        let db = InMemoryDB {
            proxy_parsing_rules: RwLock::new(expected_result.clone()),
            ..Default::default()
        };
        let result = db.get_proxy_parsing_rules().expect("Failed to parse");
        assert_eq!(result, expected_result);
    }

    #[test]
    fn get_all_shops_works() {
        let shop = create_test_shop("a");
        let expected_vec = vec![shop];
        let expected_result = expected_vec.clone().into_iter().collect::<VecDeque<Shop>>();
        let db = InMemoryDB {
            shops: RwLock::new(expected_result.clone()),
            ..Default::default()
        };
        let result = db.get_all_shops();
        assert_eq!(result, expected_vec);
    }

    #[test]
    fn get_top_shop_push_back_works() {
        let shop1 = create_test_shop("a");
        let shop2 = create_test_shop("b");
        let expected_result = vec![shop1.clone(), shop2.clone()]
            .into_iter()
            .collect::<VecDeque<Shop>>();
        let db = InMemoryDB {
            shops: RwLock::new(expected_result.clone()),
            ..Default::default()
        };
        let result1 = db.get_top_shop();
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), shop1);
        db.push_shop_back(&shop1).expect("Failed to oush shop back");
        let result2 = db.get_top_shop();
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), shop2);
    }
}
