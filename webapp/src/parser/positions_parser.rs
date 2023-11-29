use crate::data_models::{HoyaPosition, Proxy, Shop, ShopParsingRules};
use crate::db::Database;
use crate::parser::errors::ParserError;
use rand::seq::SliceRandom;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use scraper::{ElementRef, Html, Selector};
use std::str::FromStr;
use std::time::Duration;
use tokio::task::spawn_blocking;
use url::Url;

fn clean_price(price: String) -> f32 {
    price
        .replace("\u{a0}€", "")
        .replace("€\u{a0}", "")
        .replace('€', "")
        .replace(',', ".")
        .parse::<f32>()
        .unwrap_or(-10.99)
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Parser {}

impl Parser {
    pub fn find_proxy(proxies: &mut Vec<Proxy>) -> Result<Proxy, ParserError> {
        let mut rng = rand::thread_rng();
        proxies.shuffle(&mut rng);
        match proxies.pop() {
            Some(proxy) => Ok(proxy),
            None => Err(ParserError::NoProxyAvailable),
        }
    }

    pub async fn parse(&self, db: &Database) -> Result<(Shop, Vec<HoyaPosition>), ParserError> {
        let shop = db.get_top_shop().ok_or(ParserError::NoShopsFound)?;
        let proxies = db.get_proxies();
        let shop_rules = db
            .get_shop_parsing_rules(&shop)
            .ok_or(ParserError::FailedToFindShopsRules(shop.name.to_string()))?;
        let positions = self.parse_shop(shop.clone(), shop_rules, proxies).await?;
        Ok((shop, positions))
    }

    pub async fn parse_shop(
        &self,
        shop: Shop,
        shop_rules: ShopParsingRules,
        proxies: Vec<Proxy>,
    ) -> Result<Vec<HoyaPosition>, ParserError> {
        let task: tokio::task::JoinHandle<Result<Vec<HoyaPosition>, ParserError>> =
            spawn_blocking(move || {
                let mut proxies = proxies.clone();
                if shop_rules.url_categories.is_empty() {
                    return Self::parse_shop_no_categories(&shop, &mut proxies, &shop_rules);
                }
                Self::parse_shop_with_categories(&shop, &mut proxies, &shop_rules)
            });

        // match task.await? {
        //     Ok(p) => {
        //         println!("positions to be saved: {:?}", p);
        //         db.save_hoya_positions(p);
        //     }
        //     Err(err) => {
        //         println!("{:?}", err)
        //     }
        // };
        // Ok(())
        task.await?
    }

    pub fn create_client(proxy: Proxy) -> Result<Client, ParserError> {
        let url_proxy = Url::from_str(&proxy.to_string())?;
        let client = reqwest::blocking::Client::builder()
            .redirect(Policy::limited(30))
            .user_agent("User-Agent Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0")
            .proxy(reqwest::Proxy::http(url_proxy.clone()).unwrap())
            .build()?;
        Ok(client)
    }

    pub fn parse_first_page(
        shop: &Shop,
        proxies: &mut Vec<Proxy>,
        shop_rules: &ShopParsingRules,
        category: Option<String>,
    ) -> Result<(Vec<HoyaPosition>, u32), ParserError> {
        let proxy = Self::find_proxy(proxies)?;
        let client = Self::create_client(proxy)?;
        let parsing_url = shop_rules.get_shop_parsing_url(1, &category);
        let response_text = client.get(parsing_url).send()?.text()?;
        let document = Html::parse_document(&response_text);
        let n_pages = Self::retrieve_page_count(shop_rules, &document)?;
        let positions = Self::parse_data(shop, shop_rules, &document)?;
        Ok((positions, n_pages))
    }

    pub fn parse_page(
        shop: &Shop,
        proxies: &mut Vec<Proxy>,
        shop_rules: &ShopParsingRules,
        page_id: u32,
        category: Option<String>,
    ) -> Result<Vec<HoyaPosition>, ParserError> {
        let proxy = Self::find_proxy(proxies)?;
        let client = Self::create_client(proxy)?;
        let parsing_url = shop_rules.get_shop_parsing_url(page_id, &category);
        let response_text = client.get(parsing_url).send()?.text()?;
        let document = Html::parse_document(&response_text);
        let positions = Self::parse_data(shop, shop_rules, &document)?;
        Ok(positions)
    }

    pub fn parse_shop_no_categories(
        shop: &Shop,
        proxies: &mut Vec<Proxy>,
        shop_rules: &ShopParsingRules,
    ) -> Result<Vec<HoyaPosition>, ParserError> {
        let mut all_positions = vec![];
        let (page_positions, n_pages) = Self::parse_first_page(shop, proxies, shop_rules, None)?;
        all_positions.extend(page_positions);

        if let Some(duration_to_sleep) = shop_rules.sleep_timeout_sec {
            std::thread::sleep(Duration::from_secs(duration_to_sleep));
        }

        for page_id in 2..=n_pages {
            let page_positions = Self::parse_page(shop, proxies, shop_rules, page_id, None)?;
            all_positions.extend(page_positions);

            if let Some(duration_to_sleep) = shop_rules.sleep_timeout_sec {
                std::thread::sleep(Duration::from_secs(duration_to_sleep));
            }
        }
        Ok(all_positions)
    }

    pub fn parse_shop_with_categories(
        shop: &Shop,
        proxies: &mut Vec<Proxy>,
        shop_rules: &ShopParsingRules,
    ) -> Result<Vec<HoyaPosition>, ParserError> {
        let mut all_positions = vec![];
        for category in shop_rules.url_categories.iter() {
            let (page_positions, n_pages) =
                Self::parse_first_page(shop, proxies, shop_rules, Some(category.clone()))?;
            all_positions.extend(page_positions);

            if let Some(duration_to_sleep) = shop_rules.sleep_timeout_sec {
                std::thread::sleep(
                    Duration::try_from(time::Duration::seconds(duration_to_sleep as i64)).unwrap(),
                );
            }

            for page_id in 2..=n_pages {
                let page_positions =
                    Self::parse_page(shop, proxies, shop_rules, page_id, Some(category.clone()))?;
                all_positions.extend(page_positions);

                if let Some(duration_to_sleep) = shop_rules.sleep_timeout_sec {
                    std::thread::sleep(
                        Duration::try_from(time::Duration::seconds(duration_to_sleep as i64))
                            .unwrap(),
                    );
                }
            }
        }
        Ok(all_positions)
    }

    fn parse_data(
        shop: &Shop,
        shop_rules: &ShopParsingRules,
        document: &Html,
    ) -> Result<Vec<HoyaPosition>, ParserError> {
        let mut products = vec![];
        let table_selector = Selector::parse(&shop_rules.product_table_lookup).unwrap();
        let prod_selector = Selector::parse(&shop_rules.product_lookup).unwrap();
        let mut selected = document.select(&table_selector);
        if let Some(table) = selected.next() {
            for product in table.select(&prod_selector) {
                match Self::parse_product(shop, shop_rules, product) {
                    Ok(hoya_position) => products.push(hoya_position),
                    Err(err) => println!("Error while parsing a product {:?}", err),
                };
            }
        }
        Ok(products)
    }

    pub fn parse_product(
        shop: &Shop,
        shop_rules: &ShopParsingRules,
        product: ElementRef,
    ) -> Result<HoyaPosition, ParserError> {
        let name = {
            let name_selector = Selector::parse(&shop_rules.name_lookup)
                .map_err(|_| ParserError::CrawlerSelectorError)?;
            product
                .select(&name_selector)
                .next()
                .map(|name| name.text().collect::<String>())
                .ok_or(ParserError::CrawlerSelectorError)?
        };
        let price = {
            let price_selector = Selector::parse(&shop_rules.price_lookup)
                .map_err(|_| ParserError::CrawlerSelectorError)?;

            let price = product
                .select(&price_selector)
                .next()
                .map(|price| price.text().collect::<String>())
                .ok_or(ParserError::CrawlerSelectorError)?;
            clean_price(price)
        };
        let url = {
            let url_selector = Selector::parse(&shop_rules.url_lookup)
                .map_err(|_| ParserError::CrawlerSelectorError)?;
            product
                .select(&url_selector)
                .next()
                .map(|url_elem| {
                    if shop_rules.look_for_href {
                        let url = url_elem.attr("href").unwrap();
                        return url.to_string();
                    }
                    url_elem.text().collect::<String>()
                })
                .ok_or(ParserError::CrawlerSelectorError)?
        };
        Ok(HoyaPosition::new(shop.clone(), name, price, url))
    }

    pub fn retrieve_page_count(
        shop_rules: &ShopParsingRules,
        document: &Html,
    ) -> Result<u32, ParserError> {
        let mut max = 0;
        if let Ok(selector) = Selector::parse(&shop_rules.max_page_lookup) {
            for element in document.select(&selector) {
                let num: String = element
                    .text()
                    .map(|s| String::from_str(s).unwrap())
                    .collect();
                if let Ok(number) = num.parse::<u32>() {
                    if max < number {
                        max = number;
                    }
                }
            }
        }
        Ok(max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_shop() -> Shop {
        Shop {
            logo_path: "path/to/file".to_string(),
            name: "test name".to_string(),
        }
    }

    #[test]
    fn clean_price_comma_endeuro_works() {
        let price = clean_price("10,11\u{a0}€".to_string());
        assert!((price - 10.11 < f32::EPSILON));
    }

    #[test]
    fn clean_price_point_endeuro_works() {
        let price = clean_price("10.11€".to_string());
        assert!((price - 10.11 < f32::EPSILON));
    }

    #[test]
    fn clean_price_point_endspaceeuro_works() {
        let price = clean_price("10.11\u{a0}€".to_string());
        assert!((price - 10.11 < f32::EPSILON));
    }

    #[test]
    fn clean_price_comma_starteuro_works() {
        let price = clean_price("€10,11".to_string());
        assert!(price - 10.11 < f32::EPSILON);
    }

    #[test]
    fn clean_price_point_starteuro_works() {
        let price = clean_price("€10.11".to_string());
        assert!(price - 10.11 < f32::EPSILON);
    }

    #[test]
    fn clean_price_point_startspaceeuro_works() {
        let price = clean_price("€\u{a0}10.11".to_string());
        assert!(price - 10.11 < f32::EPSILON);
    }

    #[test]
    fn find_proxy_works() {
        let org_proxies: Vec<_> = vec![
            Proxy::dummy("a"),
            Proxy::dummy("b"),
            Proxy::dummy("c"),
            Proxy::dummy("d"),
        ];
        let mut proxies = org_proxies.clone();
        let mut seen = vec![];
        for _ in 0..proxies.len() {
            let parser = Parser::find_proxy(&mut proxies).expect(&format!("Failed to find proxy"));
            seen.push(parser);
        }
        assert!(proxies.is_empty());
        assert_eq!(seen.len(), 4);
        for proxy in org_proxies.iter() {
            assert!(seen.contains(proxy));
        }
    }

    #[test]
    fn find_proxy_fails() {
        let mut proxies: Vec<_> = vec![];
        let res = Parser::find_proxy(&mut proxies);
        assert!(res.is_err());
    }

    #[test]
    fn parse_max_page_works() {
        let html = Html::parse_document(
            r#"
        <!DOCTYPE html>
        <html lang="en">
          <head><title></title></head>
          <body>
            <ul class=\"pagination\">
                    <li>1</li>
                    <li>2</li>
                    <li>>></li>
                    <li> ... </li>
                    <li>3</li>
             </ul>
          </body>
        </html>
        "#,
        );

        let shop_rules = ShopParsingRules {
            max_page_lookup: "li".to_string(),
            ..Default::default()
        };
        let max_page = Parser::retrieve_page_count(&shop_rules, &html);
        assert!(max_page.is_ok());
        assert_eq!(max_page.unwrap(), 3);
    }

    #[test]
    fn parse_product_href_works() {
        let shop = create_test_shop();
        let shop_rules = ShopParsingRules {
            product_lookup: "div.products > div.product".to_string(),
            name_lookup: "span.product_name".to_string(),
            price_lookup: "div.price".to_string(),
            url_lookup: "a".to_string(),
            look_for_href: true,
            ..Default::default()
        };

        let html = Html::parse_document(
            r#"
        <!DOCTYPE html>
        <html lang="en">
          <head><title></title></head>
          <body>
          <div class="products">
            <div class="product">
                <h2>Title</h2>
                <span class="product_name">Test name</span>
                <span class="importer">Test importer</span>
                <div class="price">14,11</div>
                <a href="https://example.com">Name</a>
            </div>
           </div>
        </body>
        </html>
        "#,
        );
        let element = html
            .select(&Selector::parse(&shop_rules.product_lookup).unwrap())
            .next();
        assert!(element.is_some());
        let result = Parser::parse_product(&shop, &shop_rules, element.unwrap());
        let expected_position = HoyaPosition::new(
            shop.clone(),
            "Test name".to_string(),
            14.11,
            "https://example.com".to_string(),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_position);
    }

    #[test]
    fn parse_product_div_works() {
        let shop = create_test_shop();
        let shop_rules = ShopParsingRules {
            product_lookup: "div.products > div.product".to_string(),
            name_lookup: "span.product_name".to_string(),
            price_lookup: "div.price".to_string(),
            url_lookup: "div.url".to_string(),
            ..Default::default()
        };

        let html = Html::parse_document(
            r#"
        <!DOCTYPE html>
        <html lang="en">
          <head><title></title></head>
          <body>
          <div class="products">
            <div class="product">
                <h2>Title</h2>
                <span class="product_name">Test name</span>
                <span class="importer">Test importer</span>
                <div class="price">14,11</div>
                <div class="url">https://example.com</div>
            </div>
           </div>
        </body>
        </html>
        "#,
        );
        let element = html
            .select(&Selector::parse(&shop_rules.product_lookup).unwrap())
            .next();
        assert!(element.is_some());
        let result = Parser::parse_product(&shop, &shop_rules, element.unwrap());
        let expected_position = HoyaPosition::new(
            shop.clone(),
            "Test name".to_string(),
            14.11,
            "https://example.com".to_string(),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_position);
    }

    #[test]
    fn parse_data_works() {
        let shop = create_test_shop();
        let shop_rules = ShopParsingRules {
            product_table_lookup: "div.products".to_string(),
            product_lookup: "div.products > div.product".to_string(),
            name_lookup: "span.product_name".to_string(),
            price_lookup: "div.price".to_string(),
            url_lookup: "div.url".to_string(),
            ..Default::default()
        };
        let html = Html::parse_document(
            r#"
        <!DOCTYPE html>
        <html lang="en">
          <head><title></title></head>
          <body>
          <div class="products">
            <div class="product">
                <h2>Title</h2>
                <span class="product_name">Test name</span>
                <span class="importer">Test importer</span>
                <div class="price">14,11</div>
                <div class="url">https://example.com</div>
            </div>
           </div>
        </body>
        </html>
        "#,
        );
        let result = Parser::parse_data(&shop, &shop_rules, &html);
        let expected_position = vec![HoyaPosition::new(
            shop.clone(),
            "Test name".to_string(),
            14.11,
            "https://example.com".to_string(),
        )];
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_position);
    }
}
