use sea_orm::entity::prelude::*;
use sea_orm::{JoinType, QuerySelect, Condition};
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
pub enum Relation {
    #[sea_orm(has_many = "super::group_user::Entity")]
    GroupUsers,
    #[sea_orm(has_many = "super::user_permission::Entity")]
    UserPermissions,
}

impl Related<super::group::Entity> for Entity {
    fn to() -> RelationDef {
        super::group_user::Relation::Group.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::group_user::Relation::User.def().rev())
    }
}

use super::{permission, user_permission, group_permission};

// ActiveModelの振る舞い（保存前バリデーションなど）を定義。
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// ユーザーが指定された権限を持っているか確認する。
    ///
    /// Djangoの `user.has_perm('app.view_model')` に相当。
    /// 以下の順でチェックします：
    /// 1. `is_admin` フラグが true なら無条件で許可（スーパーユーザー）
    /// 2. `user_permissions` テーブルに直接付与されているか
    /// 3. 所属する `groups` の `group_permissions` に含まれているか
    pub async fn has_perm(&self, db: &DatabaseConnection, perm_codename: &str) -> Result<bool, DbErr> {
        // 1. スーパーユーザーチェック
        if self.is_admin {
            return Ok(true);
        }

        // 2. 直接付与およびグループ経由の権限を一括チェック (1クエリに最適化)
        // 以下のリレーションを結合:
        // permissions -> user_permissions (直接)
        // permissions -> group_permissions -> group -> group_user (グループ経由)
        let has_perm = permission::Entity::find()
            .left_join(user_permission::Entity)
            .join(JoinType::LeftJoin, permission::Relation::GroupPermission.def())
            .join(JoinType::LeftJoin, group_permission::Relation::Group.def())
            .join(JoinType::LeftJoin, super::group::Relation::GroupUsers.def())
            .filter(permission::Column::Codename.eq(perm_codename))
            .filter(
                Condition::any()
                    .add(user_permission::Column::UserId.eq(self.id))
                    .add(super::group_user::Column::UserId.eq(self.id))
            )
            .count(db)
            .await?;

        Ok(has_perm > 0)
    }
}
