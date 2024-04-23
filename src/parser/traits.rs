use crate::db::Proxy;
use crate::parser::errors::ParserError;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use scraper::{ElementRef, Selector};
use std::str::FromStr;
use url::Url;

pub trait Parser {
    fn clean_data_point(input: ElementRef) -> String {
        let element = input.text().collect::<Vec<_>>().join(" ");
        element.trim().replace('\n', " ")
    }

    fn create_client(proxy: Option<Proxy>) -> Result<Client, ParserError> {
        let mut client = Client::builder()
            .redirect(Policy::limited(30))
            .user_agent("User-Agent Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0");
        if let Some(proxy) = proxy {
            let url_proxy = Url::from_str(&proxy.to_string())?;
            client = client.proxy(reqwest::Proxy::http(url_proxy)?)
        }
        client.build().map_err(ParserError::FailedClient)
    }

    fn select_data_point(
        element: ElementRef,
        selector_name: &str,
        look_for_href: bool,
    ) -> Result<String, ParserError> {
        let selector =
            Selector::parse(selector_name).map_err(|_| ParserError::CrawlerSelectorError)?;
        element
            .select(&selector)
            .next()
            .map(|elem| {
                if look_for_href {
                    let data = elem.attr("href").unwrap();
                    return data.to_string();
                }
                Self::clean_data_point(elem)
            })
            .ok_or(ParserError::CrawlerSelectorError)
    }
}
