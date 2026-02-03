use rocket::form::Form;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{Template, context};
use rocket::serde::json::serde_json;
use sea_orm::*;
use serde::Deserialize;
use crate::entities::{prelude::*, user, group, group_user};
use crate::guards::auth::AdminUser;
use crate::auth_utils::hash_password;
use crate::csrf::CsrfToken;

/// ユーザー作成・編集フォームのデータ構造
/// Djangoの `ModelForm` に相当
#[derive(FromForm, Deserialize)]
pub struct UserForm<'r> {
    pub username: &'r str,
    pub password: &'r str,
    #[field(default = false)]
    pub is_admin: bool,
    #[field(default = false)]
    pub is_active: bool,
    /// 所属グループIDリスト
    #[field(default = Vec::new())]
    pub group_ids: Vec<i32>,
    /// CSRFトークン（フォームから受け取る）
    #[field(default = "")]
    pub csrf_token: &'r str,
}

#[get("/")]
pub fn dashboard(admin: AdminUser) -> Template {
    Template::render("admin/dashboard", context! {
        username: admin.0.user.username,
        active_nav: "dashboard",
    })
}

use crate::views::list::ListView;

pub struct UserListView;

impl ListView<User> for UserListView {
    fn template_name(&self) -> &'static str {
        "admin/list"
    }

    fn filter_queryset(&self, query: Select<User>, q: &str) -> Select<User> {
         query.filter(user::Column::Username.contains(q))
    }

    fn get_context_data(&self, _db: &DatabaseConnection) -> serde_json::Value {
        // CSRFトークンはコントローラー側で注入するか、ここで取る手段が必要
        // 現状のListView設計ではCSRFをうまく渡せないので、render後にマージするか、
        // context変数を渡せるようにsignatureを変えるのがベターだが、
        // いったんシンプルに実装。
        serde_json::json!({
            "active_nav": "users",
        })
    }
}

/// ユーザー一覧を表示する管理画面。
/// Generic View (`ListView`) を使用してリファクタリング済み。
#[get("/users?<page>&<q>")]
pub async fn list_users(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    page: Option<usize>,
    q: Option<String>,
) -> Template {
    let view = UserListView;
    let view_context = serde_json::json!({
        "csrf_token": csrf.token(),
    });
    view.list(db, page.unwrap_or(1), q.clone(), view_context).await
}

/// ユーザー作成フォーム (GET)。
/// Djangoの `CreateView` (GET) に相当。
#[get("/users/create")]
pub async fn create_user_form(db: &State<DatabaseConnection>, _admin: AdminUser, csrf: CsrfToken) -> Template {
    let all_groups = Group::find().all(db.inner()).await.unwrap_or_default();
    Template::render("admin/form", context! {
        active_nav: "users",
        csrf_token: csrf.token(),
        all_groups: all_groups,
    })
}

/// ユーザー作成処理 (POST)。
/// Djangoの `CreateView` (POST) または `form.save()` に相当。
#[post("/users/create", data = "<form>")]
pub async fn create_user(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    form: Form<UserForm<'_>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // CSRF検証
    if !csrf.verify(form.csrf_token) {
        return Err(Flash::error(Redirect::to("/admin/users/create"), "CSRF検証に失敗しました"));
    }

    // バリデーション: 空のユーザー名はエラー
    if form.username.trim().is_empty() {
        return Err(Flash::error(Redirect::to("/admin/users/create"), "ユーザー名は必須です"));
    }

    // パスワードをArgon2でハッシュ化 (Djangoの make_password に相当)
    let password_hash = hash_password(form.password)
        .map_err(|_| Flash::error(Redirect::to("/admin/users/create"), "パスワードのハッシュ化に失敗しました"))?;

    // ActiveModel を使ってユーザーを作成 (Djangoの User.objects.create に相当)
    let new_user = user::ActiveModel {
        username: Set(form.username.to_owned()),
        password_hash: Set(password_hash),
        is_active: Set(form.is_active),
        is_admin: Set(form.is_admin),
        ..Default::default()
    };

    // DBに挿入
    let inserted_user = new_user
        .insert(db.inner())
        .await
        .map_err(|e| {
            // ユニーク制約違反をキャッチ (Djangoの IntegrityError に相当)
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                Flash::error(Redirect::to("/admin/users/create"), "このユーザー名は既に使用されています")
            } else {
                Flash::error(Redirect::to("/admin/users/create"), "ユーザーの作成に失敗しました")
            }
        })?;

    // グループ紐付け
    if !form.group_ids.is_empty() {
        let new_relations: Vec<group_user::ActiveModel> = form.group_ids.iter().map(|&gid| {
            group_user::ActiveModel {
                user_id: Set(inserted_user.id),
                group_id: Set(gid),
                ..Default::default()
            }
        }).collect();
        // エラーハンドリングは省略（ログ出力などすべき）
        let _ = group_user::Entity::insert_many(new_relations).exec(db.inner()).await;
    }

    Ok(Flash::success(Redirect::to("/admin/users"), "ユーザーを正常に追加しました"))
}

/// ユーザー編集フォーム (GET)。
/// Djangoの `UpdateView` (GET) に相当。
#[get("/users/edit/<id>")]
pub async fn edit_user_form(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    id: i32,
) -> Result<Template, Flash<Redirect>> {
    let user = User::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/admin/users"), "ユーザーの取得に失敗しました"))?
        .ok_or_else(|| Flash::error(Redirect::to("/admin/users"), "ユーザーが見つかりません"))?;

    // 所属グループを取得
    let user_groups = user.find_related(Group).all(db.inner()).await.unwrap_or_default();
    let user_group_ids: Vec<i32> = user_groups.iter().map(|g| g.id).collect();
    // 全グループを取得
    let all_groups = Group::find().all(db.inner()).await.unwrap_or_default();

    Ok(Template::render("admin/form", context! {
        user: user,
        active_nav: "users",
        csrf_token: csrf.token(),
        all_groups: all_groups,
        user_group_ids: user_group_ids,
    }))
}

/// ユーザー編集処理 (POST)。
/// Djangoの `UpdateView` (POST) に相当。
#[post("/users/edit/<id>", data = "<form>")]
pub async fn edit_user(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    id: i32,
    form: Form<UserForm<'_>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // CSRF検証
    if !csrf.verify(form.csrf_token) {
        return Err(Flash::error(Redirect::to(format!("/admin/users/edit/{}", id)), "CSRF検証に失敗しました"));
    }

    // 既存ユーザーを取得
    let existing_user = User::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/admin/users"), "ユーザーの取得に失敗しました"))?
        .ok_or_else(|| Flash::error(Redirect::to("/admin/users"), "ユーザーが見つかりません"))?;

    // バリデーション
    if form.username.trim().is_empty() {
        return Err(Flash::error(Redirect::to(format!("/admin/users/edit/{}", id)), "ユーザー名は必須です"));
    }

    // ActiveModelに変換して更新
    let mut active_model: user::ActiveModel = existing_user.into();
    active_model.username = Set(form.username.to_owned());
    active_model.is_active = Set(form.is_active);
    active_model.is_admin = Set(form.is_admin);

    // パスワードが入力された場合のみ更新
    if !form.password.is_empty() {
        let password_hash = hash_password(form.password)
            .map_err(|_| Flash::error(Redirect::to(format!("/admin/users/edit/{}", id)), "パスワードのハッシュ化に失敗しました"))?;
        active_model.password_hash = Set(password_hash);
    }

    // 更新を実行
    active_model
        .update(db.inner())
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                Flash::error(Redirect::to(format!("/admin/users/edit/{}", id)), "このユーザー名は既に使用されています")
            } else {
                Flash::error(Redirect::to(format!("/admin/users/edit/{}", id)), "ユーザーの更新に失敗しました")
            }
        })?;

    // グループ更新 (既存削除 -> 新規追加)
    // 自分の削除権限などでエラーが出ないようにトランザクションが理想だが今回は簡易実装
    let _ = group_user::Entity::delete_many()
        .filter(group_user::Column::UserId.eq(id))
        .exec(db.inner())
        .await;

    if !form.group_ids.is_empty() {
        let new_relations: Vec<group_user::ActiveModel> = form.group_ids.iter().map(|&gid| {
            group_user::ActiveModel {
                user_id: Set(id),
                group_id: Set(gid),
                ..Default::default()
            }
        }).collect();
        let _ = group_user::Entity::insert_many(new_relations).exec(db.inner()).await;
    }

    Ok(Flash::success(Redirect::to("/admin/users"), "ユーザーを正常に変更しました"))
}

/// ユーザー削除処理 (POST)。
/// Djangoの `DeleteView` に相当。
#[post("/users/delete/<id>")]
pub async fn delete_user(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    id: i32,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // ユーザーを削除
    User::delete_by_id(id)
        .exec(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/admin/users"), "ユーザーの削除に失敗しました"))?;

    Ok(Flash::success(Redirect::to("/admin/users"), "ユーザーを正常に削除しました"))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        dashboard,
        list_users,
        create_user_form,
        create_user,
        edit_user_form,
        edit_user,
        delete_user
    ]
}
