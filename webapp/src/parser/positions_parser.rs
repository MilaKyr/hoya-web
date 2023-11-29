use crate::data_models::{HoyaPosition, Proxy, Shop, ShopParsingRules};
use crate::db::Database;
use crate::parser::errors::ParserError;
use crate::parser::traits::Parser;
use rand::seq::SliceRandom;
use scraper::{ElementRef, Html, Selector};
use tokio::task::spawn_blocking;

const REMOVE_WORDS: [&str; 3] = ["\u{a0}€", "€\u{a0}", "€"];
const REPLACE_WORDS: [(&str, &str); 1] = [(",", ".")];
const PRICE_DEFAULT: f32 = -9.99;

fn clean_price(price: String) -> f32 {
    let mut cleaned_price = price;
    for word in REMOVE_WORDS {
        cleaned_price = cleaned_price.replace(word, "");
    }
    for (from, to) in REPLACE_WORDS {
        cleaned_price = cleaned_price.replace(from, to);
    }
    cleaned_price.parse::<f32>().unwrap_or(PRICE_DEFAULT)
}

#[derive(Debug, Default, Copy, Clone)]
pub struct PositionsParser {}

impl Parser for PositionsParser {}

impl PositionsParser {
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
                let mut products = vec![];
                for opt_category in &shop_rules.url_categories {
                    let new_products =
                        Self::parse_all_products(&shop, &mut proxies, &shop_rules, opt_category)?;
                    products.extend(new_products);
                }
                Ok(products)
            });

        task.await?
    }

    pub fn parse_all_products(
        shop: &Shop,
        proxies: &mut Vec<Proxy>,
        shop_rules: &ShopParsingRules,
        category: &Option<String>,
    ) -> Result<Vec<HoyaPosition>, ParserError> {
        let mut all_positions = vec![];

        let (page_positions, n_pages) = Self::parse_page(shop, proxies, shop_rules, category, 1)?;
        all_positions.extend(page_positions);

        // parse rest of the pages
        for page_id in 2..=n_pages {
            shop_rules.sleep()?;
            let (page_positions, _) =
                Self::parse_page(shop, proxies, shop_rules, category, page_id)?;
            all_positions.extend(page_positions);
        }
        Ok(all_positions)
    }

    pub fn parse_page(
        shop: &Shop,
        proxies: &mut Vec<Proxy>,
        shop_rules: &ShopParsingRules,
        category: &Option<String>,
        page_id: u32,
    ) -> Result<(Vec<HoyaPosition>, u32), ParserError> {
        let proxy = Self::find_proxy(proxies).ok();
        let client = Self::create_client(proxy)?;
        let parsing_url = shop_rules.get_shop_parsing_url(page_id, category);
        let response_text = client.get(parsing_url).send()?.text()?;
        let document = Html::parse_document(&response_text);
        let mut n_pages = 0;
        if page_id == 1 {
            n_pages = Self::retrieve_page_count(shop_rules, &document)?
        };
        let positions = Self::parse_data(shop, shop_rules, &document)?;
        Ok((positions, n_pages))
    }

    pub fn find_proxy(proxies: &mut Vec<Proxy>) -> Result<Proxy, ParserError> {
        let mut rng = rand::thread_rng();
        proxies.shuffle(&mut rng);
        proxies.pop().ok_or(ParserError::NoProxyAvailable)
    }

    fn parse_data(
        shop: &Shop,
        shop_rules: &ShopParsingRules,
        document: &Html,
    ) -> Result<Vec<HoyaPosition>, ParserError> {
        let mut products = vec![];
        let table_selector = Selector::parse(&shop_rules.product_table_lookup)
            .map_err(|_| ParserError::CrawlerSelectorError)?;
        let prod_selector = Selector::parse(&shop_rules.product_lookup)
            .map_err(|_| ParserError::CrawlerSelectorError)?;
        for table in document.select(&table_selector) {
            for product in table.select(&prod_selector) {
                let position = Self::parse_product(shop, shop_rules, product)?;
                products.push(position);
            }
        }
        Ok(products)
    }

    pub fn parse_product(
        shop: &Shop,
        shop_rules: &ShopParsingRules,
        product: ElementRef,
    ) -> Result<HoyaPosition, ParserError> {
        let name = Self::select_data_point(product, &shop_rules.name_lookup, false)?;
        let price = Self::select_data_point(product, &shop_rules.price_lookup, false)?;
        let price = clean_price(price);
        let url =
            Self::select_data_point(product, &shop_rules.url_lookup, shop_rules.look_for_href)?;
        Ok(HoyaPosition::new(shop.clone(), name, price, url))
    }

    pub fn retrieve_page_count(
        shop_rules: &ShopParsingRules,
        document: &Html,
    ) -> Result<u32, ParserError> {
        let mut max = 0;
        let selector = Selector::parse(&shop_rules.max_page_lookup)
            .map_err(|_| ParserError::CrawlerSelectorError)?;
        for element in document.select(&selector) {
            let num = Self::clean_data_point(element);
            if let Ok(number) = num.parse::<u32>() {
                if max < number {
                    max = number;
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
    fn clean_price_fails() {
        let price = clean_price("abc".to_string());
        assert!(price - PRICE_DEFAULT < f32::EPSILON);
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
            let parser =
                PositionsParser::find_proxy(&mut proxies).expect(&format!("Failed to find proxy"));
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
        let res = PositionsParser::find_proxy(&mut proxies);
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
        let max_page = PositionsParser::retrieve_page_count(&shop_rules, &html);
        assert!(max_page.is_ok());
        assert_eq!(max_page.unwrap(), 3);
    }

    #[test]
    fn parse_product_href_works() {
        let shop = create_test_shop();
        let shop_rules = ShopParsingRules {
            url_categories: vec![None],
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
        let result = PositionsParser::parse_product(&shop, &shop_rules, element.unwrap());
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
            url_categories: vec![None],
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
        let result = PositionsParser::parse_product(&shop, &shop_rules, element.unwrap());
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
            url_categories: vec![None],
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
        let result = PositionsParser::parse_data(&shop, &shop_rules, &html);
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
