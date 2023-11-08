use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fmt::Formatter;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone)]
pub struct Product {
    pub name: String,
    pub id: u32,
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

#[derive(Debug, Default, Clone)]
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
