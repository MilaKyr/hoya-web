//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "proxysources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub source: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::proxyparsingrules::Entity")]
    Proxyparsingrules,
}

impl Related<super::proxyparsingrules::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Proxyparsingrules.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}