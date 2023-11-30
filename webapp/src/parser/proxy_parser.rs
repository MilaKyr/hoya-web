use crate::data_models::{Proxy, ProxyParsingRules};
use crate::db::Database;
use crate::parser::errors::ParserError;
use crate::parser::traits::Parser;
use reqwest::redirect::Policy;
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use tokio::task::spawn_blocking;
use url::Url;

const CHECK_BY_URL: &str = "http://www.google.com";

#[derive(Debug, Default, Clone)]
pub struct ProxyManager {}

impl Parser for ProxyManager {}

impl ProxyManager {
    pub async fn get(&self, db: &Database) -> Result<Proxy, ParserError> {
        let proxy_parsing_rules = db.get_proxy_parsing_rules();
        let proxies: Result<Vec<Proxy>, ParserError> = spawn_blocking(move || {
            let mut proxies = vec![];
            for (proxy_source, parsing_rules) in proxy_parsing_rules.into_iter() {
                Self::parse_proxy(proxy_source, parsing_rules, &mut proxies)?;
            }
            Ok(proxies)
        })
        .await
        .map_err(|_| ParserError::FailedToUpdateProxies)?;
        self.check_proxies(&proxies?)
            .await
            .ok_or(ParserError::NoProxyAvailable)
    }

    pub fn parse_proxy(
        url: Url,
        rules: ProxyParsingRules,
        result: &mut Vec<Proxy>,
    ) -> Result<(), ParserError> {
        let text = Self::create_client(None)?.get(url).send()?.text()?;
        let document = Html::parse_document(&text);
        let table_selector =
            Selector::parse(&rules.table_lookup).map_err(|_| ParserError::CrawlerSelectorError)?;
        let head_elements_selector =
            Selector::parse(&rules.head_lookup).map_err(|_| ParserError::CrawlerSelectorError)?;
        let row_elements_selector =
            Selector::parse(&rules.row_lookup).map_err(|_| ParserError::CrawlerSelectorError)?;
        let row_element_data_selector =
            Selector::parse(&rules.data_lookup).map_err(|_| ParserError::CrawlerSelectorError)?;
        let mut selected_table = document.select(&table_selector);
        let table = selected_table
            .next()
            .ok_or(ParserError::FailedToFindProxyTable)?;
        let mut head: Vec<String> = vec![];
        for head_element in table.select(&head_elements_selector) {
            let element = Self::clean_data_point(head_element);
            head.push(element);
        }
        let mut rows: Vec<Vec<String>> = vec![];
        for row_element in table.select(&row_elements_selector) {
            let mut row = vec![];
            for td_element in row_element.select(&row_element_data_selector) {
                let element = Self::clean_data_point(td_element);
                row.push(element);
            }
            if !row.is_empty() {
                rows.push(row);
            }
        }
        for row in rows {
            let zipped_array = head.iter().zip(row.iter()).collect::<Vec<_>>();
            result.push(Proxy::try_from(zipped_array)?);
        }
        Ok(())
    }

    async fn check_proxies(&self, proxies: &[Proxy]) -> Option<Proxy> {
        for proxy in proxies.iter() {
            if self.check_proxy(proxy).await.is_ok() {
                return Some(proxy.clone());
            }
        }
        None
    }

    async fn check_proxy(&self, proxy: &Proxy) -> Result<(), ParserError> {
        let url_proxy = Url::parse(&proxy.to_string())?;
        let client = Client::builder()
            .redirect(Policy::limited(30))
            .timeout(Duration::from_secs(1))
            .proxy(reqwest::Proxy::all(url_proxy)?)
            .build()
            .map_err(ParserError::FailedClient)?;
        let _ = client.get(CHECK_BY_URL).send().await?;
        Ok(())
    }
}
