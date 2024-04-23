use crate::db::traits::ExternalText;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use validator::{Validate, ValidationErrors};

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

impl Deref for SearchQuery {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Validate for SearchQuery {
    fn validate(&self) -> Result<(), ValidationErrors> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deref() {
        let query_str = "test";
        let query = SearchQuery(query_str.to_string());
        assert_eq!(*query, query_str.to_string())
    }

    #[test]
    fn test_cleaned_works() {
        let query_str = "test Test <script> exec(0) </script>";
        let query = SearchQuery(query_str.to_string());
        assert_eq!(
            *query.cleaned(),
            "test test script exec0 script".to_string()
        );
    }
}
