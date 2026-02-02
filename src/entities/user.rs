use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

// Djangoの `models.Model` に相当する構造体。
// SeaORMではマクロを使ってDBテーブルとのマッピング、リレーション、アクティブレコードパターンを定義します。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "user")] // Djangoの `class Meta: db_table = "user"` に相当
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_admin: bool,
}

// Djangoの `RelatedName` や `ForeignKey` などのリレーションを定義する場所。
// 今回はリレーションなしのシンプルな構成です。
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

// ActiveModelの振る舞い（保存前バリデーションなど）を定義。
impl ActiveModelBehavior for ActiveModel {}
