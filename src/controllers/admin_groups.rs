// Imports required by the Form or other custom logic, but NOT ones imported by macro if they collide.
// Macro imports: Form, Flash, Redirect, State, Template, context, sea_orm, AdminUser, CsrfToken, ListView, CreateView..., serde_json.

// We need Deserialize/Serialize for the form struct itself.
use serde::{Deserialize, Serialize};
// We need sea_orm for the From impl.
use sea_orm::{Set, ActiveModelTrait}; 
// We need entities.
use crate::entities::{prelude::*, group};
// We need macro.
use crate::impl_admin_resource;

// グループフォーム
// Use owned String to avoid lifetime issues with generic macro usage
#[derive(FromForm, Deserialize, Serialize)]
pub struct GroupForm {
    pub name: String,
    #[field(default = "")]
    pub csrf_token: String,
}

// Convert Form to ActiveModel
impl From<GroupForm> for group::ActiveModel {
    fn from(form: GroupForm) -> Self {
        Self {
            name: Set(form.name),
            ..Default::default()
        }
    }
}

// Implement Admin Resource using Macro
impl_admin_resource! {
    entity: Group,
    active_model: group::ActiveModel,
    form: GroupForm, 
    view_prefix: Group,
    base_url: "/admin/groups",
    template_dir: "admin/group",
    search_field: group::Column::Name,
    order_field: group::Column::Id
}
