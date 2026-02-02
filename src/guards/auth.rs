use rocket::request::{Outcome, Request, FromRequest};
use rocket::http::Status;
use rocket::State;
use sea_orm::*;
use crate::entities::{prelude::*, user};

/// 認証済みユーザーを表すリクエストガード。
/// Djangoの `request.user` に相当し、ビューの引数に含めるだけで自動的に認証チェックが行われます。
pub struct AuthenticatedUser {
    pub user: user::Model,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // DBコネクションを取得
        let db = match request.guard::<&State<DatabaseConnection>>().await {
            Outcome::Success(db) => db,
            _ => return Outcome::Error((Status::InternalServerError, ())),
        };

        // セッション代わりのクッキーを確認
        let user_id = request
            .cookies()
            .get_private("user_id")
            .and_then(|c| c.value().parse::<i32>().ok());

        match user_id {
            Some(id) => {
                // DBからユーザーを取得
                match User::find_by_id(id).one(db.inner()).await {
                    Ok(Some(user)) if user.is_active => {
                        Outcome::Success(AuthenticatedUser { user })
                    }
                    _ => Outcome::Error((Status::Unauthorized, ())),
                }
            }
            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}

/// 管理者ユーザーのみを許可するガード。
/// Djangoの `user.is_staff` や `PermissionRequiredMixin` に相当。
pub struct AdminUser(pub AuthenticatedUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthenticatedUser::from_request(request).await {
            Outcome::Success(auth) => {
                if auth.user.is_admin {
                    Outcome::Success(AdminUser(auth))
                } else {
                    Outcome::Error((Status::Forbidden, ()))
                }
            }
            _ => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
