use crate::data_models::UrlHolders;
use crate::db::relational::entities;
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShopParsingRules {
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
    pub fn with(
        rules: entities::shopparsingrules::Model,
        categories: Vec<entities::parsingcategory::Model>,
        lookups: entities::parsinglookup::Model,
    ) -> Self {
        ShopParsingRules {
            url_categories: categories
                .into_iter()
                .map(|cat| cat.category.clone())
                .collect(),
            parsing_url: rules.url,
            max_page_lookup: lookups.max_page.to_string(),
            product_table_lookup: lookups.product_table.to_string(),
            product_lookup: lookups.product.to_string(),
            name_lookup: lookups.name.to_string(),
            price_lookup: lookups.price.to_string(),
            url_lookup: lookups.url.to_string(),
            look_for_href: rules.look_for_href.unwrap_or_default(),
            sleep_timeout_sec: rules.sleep_timeout_sec.map(|val| val as u64),
        }
    }
    pub fn get_shop_parsing_url(&self, page_number: u32, category: &Option<String>) -> String {
        let mut url = self.parsing_url.clone();
        url = url.replace(&UrlHolders::PageID.to_string(), &page_number.to_string());
        if let Some(category) = category {
            url = url.replace(&UrlHolders::CategoryID.to_string(), category);
        }
        url
    }

    pub fn sleep(&self) -> Result<(), time::error::ConversionRange> {
        if let Some(duration_to_sleep) = self.sleep_timeout_sec {
            std::thread::sleep(Duration::try_from(time::Duration::seconds(
                duration_to_sleep as i64,
            ))?);
        }
        Ok(())
    }
}
