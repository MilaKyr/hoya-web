use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

#[derive(Default, Template)]
#[template(path = "index.html")]
pub struct HomeTemplate {
    pub n_products: u32,
    pub n_shops: u32,
}

#[derive(Default, Template)]
#[template(path = "hoya_page.html")]
pub struct HoyaPageTemplate {
    pub name: String,
    pub desc: String,
}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
