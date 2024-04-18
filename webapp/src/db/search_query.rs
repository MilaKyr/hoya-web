use crate::db::traits::ExternalText;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SearchQuery(String);

impl ExternalText for SearchQuery {
    fn cleaned(&self) -> Self {
        let query = self.0.clone();
        SearchQuery(self.clean(&query))
    }
}

impl SearchQuery {
    pub fn new(value: String) -> Self {
        SearchQuery(value)
    }
}

impl Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}
