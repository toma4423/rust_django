use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use crate::entities::{prelude::*, permission};
use sea_orm::Set;
use crate::impl_admin_resource;

#[derive(FromForm, Deserialize, Serialize)]
pub struct PermissionForm {
    pub name: String,
    pub codename: String,
    pub csrf_token: String,
}

impl From<PermissionForm> for permission::ActiveModel {
    fn from(form: PermissionForm) -> Self {
        permission::ActiveModel {
            name: Set(form.name),
            codename: Set(form.codename),
            ..Default::default()
        }
    }
}

impl_admin_resource!(
    entity: Permission,
    active_model: permission::ActiveModel,
    form: PermissionForm,
    view_prefix: Permission,
    base_url: "/admin/permissions",
    verbose_name: "権限",
    verbose_name_plural: "権限",
    list_display: [("id", "ID"), ("name", "名前"), ("codename", "コードネーム")],
    list_filter: [],
    search_fields: ["name", "codename"],
    list_filter: [],
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
