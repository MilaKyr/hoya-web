pub mod app_state;
pub mod configuration;
mod data_models;
mod db;
pub mod errors;
mod parser;
mod templates;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{ErrorResponse, IntoResponse, Redirect};
use axum::routing::get;
use axum::routing::post;
use axum::{Form, Router};
use serde::Deserialize;
use tower_http::services::ServeDir;

use crate::app_state::AppState;
use crate::errors::Error;
use crate::templates::{
    AboutTemplate, ContactTemplate, HomeTemplate, HoyaPageTemplate, HtmlTemplate,
    LicensingTemplate, ListTemplate, NotFoundTemplate, PrivacyPolicyTemplate,
};

#[derive(Debug, Deserialize)]
struct Message {
    name: String,
    email: String,
    message: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Search {
    text: String,
}

async fn handler_404() -> impl IntoResponse {
    let template = NotFoundTemplate::default();
    HtmlTemplate(template)
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

async fn index() -> impl IntoResponse {
    let template = HomeTemplate::default();
    HtmlTemplate(template)
}

async fn contact() -> impl IntoResponse {
    let template = ContactTemplate::default();
    HtmlTemplate(template)
}

async fn send_message(Form(message): Form<Message>) -> impl IntoResponse {
    println!(
        "message {:?} {:?} {:?}",
        message.name, message.email, message.message
    );
    Redirect::to("/")
}

async fn hoya_page(Path(_hoya_id): Path<u32>) -> impl IntoResponse {
    let template = HoyaPageTemplate::dummy();
    HtmlTemplate(template)
}

async fn about() -> impl IntoResponse {
    let template = AboutTemplate::default();
    HtmlTemplate(template)
}

async fn privacy_policy() -> impl IntoResponse {
    let template = PrivacyPolicyTemplate::default();
    HtmlTemplate(template)
}

async fn licensing() -> impl IntoResponse {
    let template = LicensingTemplate::default();
    HtmlTemplate(template)
}

async fn search(Form(_text): Form<Search>) -> impl IntoResponse {
    let template = ListTemplate::dummy();
    HtmlTemplate(template)
}

async fn search_all() -> impl IntoResponse {
    let template = ListTemplate::dummy();
    HtmlTemplate(template)
}

pub async fn parse(app_state: State<AppState>) -> axum::response::Result<StatusCode> {
    match app_state.parse().await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR)),
    }
}

pub fn create_app() -> Result<(Router, AppState), Error> {
    let app_state = AppState::init();
    let app = Router::new()
        .nest_service("/public", ServeDir::new("public"))
        .route("/", get(index))
        .route("/hoya/:hoya_id", get(hoya_page))
        .route("/send_message", post(send_message))
        .route("/health_check", get(health_check))
        .route("/search", post(search))
        .route("/search_all", get(search_all))
        .route("/contact", get(contact))
        .route("/about", get(about))
        .route("/licensing", get(licensing))
        .route("/privacy_policy", get(privacy_policy))
        .route("/parse", get(parse))
        .with_state(app_state.clone())
        .fallback(handler_404);
    Ok((app, app_state))
}
