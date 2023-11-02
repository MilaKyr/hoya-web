mod app_state;
pub mod configuration;
pub mod errors;
mod templates;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::get;
use axum::routing::post;
use axum::{Form, Router};
use serde::Deserialize;
use tower_http::services::ServeDir;

use crate::app_state::AppState;
use crate::errors::Error;
use crate::templates::{HomeTemplate, HoyaPageTemplate, HtmlTemplate};

#[derive(Debug, Deserialize)]
struct Message {
    name: String,
    email: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct Search {
    text: String,
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

async fn index() -> impl IntoResponse {
    let template = HomeTemplate::default();
    HtmlTemplate(template)
}

async fn contact() -> impl IntoResponse {
    match std::fs::read_to_string("templates/contact.html") {
        Ok(content) => Html(content),
        Err(e) => Html(format!("Error : {}", e)),
    }
}

async fn send_message(Form(message): Form<Message>) -> impl IntoResponse {
    println!(
        "message {:?} {:?} {:?}",
        message.name, message.email, message.message
    );
    Redirect::to("/")
}

async fn hoya_page(Path(_hoya_id): Path<u32>) -> impl IntoResponse {
    let template = HoyaPageTemplate {
        name: "Placeholder name".to_string(),
        desc: "Placeholder desc".to_string(),
    };
    HtmlTemplate(template)
}

async fn search(Form(text): Form<Search>) -> impl IntoResponse {
    let response = format!("<p>You want to search for {} </p>", text.text);
    Html(response)
}

async fn search_all() -> impl IntoResponse {
    Html(r#"Here would be list f all available hoyas!"#)
}

pub fn create_app() -> Result<Router, Error> {
    let app_state = AppState::init();
    let app = Router::new()
        .nest_service("/public", ServeDir::new("public"))
        .route("/", get(index))
        .route("/hoya/:hoya_id", get(hoya_page))
        .route("/send_message", post(send_message))
        .route("/contact", get(contact))
        .route("/health_check", get(health_check))
        .route("/search", post(search))
        .route("/search_all", get(search_all))
        .with_state(app_state);
    Ok(app)
}
