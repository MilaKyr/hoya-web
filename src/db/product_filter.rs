use crate::db::in_memory::PriceRange;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, FromQueryResult, Validate)]
pub struct ProductFilter {
    #[validate(range(min = 0.0))]
    pub price_min: Option<f32>,
    #[validate(range(min = 0.0))]
    pub price_max: Option<f32>,
}

impl From<PriceRange> for ProductFilter {
    fn from(price_range: PriceRange) -> Self {
        ProductFilter {
            price_min: Some(price_range.min.max(0.)),
            price_max: Some(price_range.max.max(0.)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_price_range() {
        let price_range = PriceRange {
            min: 4.0,
            max: 44.0,
        };
        let product_filter: ProductFilter = price_range.into();
        assert!(product_filter.price_max.is_some());
        assert!(product_filter.price_min.is_some());

        assert!(product_filter.price_min.unwrap() - price_range.min < f32::EPSILON);
        assert!(product_filter.price_max.unwrap() - price_range.max < f32::EPSILON);
    }

    #[test]
    fn test_from_price_range_negatives() {
        let price_range = PriceRange {
            min: -4.0,
            max: -44.0,
        };
        let product_filter: ProductFilter = price_range.into();
        assert!(product_filter.price_max.is_some());
        assert!(product_filter.price_min.is_some());

        assert!(product_filter.price_min.unwrap() < f32::EPSILON);
        assert!(product_filter.price_max.unwrap() < f32::EPSILON);
    }

    #[test]
    fn test_product_filter_2none_validation_works() {
        let product_filter = ProductFilter {
            price_min: None,
            price_max: None,
        };
        assert!(product_filter.validate().is_ok());
    }

    #[test]
    fn test_product_filter_min_none_validation_works() {
        let product_filter = ProductFilter {
            price_min: Some(4.),
            price_max: None,
        };
        assert!(product_filter.validate().is_ok());
    }

    #[test]
    fn test_product_filter_max_none_validation_works() {
        let product_filter = ProductFilter {
            price_min: None,
            price_max: Some(400.),
        };
        assert!(product_filter.validate().is_ok());
    }

    #[test]
    fn test_product_filter_somes_validation_works() {
        let product_filter = ProductFilter {
            price_min: Some(0.),
            price_max: Some(0.),
        };
        assert!(product_filter.validate().is_ok());
    }

    #[test]
    fn test_product_filter_validation_fails() {
        let product_filter = ProductFilter {
            price_min: Some(-0.1),
            price_max: Some(-1.),
        };
        assert!(product_filter.validate().is_err());
    }
}
