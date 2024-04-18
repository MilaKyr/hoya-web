use crate::db::traits::ExternalText;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub email: String,
    pub message: String,
}

impl ExternalText for Message {
    fn cleaned(&self) -> Self {
        Self {
            email: self.email.to_owned(),
            message: self.clean(&self.message),
        }
    }
}
