use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use crate::entities::{prelude::*, group};
use sea_orm::Set;
use crate::impl_admin_resource;

#[derive(FromForm, Deserialize, Serialize)]
pub struct GroupForm {
    pub name: String,
    pub csrf_token: String,
}

impl From<GroupForm> for group::ActiveModel {
    fn from(form: GroupForm) -> Self {
        group::ActiveModel {
            name: Set(form.name),
            ..Default::default()
        }
    }
}

impl_admin_resource!(
    entity: Group,
    active_model: group::ActiveModel,
    form: GroupForm,
    view_prefix: Group,
    base_url: "/admin/groups",
    verbose_name: "グループ",
    verbose_name_plural: "グループ",
    list_display: [("id", "ID"), ("name", "名前")],
    search_fields: ["name"],
    list_filter: [],
    fields: [
        serde_json::json!({
            "name": "name",
            "label": "名前",
            "type": "text",
            "required": true,
            "help_text": "グループ名"
        })
    ]
);
