use rocket::form::Form;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket::serde::json::serde_json;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use crate::entities::{prelude::*, group, group_permission};
use crate::guards::auth::AdminUser;
use crate::csrf::CsrfToken;
use crate::views::list::ListView;
use crate::views::edit::{CreateView, UpdateView, DeleteView};
use crate::views::app_template::AppTemplate;

#[derive(FromForm, Deserialize, Serialize)]
pub struct GroupForm {
    pub name: String,
    #[field(default = Vec::new())]
    pub permission_ids: Vec<i32>,
    #[field(default = "")]
    pub csrf_token: String,
}

pub struct GroupListView;

impl ListView<Group> for GroupListView {
    fn template_name(&self) -> &'static str {
        "admin/group_list"
    }

    fn filter_queryset(&self, query: Select<Group>, q: &str) -> Select<Group> {
        query.filter(group::Column::Name.contains(q))
    }

    fn get_context_data(&self, _db: &DatabaseConnection) -> serde_json::Value {
        serde_json::json!({
            "active_nav": "groups",
        })
    }
}

#[get("/groups?<page>&<q>&<sort>&<dir>")]
pub async fn list_groups(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    page: Option<usize>,
    q: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
) -> AppTemplate {
    let view = GroupListView;
    let context = serde_json::json!({
        "base_url": "/admin/groups",
    });
    view.list(db, page.unwrap_or(1), q, sort, dir, &std::collections::HashMap::new(), context).await
}

pub struct GroupCreateView;

#[rocket::async_trait]
impl CreateView<group::ActiveModel> for GroupCreateView {
    fn success_url(&self) -> String {
        "/admin/groups".to_string()
    }

    async fn get_context_data(&self, db: &DatabaseConnection) -> serde_json::Value {
        let all_permissions = Permission::find().all(db).await.unwrap_or_default();
        serde_json::json!({
            "all_permissions": all_permissions
        })
    }
    
    async fn save(&self, db: &DatabaseConnection, data: &serde_json::Value) -> Result<group::Model, DbErr> {
         let name = data["name"].as_str().ok_or(DbErr::Custom("Name is required".into()))?;
         
         let active_model = group::ActiveModel {
            name: Set(name.to_owned()),
            ..Default::default()
         };

         let group = active_model.insert(db).await?;
         
         // Permission relations
         if let Some(perms) = data["permission_ids"].as_array() {
             let mut relations = Vec::new();
             for p in perms {
                  if let Some(pid) = p.as_i64() {
                      relations.push(group_permission::ActiveModel {
                          group_id: Set(group.id),
                          permission_id: Set(pid as i32),
                          ..Default::default()
                      });
                  }
             }
             if !relations.is_empty() {
                 group_permission::Entity::insert_many(relations).exec(db).await?;
             }
         }
         
         Ok(group)
    }
}

#[get("/groups/create")]
pub async fn create_group_form(db: &State<DatabaseConnection>, _admin: AdminUser) -> AppTemplate {
    let view = GroupCreateView;
    let context = serde_json::json!({
        "active_nav": "groups",
        "base_url": "/admin/groups",
    });
    view.get(db, context).await
}

#[post("/groups/create", data = "<form>")]
pub async fn create_group(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    form: Form<GroupForm>,
) -> Result<Flash<Redirect>, AppTemplate> {
    if !csrf.verify(&form.csrf_token) {
        return Ok(Flash::error(Redirect::to("/admin/groups/create"), "CSRF検証に失敗しました"));
    }
    
    let form_data = serde_json::to_value(form.into_inner()).unwrap();
    let view = GroupCreateView;
    let context = serde_json::json!({
        "active_nav": "groups",
    });

    view.post(db, &form_data, context).await
}

pub struct GroupUpdateView;

#[rocket::async_trait]
impl UpdateView<group::ActiveModel> for GroupUpdateView {
    fn success_url(&self) -> String {
        "/admin/groups".to_string()
    }

    async fn get_context_data(&self, db: &DatabaseConnection) -> serde_json::Value {
        let all_permissions = Permission::find().all(db).await.unwrap_or_default();
        serde_json::json!({
            "all_permissions": all_permissions
        })
    }

    async fn get_object(&self, db: &DatabaseConnection, id: i32) -> Result<Option<group::Model>, DbErr> {
        Group::find_by_id(id).one(db).await
    }

    async fn save(&self, db: &DatabaseConnection, id: i32, data: &serde_json::Value) -> Result<group::Model, DbErr> {
         let existing = Group::find_by_id(id).one(db).await?.ok_or(DbErr::Custom("NotFound".into()))?;
         let mut active_model: group::ActiveModel = existing.into();
         
         if let Some(n) = data["name"].as_str() {
             active_model.name = Set(n.to_owned());
         }
         
         let group = active_model.update(db).await?;
         
         // Permission replacement
         group_permission::Entity::delete_many()
            .filter(group_permission::Column::GroupId.eq(id))
            .exec(db).await?;
         
         if let Some(perms) = data["permission_ids"].as_array() {
             let mut relations = Vec::new();
             for p in perms {
                  if let Some(pid) = p.as_i64() {
                      relations.push(group_permission::ActiveModel {
                          group_id: Set(id),
                          permission_id: Set(pid as i32),
                          ..Default::default()
                      });
                  }
             }
             if !relations.is_empty() {
                 group_permission::Entity::insert_many(relations).exec(db).await?;
             }
         }
         
         Ok(group)
    }
}

#[get("/groups/edit/<id>")]
pub async fn edit_group_form(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    id: i32,
) -> Result<AppTemplate, Flash<Redirect>> {
    let view = GroupUpdateView;
    
    // Get assigned permissions
    let permission_ids: Vec<i32> = GroupPermission::find()
        .filter(group_permission::Column::GroupId.eq(id))
        .select_only()
        .column(group_permission::Column::PermissionId)
        .into_tuple()
        .all(db.inner())
        .await
        .unwrap_or_default();

    let context = serde_json::json!({
        "active_nav": "groups",
        "group_permission_ids": permission_ids,
        "base_url": "/admin/groups",
    });

    view.get(db, id, None, context).await
}

#[post("/groups/edit/<id>", data = "<form>")]
pub async fn edit_group(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    id: i32,
    form: Form<GroupForm>,
) -> Result<Flash<Redirect>, AppTemplate> {
    if !csrf.verify(&form.csrf_token) {
        return Ok(Flash::error(Redirect::to(format!("/admin/groups/edit/{}", id)), "CSRF検証に失敗しました"));
    }
    
    let form_data = serde_json::to_value(form.into_inner()).unwrap();
    let view = GroupUpdateView;
    let context = serde_json::json!({
        "active_nav": "groups",
    });

    view.post(db, id, &form_data, context).await
}

pub struct GroupDeleteView;

#[rocket::async_trait]
impl DeleteView<Group> for GroupDeleteView {
    fn success_url(&self) -> String {
        "/admin/groups".to_string()
    }
}

#[post("/groups/delete/<id>")]
pub async fn delete_group(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    id: i32,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let view = GroupDeleteView;
    view.post(db, id).await
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        list_groups,
        create_group_form,
        create_group,
        edit_group_form,
        edit_group,
        delete_group
    ]
}
