//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "shopparsingrules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub shop_id: i32,
    pub url: String,
    pub lookup_id: i32,
    pub look_for_href: Option<bool>,
    pub sleep_timeout_sec: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::parsinglookup::Entity",
        from = "Column::LookupId",
        to = "super::parsinglookup::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Parsinglookup,
    #[sea_orm(
        belongs_to = "super::shop::Entity",
        from = "Column::ShopId",
        to = "super::shop::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Shop,
}

impl Related<super::parsinglookup::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parsinglookup.def()
    }
}

impl Related<super::shop::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Shop.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}