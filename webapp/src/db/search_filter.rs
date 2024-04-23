use crate::db::product_filter::ProductFilter;
use crate::db::search_query::SearchQuery;
use crate::db::traits::ExternalText;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Default, Clone, Deserialize, Serialize, Validate)]
pub struct SearchFilter {
    #[validate(nested)]
    pub product: Option<ProductFilter>,
    #[validate(nested)]
    pub(crate) query: SearchQuery,
}

impl SearchFilter {
    pub fn query(&self) -> SearchQuery {
        self.query.cleaned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_some_product_nested_validation() {
        let filter = SearchFilter {
            product: Some(ProductFilter::default()),
            query: Default::default(),
        };
        assert!(filter.validate().is_ok())
    }

    #[test]
    fn test_none_product_nested_validation() {
        let filter = SearchFilter {
            product: None,
            query: Default::default(),
        };
        assert!(filter.validate().is_ok())
    }

    #[test]
    fn test_none_product_nested_validation_fails() {
        let filter = SearchFilter {
            product: Some(ProductFilter {
                price_min: Some(-100.),
                price_max: Some(1000.),
            }),
            query: Default::default(),
        };
        assert!(filter.validate().is_err())
    }

    #[test]
    fn test_query_nested_validation() {
        let filter = SearchFilter {
            product: Some(ProductFilter::default()),
            query: SearchQuery::new("Some string".to_string()),
        };
        assert!(filter.validate().is_ok())
    }
}
