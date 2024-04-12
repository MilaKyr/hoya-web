use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SearchQuery(String);

impl SearchQuery {
    pub fn cleaned(&self) -> Self {
        let mut query = self.0.clone();
        query = query.trim().to_lowercase();
        query = query
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect::<String>();
        SearchQuery(query)
    }

    pub fn new(value: String) -> Self {
        SearchQuery(value)
    }
}

impl Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}
