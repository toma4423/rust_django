use rocket::form::Form;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::context;
use rocket::serde::json::serde_json;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use crate::entities::{prelude::*, user, group_user, group};
use crate::guards::auth::AdminUser;
use crate::auth_utils::hash_password;
use crate::csrf::CsrfToken;
use crate::views::list::ListView;
use crate::views::edit::{CreateView, UpdateView, DeleteView};
use crate::views::app_template::AppTemplate;

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
pub fn dashboard(_admin: AdminUser) -> AppTemplate {
    AppTemplate::new("admin/dashboard", context! {
        active_nav: "dashboard",
    })
}

pub struct UserListView;

impl ListView<User> for UserListView {
    fn template_name(&self) -> &'static str {
        "admin/list"
    }

    fn filter_queryset(&self, query: Select<User>, q: &str) -> Select<User> {
         query.filter(user::Column::Username.contains(q))
    }

    fn get_context_data(&self, _db: &DatabaseConnection) -> serde_json::Value {
        serde_json::json!({
            "active_nav": "users",
        })
    }
}

#[derive(Serialize)]
struct UserWithGroups {
    user: user::Model,
    groups: Vec<group::Model>,
}

/// ユーザー一覧を表示する管理画面。
/// Generic View (`ListView`) を使用せず、グループ情報を取得するためにカスタム実装。
#[get("/users?<page>&<q>&<sort>&<dir>&<is_active>&<is_admin>")]
pub async fn list_users(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    _csrf: CsrfToken,
    page: Option<usize>,
    q: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    is_active: Option<bool>,
    is_admin: Option<bool>,
) -> AppTemplate {
    let page = if page.unwrap_or(1) < 1 { 1 } else { page.unwrap_or(1) };
    let per_page = 10;

    // 1. クエリ構築
    let mut query = User::find();

    // 2. 検索適用
    let search_query = q.clone().unwrap_or_default();
    if !search_query.trim().is_empty() {
         query = query.filter(user::Column::Username.contains(&search_query));
    }

    // 3. フィルタ適用
    if let Some(active) = is_active {
        query = query.filter(user::Column::IsActive.eq(active));
    }
    if let Some(admin) = is_admin {
        query = query.filter(user::Column::IsAdmin.eq(admin));
    }

    // 4. ソート適用
    let sort_col = sort.clone().unwrap_or_else(|| "id".to_string());
    let direction = dir.clone().unwrap_or_else(|| "desc".to_string());
    
    let order = if direction.to_lowercase() == "asc" {
        Order::Asc
    } else {
        Order::Desc
    };

    match sort_col.as_str() {
        "username" => query = query.order_by(user::Column::Username, order),
        "is_active" => query = query.order_by(user::Column::IsActive, order),
        "is_admin" => query = query.order_by(user::Column::IsAdmin, order),
        _ => query = query.order_by(user::Column::Id, order), // Default to ID
    }

    // 5. ページネーション (Userのみ)
    let paginator = query.paginate(db.inner(), per_page);
    let num_pages = paginator.num_pages().await.unwrap_or(0);
    let users = paginator.fetch_page((page - 1) as u64).await.unwrap_or_default();

    // 6. グループ情報の取得
    let mut items = Vec::new();
    for u in users {
        let groups: Vec<group::Model> = u.find_related(Group).all(db.inner()).await.unwrap_or_default();
        items.push(UserWithGroups {
            user: u,
            groups,
        });
    }

    // フィルタ定義と現在の状態
    let filters = serde_json::json!([
        {
            "label": "アクティブ",
            "parameter_name": "is_active",
            "current_value": is_active.map(|b| b.to_string()).unwrap_or_default(),
            "choices": [
                ["true", "はい"],
                ["false", "いいえ"]
            ]
        },
        {
            "label": "スタッフ",
            "parameter_name": "is_admin",
            "current_value": is_admin.map(|b| b.to_string()).unwrap_or_default(),
            "choices": [
                ["true", "はい"],
                ["false", "いいえ"]
            ]
        }
    ]);

    // 7. コンテキスト構築
    AppTemplate::new("admin/list", context! {
        items: items,
        current_page: page,
        num_pages: num_pages,
        search_query: search_query,
        sort: sort_col,
        dir: direction,
        base_url: "/admin/users",
        active_nav: "users",
        admin_filters: filters,
    })
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
pub async fn create_user_form(db: &State<DatabaseConnection>, _admin: AdminUser, _csrf: CsrfToken) -> AppTemplate {
    let view = UserCreateView;
    let context = serde_json::json!({
        "active_nav": "users",
        "base_url": "/admin/users",
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
) -> Result<Flash<Redirect>, AppTemplate> {
    if !csrf.verify(form.csrf_token) {
        return Ok(Flash::error(Redirect::to("/admin/users/create"), "CSRF検証に失敗しました"));
    }
    
    // Convert form to JSON Value
    let form_data = serde_json::to_value(form.into_inner()).unwrap();
    let view = UserCreateView;
    
    // Error case needs context
    let context = serde_json::json!({
        "active_nav": "users",
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
    _csrf: CsrfToken,
    id: i32,
) -> Result<AppTemplate, Flash<Redirect>> {
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
        "user_group_ids": group_ids,
        "base_url": "/admin/users",
    });

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
) -> Result<Flash<Redirect>, AppTemplate> {
    if !csrf.verify(form.csrf_token) {
        return Ok(Flash::error(Redirect::to(format!("/admin/users/edit/{}", id)), "CSRF検証に失敗しました"));
    }
    
    let form_data = serde_json::to_value(form.into_inner()).unwrap();
    let view = UserUpdateView;
    
    let context = serde_json::json!({
        "active_nav": "users",
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

#[derive(FromForm)]
pub struct UserActionForm {
    pub action: String,
    pub selected_ids: Vec<i32>,
    pub csrf_token: String,
}

/// ユーザー一括操作 (POST)。
#[post("/users/action", data = "<form>")]
pub async fn user_action(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    form: Form<UserActionForm>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    if !csrf.verify(&form.csrf_token) {
        return Err(Flash::error(Redirect::to("/admin/users"), "CSRF検証に失敗しました"));
    }

    match form.action.as_str() {
        "delete_selected" => {
            if form.selected_ids.is_empty() {
                return Ok(Flash::warning(Redirect::to("/admin/users"), "ユーザーが選択されていません"));
            }
            
            let result = User::delete_many()
                .filter(user::Column::Id.is_in(form.selected_ids.clone()))
                .exec(db.inner())
                .await;

            match result {
                Ok(res) => Ok(Flash::success(Redirect::to("/admin/users"), format!("{} 件のユーザーを削除しました", res.rows_affected))),
                Err(e) => Err(Flash::error(Redirect::to("/admin/users"), format!("削除に失敗しました: {}", e))),
            }
        }
        _ => Ok(Flash::warning(Redirect::to("/admin/users"), "不明な操作です")),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        dashboard,
        list_users,
        create_user_form,
        create_user,
        edit_user_form,
        edit_user,
        delete_user,
        user_action
    ]
}
