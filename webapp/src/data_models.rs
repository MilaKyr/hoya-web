use crate::errors::AppErrors;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone)]
pub struct Product {
    pub name: String,
    pub id: u32,
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
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HoyaType {
    Cutting,
    Rooted,
    Unk,
}

impl HoyaType {
    fn dummy() -> Self {
        let mut rng = thread_rng();
        if rng.gen_bool(0.5) {
            return Self::Cutting;
        }
        Self::Rooted
    }
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

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Shop {
    pub logo_path: String,
    pub name: String,
}

impl Shop {
    pub fn dummy() -> Self {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        Self {
            logo_path: "../public/img/home_icon.png".to_string(),
            name: rand_string,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShopListing {
    pub shop: Shop,
    pub category: String,
    pub name: String,
    pub prod_type: HoyaType,
    pub url: Url,
    pub price: f32,
}

impl ShopListing {
    pub fn dummy() -> Self {
        let mut rng = thread_rng();
        Self {
            shop: Shop::dummy(),
            category: "category".to_string(),
            name: "test name".to_string(),
            prod_type: HoyaType::dummy(),
            url: Url::from_str("https://example.com").unwrap(),
            price: rng.gen_range(10.0..100.00),
        }
    }
}

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

impl TryFrom<HoyaPosition> for ShopListing {
    type Error = AppErrors;
    fn try_from(position: HoyaPosition) -> Result<Self, Self::Error> {
        let url = Url::from_str(&position.url)?;
        Ok(ShopListing {
            shop: position.shop,
            category: "NA".to_string(), // TODO
            name: position.full_name.clone(),
            prod_type: HoyaType::Unk, // TODO
            url,
            price: position.price,
        })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShopParsingRules {
    #[serde(default)]
    pub url_categories: Vec<String>,
    pub parsing_url: String,
    pub max_page_lookup: String,
    pub product_table_lookup: String,
    pub product_lookup: String,
    pub name_lookup: String,
    pub price_lookup: String,
    pub url_lookup: String,
    #[serde(default)]
    pub look_for_href: bool,
    #[serde(default)]
    pub sleep_timeout_sec: Option<u64>,
}

impl ShopParsingRules {
    pub fn get_shop_parsing_url(&self, page_number: u32, category: &Option<String>) -> String {
        let mut url = self.parsing_url.clone();
        url = url.replace(&UrlHolders::PageID.to_string(), &page_number.to_string());
        if let Some(category) = category {
            url = url.replace(&UrlHolders::CategoryID.to_string(), category);
        }
        url
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProxyParsingRules {
    pub table_lookup: String,
    pub head_lookup: String,
    pub row_lookup: String,
    pub data_lookup: String,
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
    pub fn from_row(row: Vec<(&String, &String)>) -> Self {
        let mut ip = String::default();
        let mut port = 80;
        let mut https = false;
        for (name, value) in row.into_iter() {
            if name == &"IP Address".to_string() {
                ip = value.to_string();
            }
            if name == &"Port".to_string() {
                port = value.parse::<u16>().unwrap();
            }
            if name == &"Https".to_string() && value == &"yes".to_string() {
                https = true;
            }
        }
        Self { ip, port, https }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let pos1 = HoyaPosition::new(shop.clone(), name.to_string(), price, url.to_string());
        let pos2 = HoyaPosition::new(shop.clone(), name.to_string(), price, url.to_string());
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn hoya_position_to_shop_listing_works() {
        let shop = Shop::dummy();
        let name = "test name";
        let price = 1.99;
        let url = "https://example.com";
        let proper_url = Url::from_str(url).expect("Failed to convert string to url");
        let position = HoyaPosition::new(shop.clone(), name.to_string(), price, url.to_string());
        let result: ShopListing = position
            .try_into()
            .expect("Failed to convert to shop listing");
        let expected_result = ShopListing {
            shop,
            category: "NA".to_string(),
            name: name.to_string(),
            prod_type: HoyaType::Unk,
            url: proper_url,
            price,
        };
        assert_eq!(result, expected_result);
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
        let proxy = Proxy::from_row(vec![
            (&"IP Address".to_string(), &"127.0.0.1".to_string()),
            (&"Port".to_string(), &"6464".to_string()),
            (&"Https".to_string(), &"no".to_string()),
        ]);
        let proxy_url = proxy.to_string();
        let expected_url = "http://127.0.0.1:6464".to_string();
        assert_eq!(proxy_url, expected_url);
    }

    #[test]
    fn proxy_from_row_https_works() {
        let proxy = Proxy::from_row(vec![
            (&"IP Address".to_string(), &"127.0.0.1".to_string()),
            (&"Port".to_string(), &"6464".to_string()),
            (&"Https".to_string(), &"yes".to_string()),
        ]);
        let proxy_url = proxy.to_string();
        let expected_url = "https://127.0.0.1:6464".to_string();
        assert_eq!(proxy_url, expected_url);
    }
}
