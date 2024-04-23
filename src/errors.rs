use crate::parser::errors::ParserError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppErrors {
    #[error("failed parsing with: {0}")]
    ParserError(#[from] ParserError),
    #[error("transparent")]
    DatabaseError(#[from] crate::db::DatabaseError),
    #[error("transparent")]
    ConfigurationError(#[from] ConfigurationError),
    #[error("transparent")]
    ValidationError(#[from] validator::ValidationErrors),
}

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("one or more settings are missed: provider, host, port, user, password")]
    MissingDatabaseSettings,
    #[error("data file not found")]
    DataFileNotFound,
    #[error("cannot be empty: {0}")]
    CannotBeEmpty(String),
    #[error("Unknown database type")]
    UnknownDatabaseType,
}

impl IntoResponse for AppErrors {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppErrors::ValidationError(s) => (StatusCode::BAD_REQUEST, s.to_string()),
            AppErrors::ParserError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s.to_string()),
            AppErrors::DatabaseError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s.to_string()),
            AppErrors::ConfigurationError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
