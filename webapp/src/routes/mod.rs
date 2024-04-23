use crate::app_state::AppState;
use crate::data_models::Product;
use crate::db::{DatabaseProduct, Message, ProductAlert, SearchFilter};
use crate::errors::AppErrors;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Result};
use std::collections::HashMap;
use validator::Validate;

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn products(
    State(state): State<AppState>,
) -> Result<Json<Vec<DatabaseProduct>>, AppErrors> {
    let products = state.db.all_products().await?;
    Ok(Json(products))
}

pub async fn product(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<Product>, AppErrors> {
    let product = state.db.get_product_by(id.to_owned()).await?;
    let listings = state.db.get_positions_for(&product).await?;
    let prices = state.db.get_prices_for(&product).await?;

    let mut shop_with_positions = HashMap::new();
    for listing in &listings {
        shop_with_positions
            .entry(listing.shop.clone())
            .or_insert(vec![listing.into()])
            .push(listing.into());
    }

    let final_product = Product {
        name: product.name,
        id,
        listings: shop_with_positions,
        history_prices: prices,
    };
    Ok(Json(final_product))
}

pub async fn n_products(State(state): State<AppState>) -> Result<Json<usize>, AppErrors> {
    let products = state.db.all_products().await?;
    Ok(Json(products.len()))
}

pub async fn search(
    State(state): State<AppState>,
    Json(query): Json<SearchFilter>,
) -> Result<Json<Vec<DatabaseProduct>>, AppErrors> {
    query.validate()?;
    let products = state.db.search_with_filter(query).await?;
    Ok(Json(products))
}

pub async fn search_filter(State(state): State<AppState>) -> Result<Json<SearchFilter>, AppErrors> {
    let filter = state.db.get_search_filter().await?;
    Ok(Json(filter))
}

pub async fn contact(
    State(state): State<AppState>,
    Json(msg): Json<Message>,
) -> Result<StatusCode, AppErrors> {
    msg.validate()?;
    state.db.register_message(msg).await?;
    Ok(StatusCode::OK)
}

pub async fn alert(
    State(state): State<AppState>,
    Json(alert): Json<ProductAlert>,
) -> Result<StatusCode, AppErrors> {
    alert.validate()?;
    state.db.register_alert(alert).await?;
    Ok(StatusCode::OK)
}
