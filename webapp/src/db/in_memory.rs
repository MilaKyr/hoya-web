use crate::data_models::{HoyaPosition, Proxy, ProxyParsingRules, Shop, ShopParsingRules};
use crate::db::map_json_as_pairs::map_as_pairs;
use crate::db::{DatabaseProduct, ProductFilter, SearchFilter, SearchQuery};
use serde;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::RwLock;
use time::Date;
use url::Url;

pub type HoyaName = String;

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
    pub pictures: HashMap<HoyaName, String>,
    #[serde(with = "map_as_pairs")]
    pub positions: HashMap<String, Vec<HoyaPosition>>,
    #[serde(with = "map_as_pairs")]
    pub proxy_parsing_rules: HashMap<String, ProxyParsingRules>,
}

#[derive(Debug, Default)]
pub struct InMemoryDB {
    pub proxies: RwLock<Vec<Proxy>>,
    pub shops: RwLock<VecDeque<Shop>>,
    pub shops_parsing_rules: RwLock<HashMap<Shop, ShopParsingRules>>,
    pub pictures: RwLock<HashMap<HoyaName, String>>,
    pub positions: RwLock<HashMap<HoyaName, Vec<HoyaPosition>>>,
    pub products: RwLock<Vec<DatabaseProduct>>,
    pub historic_prices: RwLock<HashMap<HoyaName, HashMap<Date, f32>>>,
    pub proxy_parsing_rules: RwLock<HashMap<Url, ProxyParsingRules>>,
    pub price_range: PriceRange,
}

impl InMemoryDB {
    pub fn init() -> Self {
        let data = include_str!("data.json");
        let db: FileStructure = serde_json::from_str(data).expect("Fail to read JSON");
        let mut proxy_parsing_rules = HashMap::new();
        for (url, rules) in db.proxy_parsing_rules.into_iter() {
            if let Ok(url) = Url::from_str(&url) {
                proxy_parsing_rules.insert(url, rules);
            }
        }
        let mut min_price = 0.;
        let mut max_price = 0.;
        for (_, positions) in db.positions.iter() {
            let prices: Vec<f32> = positions.iter().map(|pos| pos.price).collect();
            let curr_min = prices
                .iter()
                .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare"))
                .unwrap_or(&0.0);
            if *curr_min < min_price {
                min_price = *curr_min;
            }
            let curr_max = prices
                .iter()
                .min_by(|a, b| b.partial_cmp(a).expect("Failed to compare"))
                .unwrap_or(&0.0);
            if *curr_max > max_price {
                max_price = *curr_max;
            }
        }
        Self {
            proxies: RwLock::new(db.proxies),
            shops: RwLock::new(VecDeque::from(db.shops)),
            shops_parsing_rules: RwLock::new(db.shops_parsing_rules),
            pictures: RwLock::new(db.pictures),
            positions: RwLock::new(db.positions),
            products: RwLock::new(vec![]), // TODO
            historic_prices: Default::default(),
            proxy_parsing_rules: RwLock::new(proxy_parsing_rules),
            price_range: PriceRange {
                min: min_price,
                max: max_price,
            },
        }
    }

    pub fn set_positions(&self, name: String, hoya_positions: Vec<HoyaPosition>) {
        let mut positions = self.positions.write().unwrap();
        (*positions).insert(name, hoya_positions);
    }

    pub fn get_positions_all(&self) -> HashMap<HoyaName, Vec<HoyaPosition>> {
        let positions = self.positions.read().unwrap();
        positions.clone()
    }

    pub fn all_products(&self) -> Vec<DatabaseProduct> {
        let products = self.products.read().unwrap();
        products.clone()
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
    ) -> Vec<DatabaseProduct> {
        let mut selected = vec![];

        for product in products.iter() {
            let positions = self.get_positions_for(product);

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
        selected
    }
    pub fn search_with_filters(&self, filter: SearchFilter) -> Vec<DatabaseProduct> {
        let all_products = self.products.read().unwrap().to_owned();
        if !filter.contains_query() {
            return all_products;
        }
        let query = filter.query().expect("Query cannot be empty");
        self.search(all_products, &filter.product, query)
    }

    pub fn get_product_filter(&self) -> ProductFilter {
        self.price_range.into()
    }

    pub fn get_product_by(&self, id: u32) -> Option<DatabaseProduct> {
        let products = self.products.read().unwrap();
        products.get(id as usize).cloned()
    }

    pub fn get_positions_for(&self, product: &DatabaseProduct) -> Vec<HoyaPosition> {
        let positions = self.positions.read().unwrap();
        positions.get(&product.name).cloned().unwrap_or_default()
    }

    pub fn get_prices_for(&self, product: &DatabaseProduct) -> HashMap<Date, f32> {
        let positions = self.historic_prices.read().unwrap();
        positions.get(&product.name).cloned().unwrap_or_default()
    }

    pub fn get_top_shop(&self) -> Option<Shop> {
        let mut shops = self.shops.write().unwrap();
        shops.pop_front()
    }

    pub fn push_shop_back(&self, shop: &Shop) {
        let mut shops = self.shops.write().unwrap();
        shops.push_back(shop.clone());
    }

    pub fn get_all_shops(&self) -> Vec<Shop> {
        let shops = self.shops.read().unwrap();
        shops.clone().into_iter().collect::<Vec<Shop>>()
    }

    pub fn get_shop_parsing_rules(&self, shop: &Shop) -> Option<ShopParsingRules> {
        let shops_parsing_rules = self.shops_parsing_rules.read().unwrap();
        shops_parsing_rules.get(shop).cloned()
    }

    pub fn set_proxies(&self, new_proxies: Vec<Proxy>) {
        let mut proxies = self.proxies.write().unwrap();
        *proxies = new_proxies;
    }

    pub fn get_proxies(&self) -> Vec<Proxy> {
        let proxies = self.proxies.read().unwrap();
        proxies.clone()
    }

    pub fn get_proxy_parsing_rules(&self) -> HashMap<Url, ProxyParsingRules> {
        let proxy_parsing_rules = self.proxy_parsing_rules.read().unwrap();
        proxy_parsing_rules.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_shop(name: &str) -> Shop {
        Shop {
            logo_path: "path/to/file".to_string(),
            name: name.to_string(),
        }
    }

    #[test]
    fn set_get_positions_all_works() {
        let db = InMemoryDB::default();
        let name = "a".to_string();
        let shop = create_test_shop("test shop");
        let hoya_positions = vec![HoyaPosition::new(
            shop,
            "full name".to_string(),
            1.2,
            "https://example.com".to_string(),
        )];

        db.set_positions("a".to_string(), hoya_positions.clone());

        let mut expected_result = HashMap::new();
        expected_result.insert(name.to_string(), hoya_positions.clone());
        let result = db.get_positions_all();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn set_get_proxies_works() {
        let expected_result = vec![Proxy::dummy("a"), Proxy::dummy("b"), Proxy::dummy("c")];
        let db = InMemoryDB::default();
        db.set_proxies(expected_result.clone());

        let result = db.get_proxies();
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
        let result = db.get_proxy_parsing_rules();
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
        assert!(result1.is_some());
        assert_eq!(result1.unwrap(), shop1);
        db.push_shop_back(&shop1);
        let result2 = db.get_top_shop();
        assert!(result2.is_some());
        assert_eq!(result2.unwrap(), shop2);
    }
}
