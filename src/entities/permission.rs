use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "permissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(unique)]
    pub codename: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::group_permission::Entity")]
    GroupPermission,
    #[sea_orm(has_many = "super::user_permission::Entity")]
    UserPermission,
}

impl Related<super::group_permission::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GroupPermission.def()
    }
}

impl Related<super::user_permission::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserPermission.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
