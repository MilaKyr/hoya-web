use crate::db::relational::entities;
use crate::db::relational::entities::proxyparsingrules::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProxyParsingRules {
    pub table_lookup: String,
    pub head_lookup: String,
    pub row_lookup: String,
    pub data_lookup: String,
}

impl From<entities::proxyparsingrules::Model> for ProxyParsingRules {
    fn from(rules: Model) -> Self {
        Self {
            table_lookup: rules.table_name.to_string(),
            head_lookup: rules.head.to_string(),
            row_lookup: rules.row.to_string(),
            data_lookup: rules.data.to_string(),
        }
    }
}
