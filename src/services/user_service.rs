use sea_orm::*;
use crate::entities::{prelude::*, user};
use crate::auth_utils::{hash_password, verify_password};
use crate::errors::AppError;

/// ユーザー関連のビジネスロジックを集約するサービス。
/// Djangoの Manager やカスタム QuerySet メソッドに相当します。
pub struct UserService;

impl UserService {
    /// IDでユーザーを検索 (Django: User.objects.get(pk=id))
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<user::Model>, AppError> {
        User::find_by_id(id)
            .one(db)
            .await
            .map_err(AppError::Database)
    }

    /// ユーザー名で検索 (Django: User.objects.get(username=...))
    pub async fn find_by_username(db: &DatabaseConnection, username: &str) -> Result<Option<user::Model>, AppError> {
        User::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await
            .map_err(AppError::Database)
    }

    /// 全ユーザー取得 (Django: User.objects.all())
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<user::Model>, AppError> {
        User::find()
            .all(db)
            .await
            .map_err(AppError::Database)
    }

    /// ユーザー作成 (Django: User.objects.create_user())
    pub async fn create(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
        is_admin: bool,
    ) -> Result<user::Model, AppError> {
        let password_hash = hash_password(password)?;
        
        let new_user = user::ActiveModel {
            username: Set(username.to_owned()),
            password_hash: Set(password_hash),
            is_active: Set(true),
            is_admin: Set(is_admin),
            ..Default::default()
        };

        new_user.insert(db).await.map_err(AppError::Database)
    }

    /// 認証処理 (Django: authenticate())
    pub async fn authenticate(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> Result<user::Model, AppError> {
        let user = Self::find_by_username(db, username)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if !verify_password(password, &user.password_hash) {
            return Err(AppError::Unauthorized);
        }

        if !user.is_active {
            return Err(AppError::Forbidden);
        }

        Ok(user)
    }

    /// アクティブユーザーかチェック
    pub fn is_active(user: &user::Model) -> bool {
        user.is_active
    }

    /// 管理者かチェック
    pub fn is_admin(user: &user::Model) -> bool {
        user.is_admin
    }
}
