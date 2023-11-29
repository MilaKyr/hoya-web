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

#[derive(Debug, Copy, Clone)]
pub enum HoyaType {
    Cutting,
    Rooted,
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
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum HoyaSize {
    Small,
    Medium,
    Large,
    Unk,
}

impl HoyaSize {
    fn dummy() -> Self {
        let mut rng = thread_rng();
        let prob: f32 = rng.gen_range(0.0..1.0);
        if prob < 0.3 {
            Self::Small
        } else if prob < 0.5 {
            Self::Medium
        } else if prob < 0.7 {
            Self::Large
        } else {
            Self::Unk
        }
    }
}
impl std::fmt::Display for HoyaSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HoyaSize::Small => write!(f, "small"),
            HoyaSize::Medium => write!(f, "medium"),
            HoyaSize::Large => write!(f, "large"),
            HoyaSize::Unk => write!(f, "Not available"),
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

#[derive(Debug, Clone)]
pub struct ShopListing {
    pub shop: Shop,
    pub prod_type: HoyaType,
    pub size: HoyaSize,
    pub url: Url,
    pub price: f32,
}

impl ShopListing {
    pub fn dummy() -> Self {
        let mut rng = thread_rng();
        Self {
            shop: Shop::dummy(),
            prod_type: HoyaType::dummy(),
            size: HoyaSize::dummy(),
            url: Url::from_str("https://example.com").unwrap(),
            price: rng.gen_range(10.0..100.00),
        }
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
    pub price_lookups: String,
    pub url_lookup: String,
    #[serde(default)]
    pub look_for_href: bool,
    #[serde(default)]
    pub sleep_timeout_sec: Option<u64>,
}

impl ShopParsingRules {
    pub fn get_price_lookups(&self) -> String {
        self.price_lookups.clone()
    }

    pub fn get_shop_parsing_urls(&self, category: &str, n_pages: u32) -> Vec<String> {
        let mut parsing_urls = vec![];
        let mut parsing_url = self.parsing_url.clone();
        parsing_url = parsing_url.replace(&UrlHolders::CategoryID.to_string(), category);
        for page_number in 1..=n_pages {
            let paged_parsing_url =
                parsing_url.replace(&UrlHolders::PageID.to_string(), &page_number.to_string());
            parsing_urls.push(paged_parsing_url);
        }
        parsing_urls
    }

    pub fn get_shop_parsing_url(&self, page_number: u32, category: &Option<String>) -> String {
        let parsing_url = self.parsing_url.clone();
        match category {
            None => parsing_url.replace(&UrlHolders::PageID.to_string(), &page_number.to_string()),
            Some(cat) => {
                let url_with_page =
                    parsing_url.replace(&UrlHolders::PageID.to_string(), &page_number.to_string());
                url_with_page.replace(&UrlHolders::CategoryID.to_string(), cat)
            }
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
