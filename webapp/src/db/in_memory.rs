use crate::data_models::{Proxy, ProxyParsingRules, Shop, ShopParsingRules};
use serde;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use std::sync::RwLock;
use url::Url;

mod map_as_pairs {
    use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
    use serde::ser::{Serialize, Serializer};
    use std::fmt;
    use std::marker::PhantomData;

    pub fn serialize<K, V, M, S>(map: M, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: Serialize,
        V: Serialize,
        M: IntoIterator<Item = (K, V)>,
        S: Serializer,
    {
        serializer.collect_seq(map)
    }

    pub fn deserialize<'de, K, V, M, D>(deserializer: D) -> Result<M, D::Error>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
        M: Default + Extend<(K, V)>,
        D: Deserializer<'de>,
    {
        struct MapVisitor<K, V, M> {
            keys: PhantomData<K>,
            values: PhantomData<V>,
            map: PhantomData<M>,
        }

        impl<'de, K, V, M> Visitor<'de> for MapVisitor<K, V, M>
        where
            K: Deserialize<'de>,
            V: Deserialize<'de>,
            M: Default + Extend<(K, V)>,
        {
            type Value = M;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of key-value pairs")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut map = M::default();
                while let Some((k, v)) = seq.next_element()? {
                    map.extend(Some((k, v)));
                }
                Ok(map)
            }
        }

        deserializer.deserialize_seq(MapVisitor {
            keys: PhantomData,
            values: PhantomData,
            map: PhantomData,
        })
    }
}

pub type HoyaName = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HoyaPosition {
    pub shop: Shop,
    pub full_name: String,
    pub price: f32,
    pub url: String,
}

impl PartialEq for HoyaPosition {
    fn eq(&self, other: &Self) -> bool {
        self.shop == other.shop
            && self.full_name == other.full_name
            && (self.price - other.price).abs() < f32::EPSILON
            && self.url == other.url
    }
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
    pub proxy_parsing_rules: RwLock<HashMap<Url, ProxyParsingRules>>,
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
        Self {
            proxies: RwLock::new(db.proxies),
            shops: RwLock::new(VecDeque::from(db.shops)),
            shops_parsing_rules: RwLock::new(db.shops_parsing_rules),
            pictures: RwLock::new(db.pictures),
            positions: RwLock::new(db.positions),
            proxy_parsing_rules: RwLock::new(proxy_parsing_rules),
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

    pub fn get_positions_by(&self, name: &String) -> Option<Vec<HoyaPosition>> {
        let positions = self.positions.read().unwrap();
        positions.get(name).cloned()
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
    fn set_get_positions_by_works() {
        let db = InMemoryDB::default();
        let name1 = "a".to_string();
        let name2 = "b".to_string();
        let shop = create_test_shop("test shop");
        let hoya_positions1 = vec![HoyaPosition::new(
            shop.clone(),
            "full name".to_string(),
            1.2,
            "https://example.com".to_string(),
        )];
        let hoya_positions2 = vec![HoyaPosition::new(
            shop,
            "full name 2".to_string(),
            3.4,
            "https://example2.com".to_string(),
        )];

        db.set_positions(name1, hoya_positions1.clone());
        db.set_positions(name2.clone(), hoya_positions2.clone());

        let result = db.get_positions_by(&name2);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), hoya_positions2);
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
