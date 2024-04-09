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
        result.ok_or(DBError::PricesNotFound)
    }

    pub async fn push_shop_back(&self, shop: &crate::db::Shop) -> Result<(), DBError> {
        let now = OffsetDateTime::now_utc();
        use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

        let (year, month, date) = now.to_calendar_date();
        let d = NaiveDate::from_ymd_opt(year, month as u32, date as u32).unwrap();
        let t = NaiveTime::from_hms_milli_opt(12, 34, 56, 789).unwrap();

        let dt = NaiveDateTime::new(d, t);
        match Shop::find_by_id(shop.id as i32)
            .one(&self.connection)
            .await?
        {
            None => Err(DBError::ShopNotFound),
            Some(db_shop) => {
                let mut db_shop: shop::ActiveModel = db_shop.into();
                db_shop.last_parsed = Set(Some(dt));
                let _ = db_shop.update(&self.connection).await?;
                Ok(())
            }
        }
    }
}
