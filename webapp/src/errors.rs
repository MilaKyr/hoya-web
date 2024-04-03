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
pub enum Error {
    #[error("failed to read with serde: {0}")]
    SerdeError(#[from] serde_json::error::Error),
    #[error("socket address parsing error: {0}")]
    SocketAddressParsingError(#[from] std::net::AddrParseError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    AppError(AppErrors),
}

#[derive(Error, Debug)]
pub enum AppErrors {
    #[error("failed parsing with: {0}")]
    ParserError(#[from] ParserError),
    #[error("failed to parse string as url: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("")]
    UnknownProduct,
}

impl IntoResponse for AppErrors {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppErrors::UnknownProduct => (StatusCode::BAD_REQUEST, "".to_string()),
            AppErrors::UrlParseError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s.to_string()),
            AppErrors::ParserError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        if let Error::AppError(err) = self {
            return err.into_response();
        }
        let (status, error_message) = match self {
            Error::SerdeError(s) => (StatusCode::BAD_REQUEST, s.to_string()),
            Error::SocketAddressParsingError(s) => {
                (StatusCode::INTERNAL_SERVER_ERROR, s.to_string())
            }
            Error::IoError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s.to_string()),
            Error::AppError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
