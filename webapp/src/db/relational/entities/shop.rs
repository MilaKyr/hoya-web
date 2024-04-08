//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "shop")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub url: String,
    pub logo: String,
    pub last_parsed: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::parsingcategory::Entity")]
    Parsingcategory,
    #[sea_orm(has_many = "super::parsinglookup::Entity")]
    Parsinglookup,
    #[sea_orm(has_many = "super::shopparsingrules::Entity")]
    Shopparsingrules,
    #[sea_orm(has_many = "super::shopposition::Entity")]
    Shopposition,
}

impl Related<super::parsingcategory::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parsingcategory.def()
    }
}

impl Related<super::parsinglookup::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parsinglookup.def()
    }
}

impl Related<super::shopparsingrules::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Shopparsingrules.def()
    }
}

impl Related<super::shopposition::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Shopposition.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
