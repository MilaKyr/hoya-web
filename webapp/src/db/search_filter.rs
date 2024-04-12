use crate::db::product_filter::ProductFilter;
use crate::db::search_query::SearchQuery;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SearchFilter {
    pub product: Option<ProductFilter>,
    pub query: SearchQuery,
}

impl SearchFilter {
    pub fn contains_query(&self) -> bool {
        let query = self.query.to_string();
        !query.is_empty()
    }
    pub fn query(&self) -> Result<SearchQuery, Box<dyn Error>> {
        if self.contains_query() {
            return Ok(self.query.cleaned());
        }
        Err(Box::from("Query is empty".to_string()))
    }
}
