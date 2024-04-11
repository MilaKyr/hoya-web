pub mod entities;

use entities::{prelude::*, *};
use sea_orm::prelude::Decimal;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbBackend, EntityTrait,
    FromQueryResult, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};
use std::collections::HashMap;
use time::macros::format_description;
use time::{Date, OffsetDateTime};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use url::Url;

use crate::db::errors::DBError;
use crate::db::in_memory::ShopParsingRules;
use crate::db::{
    DatabaseProduct, HoyaPosition, ProductFilter, Proxy as InnerProxy, ProxyParsingRules,
    SearchFilter, Shop as InnerShop,
};

#[derive(Debug, Default)]
pub struct RelationalDB {
    pub connection: DatabaseConnection,
}

impl RelationalDB {
    pub fn init(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    pub async fn all_products(&self) -> Result<Vec<DatabaseProduct>, DBError> {
        let products = Product::find().all(&self.connection).await?;
        Ok(products.into_iter().map(|prod| prod.into()).collect())
    }

    pub async fn get_product_by(&self, id: u32) -> Result<DatabaseProduct, DBError> {
        let product = Product::find_by_id(id as i32).one(&self.connection).await?;
        match product {
            None => Err(DBError::UnknownProduct),
            Some(prod) => Ok(prod.into()),
        }
    }

    pub async fn get_positions_for(
        &self,
        product: &DatabaseProduct,
    ) -> Result<Vec<HoyaPosition>, DBError> {
        let mut product_positions = vec![];
        let positions = Shopposition::find()
            .find_also_related(Shop)
            .filter(shopposition::Column::ProductId.eq(product.id as i32))
            .all(&self.connection)
            .await?;
        for (position, poss_shop) in positions.into_iter() {
            if let Some(shop) = poss_shop {
                let prod = HoyaPosition::try_init(position, shop.into(), product)?;
                product_positions.push(prod);
            }
        }
        Ok(product_positions)
    }

    pub async fn all_shops(&self) -> Result<Vec<InnerShop>, DBError> {
        let shops = Shop::find().all(&self.connection).await?;
        Ok(shops.into_iter().map(|shop| shop.into()).collect())
    }

    pub async fn get_prices_for(
        &self,
        product: &DatabaseProduct,
    ) -> Result<Vec<(Date, f32)>, DBError> {
        let mut final_prices = vec![];
        let format = format_description!("[year]-[month]-[day]");
        let prices = Historicprice::find()
            .filter(historicprice::Column::ProductId.eq(product.id as i32))
            .all(&self.connection)
            .await?;
        for hprice in prices.into_iter() {
            let proper_date = Date::parse(&hprice.date.format("%Y-%m-%d").to_string(), format)?;
            let price = hprice.avg_price.to_string().parse::<f32>()?;
            final_prices.push((proper_date, price));
        }
        Ok(final_prices)
    }

    pub async fn get_top_shop(&self) -> Result<crate::db::Shop, DBError> {
        let next = Shop::find()
            .filter(shop::Column::LastParsed.is_null())
            .one(&self.connection)
            .await?;
        match next {
            None => {
                let next = Shop::find()
                    .order_by_asc(shop::Column::LastParsed)
                    .one(&self.connection)
                    .await?
                    .ok_or(DBError::ShopNotFound)?;
                Ok(next.into())
            }
            Some(model) => Ok(model.into()),
        }
    }

    pub async fn get_shop_parsing_rules(
        &self,
        shop: &crate::db::Shop,
    ) -> Result<ShopParsingRules, DBError> {
        let rules = Shopparsingrules::find_by_id(shop.id as i32)
            .one(&self.connection)
            .await?
            .ok_or(DBError::ParsingRulesNotFound)?;
        let categories = Parsingcategory::find()
            .filter(parsingcategory::Column::ShopId.eq(shop.id as i32))
            .all(&self.connection)
            .await?;
        let lookups = Parsinglookup::find()
            .filter(parsinglookup::Column::ShopId.eq(shop.id as i32))
            .one(&self.connection)
            .await?
            .ok_or(DBError::ParsingRulesNotFound)?;
        Ok(ShopParsingRules::with(rules, categories, lookups))
    }

    pub async fn get_proxy_parsing_rules(
        &self,
    ) -> Result<HashMap<Url, ProxyParsingRules>, DBError> {
        let proxy_rules = Proxyparsingrules::find()
            .find_also_related(Proxysources)
            .all(&self.connection)
            .await?;
        let result: HashMap<Url, ProxyParsingRules> = proxy_rules
            .into_iter()
            .filter(|(_, source)| source.is_some())
            .map(|(rules, source)| (rules, Url::parse(&source.unwrap().source)))
            .filter(|(_, source)| source.is_ok())
            .map(|(rules, source)| (source.unwrap(), rules.into()))
            .collect();
        Ok(result)
    }

    pub async fn save_proxies(&self, new_proxies: Vec<InnerProxy>) -> Result<(), DBError> {
        let proxies_to_save: Vec<_> = new_proxies
            .into_iter()
            .map(|prox| proxy::ActiveModel {
                url: Set(prox.to_string()),
                ..Default::default()
            })
            .collect();
        let _ = Proxy::insert_many(proxies_to_save)
            .exec(&self.connection)
            .await?;
        Ok(())
    }

    pub async fn get_proxies(&self) -> Result<Vec<InnerProxy>, DBError> {
        let db_proxies = Proxy::find().all(&self.connection).await?;
        Ok(db_proxies
            .into_iter()
            .filter_map(|proxy| Url::parse(&proxy.url).ok())
            .filter_map(|url| url.try_into().ok())
            .collect())
    }

    pub async fn save_positions(&self, positions: Vec<HoyaPosition>) -> Result<(), DBError> {
        let models: Vec<shopposition::ActiveModel> = positions
            .into_iter()
            .map(|pos| {
                let decimal_price = Decimal::from_f32_retain(pos.price).unwrap_or_default();
                shopposition::ActiveModel {
                    product_id: Set(0), // TODO
                    shop_id: Set(pos.shop.id as i32),
                    image: Set(None), // TODO
                    price: Set(decimal_price),
                    url: Set(pos.url.to_string()),
                    ..Default::default()
                }
            })
            .collect();
        Shopposition::insert_many(models)
            .exec(&self.connection)
            .await?;
        Ok(())
    }

    pub async fn search_with_filter(
        &self,
        filter: SearchFilter,
    ) -> Result<Vec<DatabaseProduct>, DBError> {
        if !filter.contains_query() {
            return self.all_products().await;
        }
        let query = filter.query().expect("Query cannot be empty");
        let db_products = Product::find()
            .filter(
                Condition::any()
                    .add(product::Column::Name.contains(&query.0))
                    .add(product::Column::Description.contains(&query.0)),
            )
            .all(&self.connection)
            .await?;
        Ok(db_products.into_iter().map(|prod| prod.into()).collect())
    }

    pub async fn get_product_filter(&self) -> Result<ProductFilter, DBError> {
        let statement = Shopposition::find()
            .select_only()
            .column(shopposition::Column::Price)
            .column_as(shopposition::Column::Price.min(), "min")
            .column_as(shopposition::Column::Price.max(), "max")
            .build(DbBackend::Postgres);
        let result = ProductFilter::find_by_statement(statement)
            .one(&self.connection)
            .await?;
        println!("{:?}", result);
        result.ok_or(DBError::PricesNotFound)
    }

    fn now(&self) -> Result<NaiveDateTime, DBError> {
        let now = OffsetDateTime::now_utc();

        let (year, month, date) = now.to_calendar_date();
        let date = NaiveDate::from_ymd_opt(year, month as u32, date as u32)
            .ok_or(DBError::DatetimeError)?;
        let time = NaiveTime::from_hms_opt(now.hour() as u32, now.minute() as u32, now.second() as u32)
            .ok_or(DBError::DatetimeError)?;
        Ok(NaiveDateTime::new(date, time))
    }

    pub async fn push_shop_back(&self, shop: &crate::db::Shop) -> Result<(), DBError> {
        match Shop::find_by_id(shop.id as i32)
            .one(&self.connection)
            .await?
        {
            None => Err(DBError::ShopNotFound),
            Some(db_shop) => {
                let mut db_shop: shop::ActiveModel = db_shop.into();
                let datetime = self.now()?;
                db_shop.last_parsed = Set(Some(datetime));
                let _ = db_shop.update(&self.connection).await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use super::*;
    use sea_orm::{entity::prelude::*, DatabaseBackend, MockDatabase, MockExecResult};
    use time::macros::date;
    use crate::db::SearchQuery;

    fn create_db<T: ModelTrait>(append_query_results: Vec<Vec<T>>) -> RelationalDB {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(append_query_results)
            .into_connection();
        RelationalDB::init(connection)
    }

    #[tokio::test]
    async fn test_all_products_work() {
        let origin_prod1 = product::Model {
            id: 1,
            name: "Prod 1".to_string(),
            description: None,
        };
        let origin_prod2 = product::Model {
            id: 2,
            name: "Prod 2".to_string(),
            description: None,
        };
        let expected_result = vec![origin_prod1, origin_prod2];
        let db = create_db(vec![expected_result]);
        let products = db.all_products().await
            .expect("Failed to get all products");
        assert!(!products.is_empty());
        assert_eq!(products.len(), 2);
    }

    #[tokio::test]
    async fn test_get_product_by_works() {
        let id = 1;
        let origin_prod = product::Model {
            id,
            name: "Prod 1".to_string(),
            description: None,
        };
        let origin_prod_db: DatabaseProduct = origin_prod.clone().into();
        let expected_result = vec![origin_prod];
        let db = create_db(vec![expected_result]);

        let product = db.get_product_by(id as u32).await;
        assert!(product.is_ok());
        assert_eq!(product.unwrap(), origin_prod_db);
    }

    #[tokio::test]
    async fn test_get_product_by_fails() {
        let db = create_db::<product::Model>(vec![vec![]]);
        let product = db.get_product_by(1).await;
        assert!(product.is_err());
        assert_eq!(product.err().unwrap().to_string(), DBError::UnknownProduct.to_string())
    }


    #[tokio::test]
    async fn test_get_positions_for_works() {
        let product = DatabaseProduct {
            name: "Prod 1".to_string(),
            id: 1,
        };
        let shop = shop::Model {
            id: 1,
            name: "new shop".to_string(),
            url: "http://new_shop.com".to_string(),
            logo: "".to_string(),
            last_parsed: None,
        };
        let positions = vec![
            (shopposition::Model {
                id: 1,
                product_id: 1,
                shop_id: 1,
                image: None,
                price: Decimal::new(254, 2),
                url: "https://example.com".to_string(),
            },
             shop.clone())
        ];
        let expected_result = HoyaPosition {
            shop: shop.into(),
            full_name: "Prod 1".to_string(),
            price: 2.54,
            url: "https://example.com".to_string(),
        };
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![positions])
            .into_connection();
        let db = RelationalDB::init(connection);
        let positions = db.get_positions_for(&product).await;
        assert!(positions.is_ok());
        let positions = positions.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions.first().unwrap(), &expected_result);
    }

    #[tokio::test]
    async fn test_all_shops_works() {
        let shop1 = shop::Model {
            id: 1,
            name: "new shop".to_string(),
            url: "http://new_shop.com".to_string(),
            logo: "".to_string(),
            last_parsed: None,
        };
        let shop2 = shop::Model {
            id: 1,
            name: "new shop".to_string(),
            url: "http://new_shop.com".to_string(),
            logo: "".to_string(),
            last_parsed: None,
        };
        let db = create_db(vec![vec![shop1, shop2]]);
        let shops = db.all_shops().await;
        assert!(shops.is_ok());
        let shops = shops.unwrap();
        assert_eq!(shops.len(), 2);
    }

    #[tokio::test]
    async fn test_get_prices_for_works() {
        let product = DatabaseProduct {
            name: "Prod1".to_string(),
            id: 1,
        };
        let price1 = historicprice::Model {
            id: 1,
            product_id: 1,
            date: NaiveDate::from_str("2024-01-01").expect("Failed to parse to date"),
            avg_price: Decimal::new(255, 1),
        };
        let price2 = historicprice::Model {
            id: 2,
            product_id: 1,
            date: NaiveDate::from_str("2024-02-01").expect("Failed to parse to date"),
            avg_price: Decimal::new(315, 1),
        };
        let db = create_db(vec![vec![price1, price2]]);
        let prices = db.get_prices_for(&product).await;
        assert!(prices.is_ok());
        let prices = prices.unwrap();
        assert_eq!(prices.len(), 2);
        assert_eq!(prices, vec![ (date!(2024-01-01), 25.5), (date!(2024-02-01), 31.5)]);
    }

    #[tokio::test]
    async fn test_get_top_shop_works_with_none() {
        let shop = shop::Model {
            id: 1,
            name: "new shop".to_string(),
            url: "http://new_shop.com".to_string(),
            logo: "".to_string(),
            last_parsed: None,
        };
        let db = create_db(vec![vec![shop.clone()]]);
        let result = db.get_top_shop().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, shop.into());
    }

    #[tokio::test]
    async fn test_get_top_shop_works() {
        let shop1 = shop::Model {
            id: 1,
            name: "new shop1".to_string(),
            url: "http://new_shop1.com".to_string(),
            logo: "".to_string(),
            last_parsed: Some(NaiveDateTime::new(
                NaiveDate::from_str("2024-01-01").expect("Failed to parse to date"),
                NaiveTime::from_str("11:11:11").expect("Failed to parse to time")
            )),
        };
        let shop2 = shop::Model {
            id: 2,
            name: "new shop2".to_string(),
            url: "http://new_shop2.com".to_string(),
            logo: "".to_string(),
            last_parsed: Some(NaiveDateTime::new(
                NaiveDate::from_str("2024-02-01").expect("Failed to parse to date"),
                NaiveTime::from_str("11:11:11").expect("Failed to parse to time")
            )),
        };
        let db = create_db(vec![vec![], vec![shop1.clone(), shop2.clone()]]);
        let result = db.get_top_shop().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, shop1.into());
    }

    #[tokio::test]
    async fn test_get_shop_parsing_rules_works() {
        let inner_shop =  crate::db::Shop {
            id: 1,
            logo: "".to_string(),
            name: "new shop".to_string(),
            url: "https://example.com".to_string(),
        };
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                vec![shopparsingrules::Model {
                    id: 1,
                    shop_id: 1,
                    url: "https://example.com".to_string(),
                    lookup_id: 1,
                    look_for_href: None,
                    sleep_timeout_sec: None,
                }]])
            .append_query_results([
                vec![
                    parsingcategory::Model {
                    id: 1,
                    shop_id: 1,
                    category: "category 1".to_string(),
                    },
                     parsingcategory::Model {
                         id: 2,
                         shop_id: 1,
                         category: "category 2".to_string(),
                     }
                ]])
            .append_query_results([
                vec![parsinglookup::Model{
                    id: 1,
                    shop_id: 1,
                    max_page: "max_page".to_string(),
                    product_table: "table".to_string(),
                    product: "product".to_string(),
                    name: "name".to_string(),
                    price: "price".to_string(),
                    url: "url".to_string(),
                }],
            ])
            .into_connection();
        let expected_result = ShopParsingRules {
            url_categories: vec!["category 1".to_string(), "category 2".to_string()],
            parsing_url: "https://example.com".to_string(),
            max_page_lookup: "max_page".to_string(),
            product_table_lookup: "table".to_string(),
            product_lookup: "product".to_string(),
            name_lookup: "name".to_string(),
            price_lookup: "price".to_string(),
            url_lookup: "url".to_string(),
            look_for_href: false,
            sleep_timeout_sec: None,
        };
        let db = RelationalDB::init(connection);
        let result = db.get_shop_parsing_rules(&inner_shop).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn test_get_proxy_parsing_rules() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                vec![
                    (proxyparsingrules::Model{
                    id: 1,
                    source_id: 1,
                    table_name: "table".to_string(),
                    head: "head".to_string(),
                    row: "row".to_string(),
                    data: "data".to_string(),
                }, proxysources::Model {
                    id: 1,
                    source: "https://example.com".to_string(),
                }),
                    (proxyparsingrules::Model{
                        id: 1,
                        source_id: 1,
                        table_name: "table.table".to_string(),
                        head: "head.head".to_string(),
                        row: "row.row".to_string(),
                        data: "data.dt".to_string(),
                    }, proxysources::Model {
                        id: 1,
                        source: "https://abc.com".to_string(),
                    })],
            ])
            .into_connection();
        let expected_result: HashMap<_,_> = vec![
            (
                Url::from_str("https://example.com").unwrap(),
                ProxyParsingRules {
                    table_lookup: "table".to_string(),
                    head_lookup: "head".to_string(),
                    row_lookup: "row".to_string(),
                    data_lookup: "data".to_string(),
                }
            ),
            (
                Url::from_str("https://abc.com").unwrap(),
                ProxyParsingRules {
                    table_lookup: "table.table".to_string(),
                    head_lookup: "head.head".to_string(),
                    row_lookup: "row.row".to_string(),
                    data_lookup: "data.dt".to_string(),
                }
            ),
        ].into_iter().collect();
        let db = RelationalDB::init(connection);
        let result = db.get_proxy_parsing_rules().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn test_save_proxies_works() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                vec![proxy::Model {
                    id: 1,
                    url: "127.0.0.1:8080".to_string(),
                }]
            ])
            .append_exec_results([
                MockExecResult { last_insert_id: 1, rows_affected: 1 },
            ])
            .into_connection();
        let db = RelationalDB::init(connection);
        let to_save = vec![InnerProxy{
            ip: "127.0.0.1".to_string(),
            port: 8080,
            https: false,
        }];
        let result = db.save_proxies(to_save).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_push_shop_back_works() {
        let shop = shop::Model {
            id: 1,
            name: "new shop1".to_string(),
            url: "http://new_shop1.com".to_string(),
            logo: "".to_string(),
            last_parsed: Some(NaiveDateTime::new(
                NaiveDate::from_str("2024-01-01").expect("Failed to parse to date"),
                NaiveTime::from_str("11:11:11").expect("Failed to parse to time")
            )),
        };
        let inner_shop: InnerShop = shop.clone().into();
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![shop.clone()], vec![shop.clone()]])
            .append_exec_results([
                MockExecResult { last_insert_id: 1, rows_affected: 1 },
            ])
            .into_connection();
        let db = RelationalDB::init(connection);
        let result = db.push_shop_back(&inner_shop).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_push_shop_back_fails() {
        let shop = InnerShop::dummy();
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![], vec![shop::Model {
                id: 0,
                name: "".to_string(),
                url: "".to_string(),
                logo: "".to_string(),
                last_parsed: None,
            }]])
            .into_connection();
        let db = RelationalDB::init(connection);
        let result = db.push_shop_back(&shop).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), DBError::ShopNotFound.to_string());
    }

    #[tokio::test]
    async fn test_save_positions_works() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                vec![shopposition::Model {
                    id: 1,
                    product_id: 1,
                    shop_id: 1,
                    image: None,
                    price: Decimal::new(354, 2),
                    url: "https://example.com".to_string(),
                }]
            ])
            .append_exec_results([
                MockExecResult { last_insert_id: 1, rows_affected: 1 },
            ])
            .into_connection();
        let db = RelationalDB::init(connection);
        let to_save = vec![HoyaPosition{
            shop: Default::default(),
            full_name: "position 1".to_string(),
            price: 3.54,
            url: "https://example.com".to_string(),
        }];
        let result = db.save_positions(to_save).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_proxies_works() {
        let proxies = vec![
            proxy::Model {
                id: 1,
                url: "http://127.0.0.1:8080".to_string(),
            },
            proxy::Model {
                id: 2,
                url: "https://127.0.0.1:8900".to_string(),
            }];
        let expected_result = vec![
            InnerProxy {
                ip: "127.0.0.1".to_string(),
                port: 8080,
                https: false,
            }, InnerProxy {
                ip: "127.0.0.1".to_string(),
                port: 8900,
                https: true,
            }];
        let db = create_db(vec![proxies]);
        let result = db.get_proxies().await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn test_get_product_filter_works() {
        let positions = vec![
            shopposition::Model {
                id: 1,
                product_id: 1,
                shop_id: 1,
                image: None,
                price: Decimal::new(2, 0),
                url: "".to_string(),
            },
            shopposition::Model {
                id: 2,
                product_id: 1,
                shop_id: 2,
                image: None,
                price: Decimal::new(100, 0),
                url: "".to_string(),
            }
        ];
        let db = create_db(vec![positions]);
        let result = db.get_product_filter().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_with_filter_works() {
        let filter = SearchFilter {
            product: None,
            query: SearchQuery("test".to_string()),
        };
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                vec![product::Model {
                    id: 1,
                    name: "test and ..".to_string(),
                    description: None,
                },
                     product::Model {
                         id: 2,
                         name: "test".to_string(),
                         description: None,
                     },
                product::Model {
                    id: 3,
                    name: "the test ...".to_string(),
                    description: None,
                }]
            ])
            .into_connection();
        let db = RelationalDB::init(connection);
        let expected_result = vec![DatabaseProduct {
            name: "test and ..".to_string(),
            id: 1,
        }, DatabaseProduct {
            name: "test".to_string(),
            id: 2,
        }, DatabaseProduct {
            name: "the test ...".to_string(),
            id: 3,
        }];
        let result = db.search_with_filter(filter).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn test_search_with_filter_works_returns_all() {
        let filter = SearchFilter {
            product: None,
            query: SearchQuery("".to_string()),
        };
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                vec![product::Model {
                    id: 1,
                    name: "test and ..".to_string(),
                    description: None,
                },
                     product::Model {
                         id: 2,
                         name: "test".to_string(),
                         description: None,
                     }]
            ])
            .into_connection();
        let db = RelationalDB::init(connection);
        let expected_result = vec![DatabaseProduct {
            name: "test and ..".to_string(),
            id: 1,
        }, DatabaseProduct {
            name: "test".to_string(),
            id: 2,
        }];
        let result = db.search_with_filter(filter).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected_result);
    }
}
