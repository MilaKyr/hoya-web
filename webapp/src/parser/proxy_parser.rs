use crate::data_models::{Proxy, ProxyParsingRules};
use crate::db::Database;
use crate::parser::errors::ParserError;
use scraper::{Html, Selector};
use tokio::task::spawn_blocking;
use url::Url;

#[derive(Debug, Default, Clone)]
pub struct ProxyManager {}

impl ProxyManager {
    pub async fn update(&self, db: &Database) -> Result<(), ParserError> {
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
        db.save_proxies(proxies?);
        Ok(())
    }

    pub fn parse_proxy(
        url: Url,
        rules: ProxyParsingRules,
        result: &mut Vec<Proxy>,
    ) -> Result<(), ParserError> {
        let text = reqwest::blocking::get(url)?.text()?;
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
        let head_elements_selected = table.select(&head_elements_selector);
        for head_element in head_elements_selected {
            let mut element = head_element.text().collect::<Vec<_>>().join(" ");
            element = element.trim().replace('\n', " ");
            head.push(element);
        }
        let mut rows: Vec<Vec<String>> = vec![];
        let row_elements = table.select(&row_elements_selector);
        for row_element in row_elements {
            let mut row = vec![];
            for td_element in row_element.select(&row_element_data_selector) {
                let mut element = td_element.text().collect::<Vec<_>>().join(" ");
                element = element.trim().replace('\n', " ");
                row.push(element);
            }
            if !row.is_empty() {
                rows.push(row);
            }
        }
        for row in rows {
            let zipped_array = head.iter().zip(row.iter()).collect::<Vec<_>>();
            result.push(Proxy::from_row(zipped_array));
        }
        Ok(())
    }
}
