use axum::{
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{self, Request, StatusCode},
};
use tower::ServiceExt;
use webapp::create_app;

#[tokio::test]
async fn health_check_works() {
    let app = create_app().expect("Failed to create an app");

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
async fn index_page_works() {
    let app = create_app().expect("Failed to create an app");

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn hoya_page_works() {
    let app = create_app().expect("Failed to create an app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/hoya/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn contact_page_works() {
    let app = create_app().expect("Failed to create an app");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/contact")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn unk_page_works() {
    let app = create_app().expect("Failed to create an app");

    let response = app
        .oneshot(Request::builder().uri("/unk").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
