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
use crate::impl_admin_resource; // Import the macro

// Form Structs
#[derive(FromForm, Deserialize, Serialize)]
pub struct UserForm<'r> {
    pub username: &'r str,
    pub password: &'r str,
    #[field(default = false)]
    pub is_admin: bool,
    #[field(default = false)]
    pub is_active: bool,
    #[field(default = Vec::new())]
    pub group_ids: Vec<i32>,
    #[field(default = "")]
    pub csrf_token: &'r str,
}

#[derive(FromForm, Deserialize, Serialize)]
pub struct UserActionForm {
    pub action: String,
    pub selected_ids: Vec<i32>,
    pub csrf_token: String,
}

#[derive(Serialize)]
struct UserWithGroups {
    user: user::Model,
    groups: Vec<group::Model>,
}

// Macro Wrapper
impl_admin_resource! {
    entity: User,
    active_model: user::ActiveModel,
    form: UserForm<'static>, // Lifetime? Form uses 'r. Macro expects type.
    view_prefix: User,
    base_url: "/admin/users",
    template_dir: "admin/user", // assuming templates are admin/user_list, etc? 
                                // Wait, templates are `admin/list.html.tera` currently.
                                // If I use generic template name `admin/list`, I should specify that.
                                // But macro uses `concat!($template_dir, "_list")` -> `admin/user_list`.
                                // Existing template is `admin/list`.
                                // So I should probably use `admin/user` and rename template?
                                // OR use `admin` and rely on `_list`? No `admin_list`.
                                // Currently `admin/list.html.tera` is the generic User list.
                                // I should rename `admin/list.html.tera` to `admin/user_list.html.tera` or similar to follow convention?
                                // OR patch macro to allow exact template name.
                                // For now, I will use manual implementation for List anyway, so I can stick to `admin/list`.
                                // For Create/Update, existing are likely `admin/user_form`? 
                                // Let's check templates.
    search_field: user::Column::Username,
    order_field: user::Column::Id, // Default
    skip_list: true,
    skip_create: true,
    skip_update: true
}

// Manual Implementations

// LIST VIEW
impl ListView<User> for UserListView {
    fn template_name(&self) -> &'static str {
        "admin/list" // Generic user list template
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

#[get("/?<page>&<q>&<sort>&<dir>&<is_active>&<is_admin>")]
pub async fn list(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    page: Option<usize>,
    q: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    is_active: Option<bool>,
    is_admin: Option<bool>,
) -> AppTemplate {
    // Copied logic from admin.rs
    let page = if page.unwrap_or(1) < 1 { 1 } else { page.unwrap_or(1) };
    let per_page = 10;
    
    let mut query = User::find();
    let search_query = q.clone().unwrap_or_default();
    if !search_query.trim().is_empty() {
         query = query.filter(user::Column::Username.contains(&search_query));
    }
    if let Some(active) = is_active {
        query = query.filter(user::Column::IsActive.eq(active));
    }
    if let Some(admin) = is_admin {
        query = query.filter(user::Column::IsAdmin.eq(admin));
    }

    let sort_col = sort.clone().unwrap_or_else(|| "id".to_string());
    let direction = dir.clone().unwrap_or_else(|| "desc".to_string());
    let order = if direction.to_lowercase() == "asc" { Order::Asc } else { Order::Desc };

    match sort_col.as_str() {
        "username" => query = query.order_by(user::Column::Username, order),
        "is_active" => query = query.order_by(user::Column::IsActive, order),
        "is_admin" => query = query.order_by(user::Column::IsAdmin, order),
        _ => query = query.order_by(user::Column::Id, order),
    }

    let paginator = query.paginate(db.inner(), per_page);
    let num_pages = paginator.num_pages().await.unwrap_or(0);
    let users = paginator.fetch_page((page - 1) as u64).await.unwrap_or_default();

    let mut items = Vec::new();
    for u in users {
        let groups: Vec<group::Model> = u.find_related(Group).all(db.inner()).await.unwrap_or_default();
        items.push(UserWithGroups { user: u, groups });
    }

    let filters = serde_json::json!([
        { "label": "アクティブ", "parameter_name": "is_active", "current_value": is_active.map(|b| b.to_string()).unwrap_or_default(), "choices": [["true", "はい"], ["false", "いいえ"]] },
        { "label": "スタッフ", "parameter_name": "is_admin", "current_value": is_admin.map(|b| b.to_string()).unwrap_or_default(), "choices": [["true", "はい"], ["false", "いいえ"]] }
    ]);

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

// CREATE VIEW
#[rocket::async_trait]
impl CreateView<user::ActiveModel> for UserCreateView {
    fn success_url(&self) -> String { "/admin/users".to_string() }

    async fn get_context_data(&self, db: &DatabaseConnection) -> serde_json::Value {
        let all_groups = Group::find().all(db).await.unwrap_or_default();
        serde_json::json!({ "all_groups": all_groups })
    }
    
    async fn save(&self, db: &DatabaseConnection, data: &serde_json::Value) -> Result<user::Model, DbErr> {
         let username = data["username"].as_str().ok_or(DbErr::Custom("ユーザー名は必須です".into()))?;
         let password = data["password"].as_str().unwrap_or("");
         if username.trim().is_empty() { return Err(DbErr::Custom("ユーザー名は必須です".into())); }
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
            println!("User Insert Error: {:?}", e);
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                DbErr::Custom("このユーザー名は既に使用されています".into())
            } else { e }
         })?;
         
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
             if !relations.is_empty() { group_user::Entity::insert_many(relations).exec(db).await?; }
         }
         Ok(user)
    }
}

#[get("/create")]
pub async fn create_form(db: &State<DatabaseConnection>, _admin: AdminUser) -> AppTemplate {
    let view = UserCreateView;
    let context = serde_json::json!({ "active_nav": "users", "base_url": "/admin/users" });
    view.get(db, context).await
}

#[post("/create", data = "<form>")]
pub async fn create(db: &State<DatabaseConnection>, _admin: AdminUser, csrf: CsrfToken, form: Form<UserForm<'_>>) -> Result<Flash<Redirect>, AppTemplate> {
    if !csrf.verify(form.csrf_token) { return Ok(Flash::error(Redirect::to("/admin/users/create"), "CSRF検証に失敗しました")); }
    let form_data = serde_json::to_value(form.into_inner()).unwrap();
    let view = UserCreateView;
    let context = serde_json::json!({ "active_nav": "users" });
    view.post(db, &form_data, context).await
}

// UPDATE VIEW
#[rocket::async_trait]
impl UpdateView<user::ActiveModel> for UserUpdateView {
    fn success_url(&self) -> String { "/admin/users".to_string() }

    async fn get_context_data(&self, db: &DatabaseConnection) -> serde_json::Value {
        let all_groups = Group::find().all(db).await.unwrap_or_default();
        serde_json::json!({ "all_groups": all_groups })
    }

    async fn get_object(&self, db: &DatabaseConnection, id: i32) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(db).await
    }

    async fn save(&self, db: &DatabaseConnection, id: i32, data: &serde_json::Value) -> Result<user::Model, DbErr> {
         let existing = User::find_by_id(id).one(db).await?.ok_or(DbErr::Custom("NotFound".into()))?;
         let mut active_model: user::ActiveModel = existing.into();
         if let Some(u) = data["username"].as_str() {
             if !u.trim().is_empty() { active_model.username = Set(u.to_owned()); }
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
             } else { e }
         })?;
         
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
             if !relations.is_empty() { group_user::Entity::insert_many(relations).exec(db).await?; }
         }
         Ok(user)
    }
}

#[get("/edit/<id>")]
pub async fn edit_form(db: &State<DatabaseConnection>, _admin: AdminUser, id: i32) -> Result<AppTemplate, Flash<Redirect>> {
    let view = UserUpdateView;
    let group_ids: Vec<i32> = GroupUser::find().filter(group_user::Column::UserId.eq(id)).select_only().column(group_user::Column::GroupId).into_tuple().all(db.inner()).await.unwrap_or_default();
    let context = serde_json::json!({ "active_nav": "users", "user_group_ids": group_ids, "base_url": "/admin/users" });
    view.get(db, id, None, context).await
}

#[post("/edit/<id>", data = "<form>")]
pub async fn edit(db: &State<DatabaseConnection>, _admin: AdminUser, csrf: CsrfToken, id: i32, form: Form<UserForm<'_>>) -> Result<Flash<Redirect>, AppTemplate> {
    if !csrf.verify(form.csrf_token) { return Ok(Flash::error(Redirect::to(format!("/admin/users/edit/{}", id)), "CSRF検証に失敗しました")); }
    let form_data = serde_json::to_value(form.into_inner()).unwrap();
    let view = UserUpdateView;
    let context = serde_json::json!({ "active_nav": "users" });
    view.post(db, id, &form_data, context).await
}

// EXTRA ACTIONS (Not covered by macro)
#[post("/action", data = "<form>")]
pub async fn user_action(db: &State<DatabaseConnection>, _admin: AdminUser, csrf: CsrfToken, form: Form<UserActionForm>) -> Result<Flash<Redirect>, Flash<Redirect>> {
    if !csrf.verify(&form.csrf_token) { return Err(Flash::error(Redirect::to("/admin/users"), "CSRF検証に失敗しました")); }
    match form.action.as_str() {
        "delete_selected" => {
            if form.selected_ids.is_empty() { return Ok(Flash::warning(Redirect::to("/admin/users"), "ユーザーが選択されていません")); }
            let result = User::delete_many().filter(user::Column::Id.is_in(form.selected_ids.clone())).exec(db.inner()).await;
            match result {
                Ok(res) => Ok(Flash::success(Redirect::to("/admin/users"), format!("{} 件のユーザーを削除しました", res.rows_affected))),
                Err(e) => Err(Flash::error(Redirect::to("/admin/users"), format!("削除に失敗しました: {}", e))),
            }
        }
        _ => Ok(Flash::warning(Redirect::to("/admin/users"), "不明な操作です")),
    }
}

// routes() is generated by macro for generic actions, but we need to verify if it works with custom implementations if names match.
// Also need to add `user_action` to routes if possible, or append it.
// The macro generates `routes()` returning specific list. `user_action` is NOT in it.
// So we need to expose a combined route list.

pub fn all_routes() -> Vec<rocket::Route> {
    let mut r = routes(); // Generated by macro
    r.push(routes![user_action][0].clone());
    r
}
