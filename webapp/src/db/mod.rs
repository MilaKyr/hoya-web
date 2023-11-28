use crate::data_models::{Proxy, ProxyParsingRules, Shop, ShopParsingRules};
use crate::db::in_memory::{HoyaPosition, InMemoryDB};
use std::collections::HashMap;
use url::Url;

pub mod in_memory;

#[derive(Debug)]
pub enum Database {
    InMemory(InMemoryDB),
}

impl Database {
    pub fn save_proxies(&self, proxies: Vec<Proxy>) {
        match self {
            Database::InMemory(in_memory_db) => {
                in_memory_db.set_proxies(proxies);
            }
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

    pub fn get_positions_by(&self, name: &String) -> Option<Vec<HoyaPosition>> {
        match self {
            Database::InMemory(in_memory_db) => in_memory_db.get_positions_by(name),
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
