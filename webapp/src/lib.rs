pub mod app_state;
pub mod configuration;
pub mod data_models;
pub mod db;
pub mod errors;
mod parser;
mod routes;

use crate::app_state::AppState;
use crate::db::Database;
use crate::errors::AppErrors;
use axum::routing::{get, post};
use axum::Router;

pub fn create_app(db: Database) -> Result<(Router, AppState), AppErrors> {
    let app_state = AppState::init(db);
    let app = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/products", get(routes::products))
        .route("/product/:id", get(routes::product))
        .route("/n_products", get(routes::n_products))
        .route("/search", post(routes::search))
        .route("/search_filter", get(routes::search_filter))
        .route("/contact", post(routes::contact))
        .route("/alert", post(routes::alert))
        .with_state(app_state.clone());
    Ok((app, app_state))
}
