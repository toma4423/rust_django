use sea_orm::entity::prelude::*;
use sea_orm::{JoinType, QuerySelect};
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

        // 2. 直接付与された権限のチェック
        // SELECT count(*) FROM permissions
        // JOIN user_permissions ON permissions.id = user_permissions.permission_id
        // WHERE user_permissions.user_id = $1 AND permissions.codename = $2
        let user_has = permission::Entity::find()
            .join(JoinType::InnerJoin, permission::Relation::UserPermission.def())
            .filter(user_permission::Column::UserId.eq(self.id))
            .filter(permission::Column::Codename.eq(perm_codename))
            .count(db)
            .await?;

        if user_has > 0 {
            return Ok(true);
        }

        // 3. グループ経由の権限チェック
        // まず所属グループIDを取得 (これをサボるとクエリが複雑になりすぎるため分割)
        let group_ids: Vec<i32> = super::group_user::Entity::find()
            .filter(super::group_user::Column::UserId.eq(self.id))
            .select_only()
            .column(super::group_user::Column::GroupId)
            .into_tuple()
            .all(db)
            .await?;

        if group_ids.is_empty() {
             return Ok(false);
        }

        // SELECT count(*) FROM permissions
        // JOIN group_permissions ON permissions.id = group_permissions.permission_id
        // WHERE group_permissions.group_id IN ($groups) AND permissions.codename = $2
        let group_has = permission::Entity::find()
             .join(JoinType::InnerJoin, permission::Relation::GroupPermission.def())
             .filter(group_permission::Column::GroupId.is_in(group_ids))
             .filter(permission::Column::Codename.eq(perm_codename))
             .count(db)
             .await?;

        Ok(group_has > 0)
    }
}
