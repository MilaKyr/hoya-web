use crate::db::relational::entities;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DatabaseProduct {
    pub name: String,
    pub id: u32,
}

impl From<entities::product::Model> for DatabaseProduct {
    fn from(prod: entities::product::Model) -> Self {
        Self {
            name: prod.name.to_string(),
            id: prod.id as u32,
        }
    }
}
