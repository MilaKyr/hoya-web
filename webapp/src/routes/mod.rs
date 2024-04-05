use crate::app_state::AppState;
use crate::data_models::Product;
use crate::db::Shop;
use crate::db::{DatabaseProduct, SearchFilter};
use crate::errors::{AppErrors, Error};
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

pub async fn products(State(state): State<AppState>) -> Result<Json<Vec<DatabaseProduct>>, Error> {
    let products = state
        .db
        .all_products()
        .await
        .map_err(|e| Error::AppError(AppErrors::DatabaseError(e)))?;
    Ok(Json(products))
}

pub async fn product(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<Product>, Error> {
    let product = state
        .db
        .get_product_by(id.to_owned())
        .await
        .map_err(|_| Error::AppError(AppErrors::UnknownProduct))?;
    let listings = state
        .db
        .get_positions_for(&product)
        .await
        .map_err(|e| Error::AppError(AppErrors::DatabaseError(e)))?;
    let prices = state
        .db
        .get_prices_for(&product)
        .await
        .map_err(|e| Error::AppError(AppErrors::DatabaseError(e)))?;

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

pub async fn n_products(State(state): State<AppState>) -> Result<Json<usize>, Error> {
    let products = state
        .db
        .all_products()
        .await
        .map_err(|e| Error::AppError(AppErrors::DatabaseError(e)))?;
    Ok(Json(products.len()))
}

pub async fn search(
    State(state): State<AppState>,
    Json(_query): Json<SearchFilter>,
) -> Result<Json<Vec<DatabaseProduct>>, Error> {
    let products = state
        .db
        .search_with_filter(_query)
        .await
        .map_err(|e| Error::AppError(AppErrors::DatabaseError(e)))?;
    Ok(Json(products))
}

pub async fn search_filter(State(state): State<AppState>) -> Result<Json<SearchFilter>> {
    let filter = state.db.get_search_filter().await;
    Ok(Json(filter))
}

pub async fn contact(State(_state): State<AppState>, Json(_msg): Json<Message>) -> StatusCode {
    todo!()
}

pub async fn alert(State(_state): State<AppState>, Json(_alert): Json<ProductAlert>) -> StatusCode {
    todo!()
}
