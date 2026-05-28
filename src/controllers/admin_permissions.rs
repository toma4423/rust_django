use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use crate::entities::{prelude::*, permission};
use sea_orm::{Set, DatabaseConnection};
use crate::impl_admin_resource;
use crate::views::list::AdminFilter;

#[derive(FromForm, Deserialize, Serialize)]
pub struct PermissionForm {
    pub name: String,
    pub codename: String,
    pub csrf_token: String,
}

impl Into<permission::ActiveModel> for PermissionForm {
    fn into(self) -> permission::ActiveModel {
        permission::ActiveModel {
            name: Set(self.name),
            codename: Set(self.codename),
            ..Default::default()
        }
    }
}

impl_admin_resource!(
    CUSTOM_LIST_VIEW:
    entity: Permission,
    active_model: permission::ActiveModel,
    form: PermissionForm,
    view_prefix: Permission,
    base_url: "/admin/permissions",
    fields: [
        serde_json::json!({
            "name": "name",
            "label": "名前",
            "type": "text",
            "required": true,
            "help_text": "表示名 (例: ユーザーを閲覧)"
        }),
        serde_json::json!({
            "name": "codename",
            "label": "コードネーム",
            "type": "text",
            "required": true,
            "help_text": "内部名 (例: auth.view_user)"
        })
    ]
);

pub struct PermissionListView;

#[rocket::async_trait]
impl crate::views::list::ListView<Permission> for PermissionListView {
    fn verbose_name(&self) -> &'static str { "権限" }
    fn verbose_name_plural(&self) -> &'static str { "権限" }
    fn list_display(&self) -> Vec<(&'static str, &'static str)> {
        vec![("id", "ID"), ("name", "名前"), ("codename", "コードネーム")]
    }
    fn search_fields(&self) -> Vec<&'static str> {
        vec!["name", "codename"]
    }
    fn list_filters(&self) -> Vec<&'static str> {
        vec!["codename"]
    }

    fn get_filters(&self, _db: &DatabaseConnection) -> Vec<AdminFilter> {
        vec![
            AdminFilter {
                label: "種別".into(),
                parameter_name: "codename".into(),
                choices: vec![
                    ("auth.view_".into(), "閲覧権限".into()),
                    ("auth.add_".into(), "追加権限".into()),
                    ("auth.change_".into(), "変更権限".into()),
                    ("auth.delete_".into(), "削除権限".into()),
                ]
            }
        ]
    }

    fn apply_filters(&self, mut query: sea_orm::Select<Permission>, params: &std::collections::HashMap<String, String>) -> sea_orm::Select<Permission> {
        use sea_orm::ColumnTrait;
        if let Some(codename) = params.get("codename") {
            if !codename.is_empty() {
                query = query.filter(permission::Column::Codename.contains(codename));
            }
        }
        query
    }
}
