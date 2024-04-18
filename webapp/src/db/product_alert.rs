use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ProductAlert {
    pub product_id: u32,
    pub email: String,
    pub price_below: f32,
}
