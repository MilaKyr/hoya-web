pub mod entities;

use std::collections::HashMap;
use entities::{prelude::*, *};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use sea_orm::prelude::Decimal;
use time::Date;
use time::macros::format_description;

use crate::db::{DatabaseProduct, HoyaPosition, Shop as InnerShop};
use crate::db::errors::DBError;

#[derive(Debug, Default)]
pub struct RelationalDB {
    pub connection: DatabaseConnection,
}

impl RelationalDB {
    pub fn init(connection: DatabaseConnection) -> Self {
        Self {
            connection
        }
    }


    pub async fn all_products(&self) -> Result<Vec<DatabaseProduct>, DBError> {
        let products =  Product::find().all(&self.connection).await?;
        Ok(products.into_iter().map(|prod| prod.into()).collect())
    }

    pub async fn get_product_by(&self, id: u32) -> Result<DatabaseProduct, DBError> {
        let product = Product::find_by_id(id as i32).one(&self.connection).await?;
        match product {
            None => Err(DBError::UnknownProduct),
            Some(prod) => Ok(prod.into())
        }
    }

    pub async fn get_positions_for(&self, product: &DatabaseProduct) -> Result<Vec<HoyaPosition>, DBError> {
        let db_product = self.get_product_by(product.id).await?;
        let shops = self.all_shops().await?;
        let shops_hm: HashMap<i32, InnerShop> = shops.into_iter().map(|shop| (shop.id as i32, shop.into())).collect();
        let positions = Shopposition::find()
            .filter(shopposition::Column::ProductId.eq(db_product.id as i32))
            .all(&self.connection)
            .await?;
        let mut product_positions = vec![];
        for position in positions.into_iter() {
            if let Some(shop_info) = shops_hm.get(&position.shop_id).cloned() {
                let prod = HoyaPosition::init(position, shop_info, &db_product);
                product_positions.push(prod)
            }
        }
        Ok(product_positions)
    }

    pub async fn all_shops(&self) -> Result<Vec<InnerShop>, DBError> {
        let shops = Shop::find().all(&self.connection).await?;
        Ok(shops.into_iter().map(|shop| shop.into()).collect())
    }

    pub async fn get_prices_for(&self, product: &DatabaseProduct) -> Result<Vec<(Date, f32)>, DBError> {
        let prices = Historicprice::find()
            .filter(historicprice::Column::ProductId.eq(product.id as i32))
            .all(&self.connection)
            .await?;
        Ok(prices.into_iter()
            .map(|price| {
                let format = format_description!("[year]-[month]-[day]");
                let proper_date = time::Date::parse(&price.date.format("%Y-%m-%d").to_string(), format).expect("Failed to parse date");
                let price = price.avg_price.unwrap_or(Decimal::new(0, 1)).to_string().parse::<f32>().unwrap();
                (proper_date, price)
            })
            .collect())
    }
}