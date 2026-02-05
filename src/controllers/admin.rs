use rocket::State;
use rocket_dyn_templates::context;
use crate::guards::auth::AdminUser;
use crate::views::app_template::AppTemplate;

#[get("/")]
pub fn dashboard(_admin: AdminUser) -> AppTemplate {
    AppTemplate::new("admin/dashboard", context! {
        active_nav: "dashboard",
    })
}




pub fn routes() -> Vec<rocket::Route> {
    routes![
        dashboard,
    ]
}

