use crate::app_state::AppState;
use crate::data_models::{Product, Shop};
use crate::db::{DatabaseProduct, SearchFilter};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Message {
    pub name: String,
    pub email: String,
    pub message: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ProductAlert {
    pub product_id: u32,
    pub price_below: f32,
    pub shop: Option<Shop>,
}

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn products(State(state): State<AppState>) -> Result<Json<Vec<DatabaseProduct>>> {
    let products = state.db.all_products();
    Ok(Json(products))
}

pub async fn product(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<Product>, StatusCode> {
    let product = state
        .db
        .get_product_by(id.to_owned())
        .ok_or(StatusCode::NOT_FOUND)?;
    let listings = state.db.get_positions_for(&product);
    let prices = state.db.get_prices_for(&product);

    let mut shop_with_positions = HashMap::new();
    for listing in &listings {
        shop_with_positions
            .entry(listing.shop.clone())
            .or_insert(vec![listing.into()])
            .push(listing.into());
    }
    let mut prices: Vec<(String, f32)> = prices
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    prices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let final_product = Product {
        name: product.name,
        id,
        listings: shop_with_positions,
        history_prices: prices,
    };
    Ok(Json(final_product))
}

pub async fn n_products(State(state): State<AppState>) -> Result<Json<usize>> {
    let products = state.db.all_products();
    Ok(Json(products.len()))
}

pub async fn search(
    State(state): State<AppState>,
    Json(_query): Json<SearchFilter>,
) -> Result<Json<Vec<DatabaseProduct>>> {
    let products = state.db.search_with_filter(_query);
    Ok(Json(products))
}

pub async fn search_filter(State(state): State<AppState>) -> Result<Json<SearchFilter>> {
    let filter = state.db.get_search_filter();
    Ok(Json(filter))
}

pub async fn contact(State(_state): State<AppState>, Json(_msg): Json<Message>) -> StatusCode {
    // TODO How to alert about new messages?
    StatusCode::OK
}

pub async fn alert(State(_state): State<AppState>, Json(_alert): Json<ProductAlert>) -> StatusCode {
    // TODO How to save alerts?
    StatusCode::OK
}
