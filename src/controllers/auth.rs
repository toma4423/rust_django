use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::Redirect;
use rocket::form::Form;
use rocket::State;
use sea_orm::*;
use serde::Deserialize;
use crate::entities::{prelude::*, user};
use crate::auth_utils::verify_password;

use crate::views::app_template::AppTemplate;
use rocket_dyn_templates::context;

#[derive(FromForm, Deserialize)]
pub struct LoginForm<'r> {
    pub username: &'r str,
    pub password: &'r str,
}

/// ログイン画面 (GET)
/// `templates/login.html.tera` (旧 index.html.tera) を表示
#[get("/login")]
pub fn login_form() -> AppTemplate {
    AppTemplate::new("login", context! {
        title: "Login",
    })
}

/// ログイン処理を行うビュー。
/// Djangoの `LoginView` に相当します。
#[post("/login", data = "<login_form>")]
pub async fn login(
    db: &State<DatabaseConnection>,
    login_form: Form<LoginForm<'_>>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Status> {
    // ユーザーをDBから取得 (Djangoの User.objects.get(username=...) に相当)
    let user_result = User::find()
        .filter(user::Column::Username.eq(login_form.username))
        .one(db.inner())
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::Unauthorized)?;

    // パスワード検証 (Djangoの check_password に相当)
    if !verify_password(login_form.password, &user_result.password_hash) {
        return Err(Status::Unauthorized);
    }

    // アクティブユーザーかチェック (Djangoの user.is_active に相当)
    if !user_result.is_active {
        return Err(Status::Forbidden);
    }

    // セッションクッキーをセット (Djangoの login(request, user) に相当)
    cookies.add_private(Cookie::new("user_id", user_result.id.to_string()));
    
    // Smart Redirect
    let redirect_url = if user_result.is_admin {
        "/admin"
    } else {
        "/"
    };
    
    Ok(Redirect::to(redirect_url))
}

/// ログアウト処理。
/// Djangoの `LogoutView` に相当します。
#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private(Cookie::from("user_id"));
    Redirect::to("/")
}
