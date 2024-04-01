pub mod app_state;
pub mod configuration;
pub mod data_models;
mod db;
pub mod errors;
mod parser;
mod routes;

use axum::routing::get;
use axum::{Router};
use crate::app_state::AppState;
use crate::errors::Error;



pub fn create_app() -> Result<(Router, AppState), Error> {
    let app_state = AppState::init();
    let app = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/products", get(routes::products))
        .route("/product/:id", get(routes::product))
        .route("/n_products", get(routes::n_products))
        .with_state(app_state.clone());
    Ok((app, app_state))
}
