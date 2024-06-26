use crate::db::{Shop, ShopPosition};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub name: String,
    pub id: u32,
    pub listings: HashMap<Shop, Vec<Listing>>,
    pub history_prices: Vec<(String, f32)>,
}

pub enum UrlHolders {
    PageID,
    CategoryID,
}

impl Display for UrlHolders {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlHolders::PageID => write!(f, "__PAGE_ID__"),
            UrlHolders::CategoryID => write!(f, "__CATEGORY_ID__"),
        }
    }
}

impl Product {
    pub fn dummy() -> Self {
        let mut rng = thread_rng();
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        Self {
            name: rand_string,
            id: rng.gen(),
            listings: Default::default(),
            history_prices: Default::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HoyaType {
    Cutting,
    Rooted,
    Unk,
}

impl std::fmt::Display for HoyaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HoyaType::Cutting => write!(f, "cutting"),
            HoyaType::Rooted => write!(f, "rooted plant"),
            HoyaType::Unk => write!(f, "n/a"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Listing {
    pub category: Option<String>,
    pub name: String,
    pub url: String,
    pub price: f32,
}

impl Listing {
    pub fn dummy() -> Self {
        let mut rng = thread_rng();
        Self {
            category: Some("category".to_string()),
            name: "test name".to_string(),
            url: "https://example.com".to_string(),
            price: rng.gen_range(10.0..100.00),
        }
    }
}

impl From<&ShopPosition> for Listing {
    fn from(position: &ShopPosition) -> Self {
        Listing {
            category: None, // TODO
            name: position.full_name.clone(),
            url: position.url.clone(),
            price: position.price,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ShopParsingRules;

    #[test]
    fn url_holders_to_string_work() {
        assert_eq!(
            UrlHolders::CategoryID.to_string(),
            "__CATEGORY_ID__".to_string()
        );
        assert_eq!(UrlHolders::PageID.to_string(), "__PAGE_ID__".to_string());
    }

    #[test]
    fn hoya_type_to_string_work() {
        assert_eq!(HoyaType::Cutting.to_string(), "cutting".to_string());
        assert_eq!(HoyaType::Rooted.to_string(), "rooted plant".to_string());
        assert_eq!(HoyaType::Unk.to_string(), "n/a".to_string());
    }

    #[test]
    fn hoya_positions_equal() {
        let shop = Shop::dummy();
        let name = "test name";
        let price = 1.99;
        let url = "https://example.com";
        let pos1 = ShopPosition::new(shop.clone(), name.to_string(), price, url.to_string());
        let pos2 = ShopPosition::new(shop.clone(), name.to_string(), price, url.to_string());
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn get_shop_parsing_url_page_and_category_works() {
        let shop_parsing_rules = ShopParsingRules {
            parsing_url: "https://example.com/products/__CATEGORY_ID__?page=__PAGE_ID__"
                .to_string(),
            ..Default::default()
        };
        let url = shop_parsing_rules.get_shop_parsing_url(2, &Some("category_1".to_string()));
        let expected_url = "https://example.com/products/category_1?page=2".to_string();
        assert_eq!(url, expected_url);
    }

    #[test]
    fn get_shop_parsing_url_no_category_works() {
        let shop_parsing_rules = ShopParsingRules {
            parsing_url: "https://example.com/products/?page=__PAGE_ID__".to_string(),
            ..Default::default()
        };
        let url = shop_parsing_rules.get_shop_parsing_url(2, &None);
        let expected_url = "https://example.com/products/?page=2".to_string();
        assert_eq!(url, expected_url);
    }
}
