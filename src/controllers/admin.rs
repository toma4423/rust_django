use rocket::form::Form;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{Template, context};
use rocket::serde::json::serde_json;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use crate::entities::{prelude::*, user, group, group_user};
use crate::guards::auth::AdminUser;
use crate::auth_utils::hash_password;
use crate::csrf::CsrfToken;

/// ユーザー作成・編集フォームのデータ構造
/// Djangoの `ModelForm` に相当
#[derive(FromForm, Deserialize, Serialize)]
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
use crate::views::edit::{CreateView, UpdateView, DeleteView};

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

pub struct UserCreateView;

#[rocket::async_trait]
impl CreateView<user::ActiveModel> for UserCreateView {
    fn success_url(&self) -> String {
        "/admin/users".to_string()
    }

    async fn get_context_data(&self, db: &DatabaseConnection) -> serde_json::Value {
        let all_groups = Group::find().all(db).await.unwrap_or_default();
        serde_json::json!({
            "all_groups": all_groups
        })
    }
    
    async fn save(&self, db: &DatabaseConnection, data: &serde_json::Value) -> Result<user::Model, DbErr> {
         let username = data["username"].as_str().ok_or(DbErr::Custom("ユーザー名は必須です".into()))?;
         let password = data["password"].as_str().unwrap_or("");
         
         if username.trim().is_empty() {
             return Err(DbErr::Custom("ユーザー名は必須です".into()));
         }
         
         let password_hash = hash_password(password).map_err(|e| DbErr::Custom(e.to_string()))?;
         
         let is_admin = data["is_admin"].as_bool().unwrap_or(false);
         let is_active = data["is_active"].as_bool().unwrap_or(false);

         let active_model = user::ActiveModel {
            username: Set(username.to_owned()),
            password_hash: Set(password_hash),
            is_admin: Set(is_admin),
            is_active: Set(is_active),
            ..Default::default()
         };

         let user = active_model.insert(db).await.map_err(|e| {
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                DbErr::Custom("このユーザー名は既に使用されています".into())
            } else {
                e
            }
         })?;
         
         // Group relations
         if let Some(groups) = data["group_ids"].as_array() {
             let mut relations = Vec::new();
             for g in groups {
                  if let Some(gid) = g.as_i64() {
                      relations.push(group_user::ActiveModel {
                          user_id: Set(user.id),
                          group_id: Set(gid as i32),
                          ..Default::default()
                      });
                  }
             }
             if !relations.is_empty() {
                 group_user::Entity::insert_many(relations).exec(db).await?;
             }
         }
         
         Ok(user)
    }
}
/// ユーザー作成フォーム (GET)。
/// Djangoの `CreateView` (GET) に相当。
#[get("/users/create")]
pub async fn create_user_form(db: &State<DatabaseConnection>, _admin: AdminUser, csrf: CsrfToken) -> Template {
    let view = UserCreateView;
    let context = serde_json::json!({
        "active_nav": "users",
        "csrf_token": csrf.token(),
    });
    view.get(db, context).await
}

/// ユーザー作成処理 (POST)。
/// Djangoの `CreateView` (POST) または `form.save()` に相当。
#[post("/users/create", data = "<form>")]
pub async fn create_user(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    form: Form<UserForm<'_>>,
) -> Result<Flash<Redirect>, Template> {
    if !csrf.verify(form.csrf_token) {
        // CSRF失敗はリダイレクトさせる（フォーム再表示でもいいが、Flash使うならリダイレクト）
        return Ok(Flash::error(Redirect::to("/admin/users/create"), "CSRF検証に失敗しました"));
    }
    
    // Convert form to JSON Value
    let form_data = serde_json::to_value(form.into_inner()).unwrap();
    let view = UserCreateView;
    
    // Error case needs context (CSRF)
    let context = serde_json::json!({
        "active_nav": "users",
        "csrf_token": csrf.token(),
    });

    view.post(db, &form_data, context).await
}

pub struct UserUpdateView;

#[rocket::async_trait]
impl UpdateView<user::ActiveModel> for UserUpdateView {
    fn success_url(&self) -> String {
        "/admin/users".to_string()
    }

    async fn get_context_data(&self, db: &DatabaseConnection) -> serde_json::Value {
        let all_groups = Group::find().all(db).await.unwrap_or_default();
        serde_json::json!({
            "all_groups": all_groups
        })
    }

    async fn get_object(&self, db: &DatabaseConnection, id: i32) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(db).await
    }

    async fn save(&self, db: &DatabaseConnection, id: i32, data: &serde_json::Value) -> Result<user::Model, DbErr> {
         let existing = User::find_by_id(id).one(db).await?.ok_or(DbErr::Custom("NotFound".into()))?;
         let mut active_model: user::ActiveModel = existing.into();
         
         if let Some(u) = data["username"].as_str() {
             if !u.trim().is_empty() {
                 active_model.username = Set(u.to_owned());
             }
         }
         
         if let Some(p) = data["password"].as_str() {
             if !p.is_empty() {
                 let hash = hash_password(p).map_err(|e| DbErr::Custom(e.to_string()))?;
                 active_model.password_hash = Set(hash);
             }
         }
         
         active_model.is_admin = Set(data["is_admin"].as_bool().unwrap_or(false));
         active_model.is_active = Set(data["is_active"].as_bool().unwrap_or(false));
         
         let user = active_model.update(db).await.map_err(|e| {
             if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                  DbErr::Custom("このユーザー名は既に使用されています".into())
             } else {
                  e
             }
         })?;
         
         // Groups replacement
         group_user::Entity::delete_many().filter(group_user::Column::UserId.eq(id)).exec(db).await?;
         
         if let Some(groups) = data["group_ids"].as_array() {
             let mut relations = Vec::new();
             for g in groups {
                  if let Some(gid) = g.as_i64() {
                      relations.push(group_user::ActiveModel {
                          user_id: Set(id),
                          group_id: Set(gid as i32),
                          ..Default::default()
                      });
                  }
             }
             if !relations.is_empty() {
                 group_user::Entity::insert_many(relations).exec(db).await?;
             }
         }
         
         Ok(user)
    }
}

pub struct UserDeleteView;

#[rocket::async_trait]
impl DeleteView<User> for UserDeleteView {
    fn success_url(&self) -> String {
        "/admin/users".to_string()
    }
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
    let view = UserUpdateView;
    // ユーザーとグループ情報を取得して初期値として渡す必要がある
    // `view.get` は `get_object` を呼ぶが、それは `form` 初期値（Model）のため。
    // グループIDリスト（M2M）は Model に含まれないため、手動で取得して context に入れる
    
    // 現在のグループIDリストを取得
    let group_ids: Vec<i32> = GroupUser::find()
        .filter(group_user::Column::UserId.eq(id))
        .select_only()
        .column(group_user::Column::GroupId)
        .into_tuple()
        .all(db.inner())
        .await
        .unwrap_or_default();

    let context = serde_json::json!({
        "active_nav": "users",
        "csrf_token": csrf.token(),
        "user_group_ids": group_ids,
    });

    // view.get を呼ぶ。idを渡すことで、内部で get_object(id) が呼ばれる。
    // object引数は None でよい（内部で取得させる）。
    view.get(db, id, None, context).await
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
) -> Result<Flash<Redirect>, Template> {
    if !csrf.verify(form.csrf_token) {
        return Ok(Flash::error(Redirect::to(format!("/admin/users/edit/{}", id)), "CSRF検証に失敗しました"));
    }
    
    let form_data = serde_json::to_value(form.into_inner()).unwrap();
    let view = UserUpdateView;
    
    let context = serde_json::json!({
        "active_nav": "users",
        "csrf_token": csrf.token(),
    });

    view.post(db, id, &form_data, context).await
}

/// ユーザー削除処理 (POST)。
/// Djangoの `DeleteView` に相当。
#[post("/users/delete/<id>")]
pub async fn delete_user(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    id: i32,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let view = UserDeleteView;
    view.post(db, id).await
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
