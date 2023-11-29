use crate::data_models::{Product, ShopListing};
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
    pub listings: Vec<ShopListing>,
}

impl HoyaPageTemplate {
    pub fn dummy() -> Self {
        let mut listings = vec![];
        for _ in 0..5 {
            listings.push(ShopListing::dummy())
        }
        Self {
            name: "Some name here".to_string(),
            desc: "Some description here".to_string(),
            listings,
        }
    }
}

#[derive(Default, Template)]
#[template(path = "list.html")]
pub struct ListTemplate {
    pub products: Vec<Product>,
}

impl ListTemplate {
    pub fn dummy() -> Self {
        let mut products = vec![];
        for _ in 0..5 {
            products.push(Product::dummy())
        }
        Self { products }
    }
}

#[derive(Default, Template)]
#[template(path = "contact.html")]
pub struct ContactTemplate {}

#[derive(Default, Template)]
#[template(path = "privacy_policy.html")]
pub struct PrivacyPolicyTemplate {}

#[derive(Default, Template)]
#[template(path = "licensing.html")]
pub struct LicensingTemplate {}

#[derive(Default, Template)]
#[template(path = "about.html")]
pub struct AboutTemplate {}

#[derive(Default, Template)]
#[template(path = "not_found.html")]
pub struct NotFoundTemplate {}

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
