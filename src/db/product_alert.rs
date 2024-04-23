use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Default, Clone, Deserialize, Serialize, Validate)]
pub struct ProductAlert {
    #[validate(range(min = 1))]
    pub product_id: u32,
    #[validate(email)]
    pub email: String,
    #[validate(range(exclusive_min = 0.0))]
    pub price_below: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_id_validation_fails() {
        let alert = ProductAlert {
            product_id: 0,
            email: "test@test.com".to_string(),
            price_below: 15.0,
        };
        assert!(alert.validate().is_err())
    }

    #[test]
    fn test_product_id_1_validation_works() {
        let alert = ProductAlert {
            product_id: 1,
            email: "test@test.com".to_string(),
            price_below: 15.0,
        };
        assert!(alert.validate().is_ok())
    }

    #[test]
    fn test_product_id_max_validation_works2() {
        let alert = ProductAlert {
            product_id: u32::MAX - 1,
            email: "test@test.com".to_string(),
            price_below: 15.0,
        };
        assert!(alert.validate().is_ok())
    }

    #[test]
    fn test_email_validation_fails() {
        let alert = ProductAlert {
            product_id: 10,
            email: "test.com".to_string(),
            price_below: 15.0,
        };
        assert!(alert.validate().is_err())
    }

    #[test]
    fn test_price_validation_fails() {
        let alert = ProductAlert {
            product_id: 10,
            email: "test@test.com".to_string(),
            price_below: -15.0,
        };
        assert!(alert.validate().is_err())
    }

    #[test]
    fn test_price_validation_fails2() {
        let alert = ProductAlert {
            product_id: 10,
            email: "test@test.com".to_string(),
            price_below: 0.,
        };
        assert!(alert.validate().is_err())
    }
}
