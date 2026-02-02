use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// TODOモデル。
/// Djangoの `models.Model` に相当します。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "todo")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    
    /// TODOのタイトル (必須、最大200文字)
    pub title: String,
    
    /// 詳細な説明 (任意)
    pub description: Option<String>,
    
    /// 完了状態
    pub completed: bool,
    
    /// 優先度 (1: 低, 2: 中, 3: 高)
    pub priority: i32,
    
    /// 作成者のユーザーID (外部キー)
    pub user_id: i32,

    /// グループID (任意、外部キー)
    pub group_id: Option<i32>,
    
    /// 作成日時
    pub created_at: DateTimeWithTimeZone,
    
    /// 更新日時
    pub updated_at: DateTimeWithTimeZone,
}

/// リレーション定義
/// Djangoの `ForeignKey` に相当します。
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::group::Entity",
        from = "Column::GroupId",
        to = "super::group::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Group,
}

impl Related<super::group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
