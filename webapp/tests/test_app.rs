use std::env;
use axum::{
    body,
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use webapp::configuration::{DatabaseSettings};
use webapp::create_app;
use webapp::data_models::Product;
use webapp::db::Database;

pub async fn read_body(body: Body) -> String {
    let bytes = body::to_bytes(body, usize::MAX).await.expect("Failed");
    String::from_utf8(bytes.to_vec()).expect("response was not valid utf-8")
}

async fn create_db() -> Database {
    let directory = env::current_dir().expect("Failed to find current directory");
    let settings = DatabaseSettings {
        file_path: Some(format!("{}/tests/data.json", directory.to_str().unwrap())),
        ..Default::default()
    };
    Database::try_from(&settings).await
        .expect("Failed to create in memory db")
}

#[tokio::test]
async fn health_check_works() {
    let db = create_db().await;
    let (app, _) = create_app(db).expect("Failed to create an app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health_check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn products_works() {
    let db = create_db().await;
    let (app, _) = create_app(db).expect("Failed to create an app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/products")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let (parts, body) = response.into_parts();
    let text = read_body(body).await;
    assert_eq!(parts.status, StatusCode::OK);
    assert!(serde_json::from_str::<Vec<Product>>(&text)
        .expect("Failed to convert string to vec")
        .is_empty());
}

#[tokio::test]
async fn n_product_works() {
    let db = create_db().await;
    let (app, _) = create_app(db).expect("Failed to create an app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/n_products")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let (parts, body) = response.into_parts();
    let text = read_body(body).await;
    assert_eq!(parts.status, StatusCode::OK);
    assert_eq!(text.parse::<i32>().expect("Failed to parse to integer"), 0);
}

#[tokio::test]
async fn product_id_fails() {
    let db = create_db().await;
    let (app, _) = create_app(db).expect("Failed to create an app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/product/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
