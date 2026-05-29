use rocket::serde::json::serde_json;
use serde::{Deserialize, Serialize};
use crate::entities::{prelude::*, todo};
use sea_orm::Set;
use crate::impl_admin_resource;

#[derive(FromForm, Deserialize, Serialize)]
pub struct TodoAdminForm {
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub priority: i32,
    pub user_id: i32,
    pub group_id: Option<i32>,
    pub csrf_token: String,
}

impl From<TodoAdminForm> for todo::ActiveModel {
    fn from(form: TodoAdminForm) -> Self {
        todo::ActiveModel {
            title: Set(form.title),
            description: Set(form.description),
            completed: Set(form.completed),
            priority: Set(form.priority),
            user_id: Set(form.user_id),
            group_id: Set(form.group_id),
            ..Default::default()
        }
    }
}

impl_admin_resource!(
    entity: Todo,
    active_model: todo::ActiveModel,
    form: TodoAdminForm,
    view_prefix: Todo,
    base_url: "/admin/todos",
    verbose_name: "TODO",
    verbose_name_plural: "TODO",
    list_display: [("id", "ID"), ("title", "タイトル"), ("completed", "完了"), ("priority", "優先度")],
    search_fields: ["title", "description"],
    list_filter: [("completed", "完了"), ("priority", "優先度")],
    fields: [
        serde_json::json!({
            "name": "title",
            "label": "タイトル",
            "type": "text",
            "required": true,
            "help_text": "TODOの内容"
        }),
        serde_json::json!({
            "name": "description",
            "label": "詳細",
            "type": "text",
            "required": false,
            "help_text": "詳細な説明"
        }),
        serde_json::json!({
            "name": "completed",
            "label": "完了済み",
            "type": "checkbox",
            "required": false
        }),
        serde_json::json!({
            "name": "priority",
            "label": "優先度",
            "type": "select",
            "required": true,
            "choices": [
                ["1", "低"],
                ["2", "中"],
                ["3", "高"]
            ]
        }),
        serde_json::json!({
            "name": "user_id",
            "label": "ユーザーID",
            "type": "number",
            "required": true
        }),
        serde_json::json!({
            "name": "group_id",
            "label": "グループID",
            "type": "number",
            "required": false
        })
    ]
);
