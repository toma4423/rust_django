use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "group")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

    #[sea_orm(has_many = "super::todo::Entity")]
    Todos,
    #[sea_orm(has_many = "super::group_user::Entity")]
    GroupUsers,
}



impl Related<super::todo::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Todos.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::group_user::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::group_user::Relation::Group.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
