#[macro_export]
macro_rules! impl_admin_resource {
    (
        entity: $entity:ty,
        active_model: $active_model:ty,
        form: $form:ty,
        view_prefix: $view_prefix:ident,
        base_url: $base_url:expr,
        template_dir: $template_dir:expr,
        search_field: $search_field:expr,
        order_field: $order_field:expr
    ) => {
        use rocket::form::Form;
        use rocket::response::{Flash, Redirect};
        use rocket::State;
        // use rocket_dyn_templates::{Template, context}; // Replaced by AppTemplate
        use rocket_dyn_templates::context; // Keep context macro if needed, or use serde_json
        use sea_orm::*;
        use crate::guards::auth::AdminUser;
        use crate::csrf::CsrfToken;
        use crate::views::list::ListView;
        use crate::views::edit::{CreateView, UpdateView, DeleteView};
        use crate::views::app_template::AppTemplate; // New
        use rocket::serde::json::serde_json;

        paste::paste! {
            // View Structs
            pub struct [<$view_prefix ListView>];
            pub struct [<$view_prefix CreateView>];
            pub struct [<$view_prefix UpdateView>];
            pub struct [<$view_prefix DeleteView>];

            #[rocket::async_trait]
            impl ListView<$entity> for [<$view_prefix ListView>] {
                fn template_name(&self) -> &'static str {
                    concat!($template_dir, "_list")
                }

                fn filter_queryset(&self, query: Select<$entity>, q: &str) -> Select<$entity> {
                    query.filter($search_field.contains(q))
                }

                fn get_context_data(&self, _db: &DatabaseConnection) -> serde_json::Value {
                    serde_json::json!({
                        "active_nav": $base_url.trim_start_matches("/admin/"),
                    })
                }
            }
            
            // List Handler
            #[get("/?<page>&<q>&<sort>&<dir>")]
            pub async fn list(
                db: &State<DatabaseConnection>,
                _admin: AdminUser,
                // csrf: CsrfToken, // Handled by AppTemplate
                page: Option<usize>,
                q: Option<String>,
                sort: Option<String>,
                dir: Option<String>,
            ) -> AppTemplate {
                let view = [<$view_prefix ListView>];
                let context = serde_json::json!({
                    // "csrf_token": csrf.token(), // Auto injected
                    "base_url": $base_url,
                });
                view.list(db, page.unwrap_or(1), q, sort, dir, context).await
            }


            // Create View Impl
            #[rocket::async_trait]
            impl CreateView<$active_model> for [<$view_prefix CreateView>] {
                fn success_url(&self) -> String {
                    $base_url.to_string()
                }
                
                async fn save(&self, db: &DatabaseConnection, data: &serde_json::Value) -> Result<<$entity as EntityTrait>::Model, DbErr> {
                     let form_struct: $form = serde_json::from_value(data.clone()).map_err(|e| DbErr::Custom(e.to_string()))?;
                     let active_model: $active_model = form_struct.into();
                     active_model.insert(db).await
                }
            }

            #[get("/create")]
            pub fn create_form(_admin: AdminUser) -> AppTemplate {
                AppTemplate::new(concat!($template_dir, "_form"), context! {
                    active_nav: $base_url.trim_start_matches("/admin/"),
                    // csrf_token: csrf.token(), // Injected
                    base_url: $base_url,
                })
            }

            #[post("/create", data = "<form>")]
            pub async fn create(
                db: &State<DatabaseConnection>,
                _admin: AdminUser,
                csrf: CsrfToken,
                form: Form<$form>,
            ) -> Result<Flash<Redirect>, Flash<Redirect>> {
                 if !csrf.verify(&form.csrf_token) {
                     return Err(Flash::error(Redirect::to(format!("{}/create", $base_url)), "CSRF検証に失敗しました"));
                 }
                 
                 let view = [<$view_prefix CreateView>];
                 let json_data = serde_json::to_value(form.into_inner()).unwrap();
                 
                 match view.save(db, &json_data).await {
                     Ok(_) => Ok(Flash::success(Redirect::to($base_url), "作成しました")),
                     Err(e) => {
                         if e.to_string().contains("duplicate") {
                             Err(Flash::error(Redirect::to(format!("{}/create", $base_url)), "既に使用されています"))
                         } else {
                             Err(Flash::error(Redirect::to(format!("{}/create", $base_url)), format!("作成に失敗しました: {}", e)))
                         }
                 }
                 }
            }
            
            // Update View
            #[rocket::async_trait]
            impl UpdateView<$active_model> for [<$view_prefix UpdateView>] {
                fn success_url(&self) -> String {
                    $base_url.to_string()
                }

                async fn get_object(&self, db: &DatabaseConnection, id: i32) -> Result<Option<<$entity as EntityTrait>::Model>, DbErr> {
                    <$entity>::find_by_id(id).one(db).await
                }

                async fn save(&self, db: &DatabaseConnection, id: i32, data: &serde_json::Value) -> Result<<$entity as EntityTrait>::Model, DbErr> {
                     // Check existence
                     let _existing = <$entity>::find_by_id(id).one(db).await?.ok_or(DbErr::Custom("NotFound".into()))?;
                     
                     let form_struct: $form = serde_json::from_value(data.clone()).map_err(|e| DbErr::Custom(e.to_string()))?;
                     let mut active_model: $active_model = form_struct.into();
                     
                     // Force ID to match route
                     active_model.id = Set(id);
                     
                     Ok(active_model.update(db).await?)
                }
            }
            
            #[get("/edit/<id>")]
            pub async fn edit_form(db: &State<DatabaseConnection>, _admin: AdminUser, id: i32) -> Result<AppTemplate, Flash<Redirect>> {
                let view = [<$view_prefix UpdateView>];
                let context = serde_json::json!({
                     // "csrf_token": csrf.token(),
                     "base_url": $base_url,
                     "active_nav": $base_url.trim_start_matches("/admin/"),
                });
                view.get(db, id, None, context).await
            }
            
            #[post("/edit/<id>", data = "<form>")]
            pub async fn edit(db: &State<DatabaseConnection>, _admin: AdminUser, csrf: CsrfToken, id: i32, form: Form<$form>) -> Result<Flash<Redirect>, Flash<Redirect>> {
                 if !csrf.verify(&form.csrf_token) {
                     return Err(Flash::error(Redirect::to(format!("{}/edit/{}", $base_url, id)), "CSRF検証に失敗しました"));
                 }
                 let view = [<$view_prefix UpdateView>];
                 let json_data = serde_json::to_value(form.into_inner()).unwrap();
                 
                 match view.save(db, id, &json_data).await {
                     Ok(_) => Ok(Flash::success(Redirect::to($base_url), "更新しました")),
                     Err(e) => {
                         Err(Flash::error(Redirect::to(format!("{}/edit/{}", $base_url, id)), format!("更新に失敗しました: {}", e)))
                     }
                 }
            }

            // Delete View
            #[rocket::async_trait]
            impl DeleteView<$entity> for [<$view_prefix DeleteView>] {
                fn success_url(&self) -> String {
                    $base_url.to_string()
                }
            }
            
            #[post("/delete/<id>")]
            pub async fn delete(db: &State<DatabaseConnection>, _admin: AdminUser, id: i32) -> Result<Flash<Redirect>, Flash<Redirect>> {
                let view = [<$view_prefix DeleteView>];
                view.post(db, id).await
            }

            pub fn routes() -> Vec<rocket::Route> {
                routes![
                    list,
                    create_form,
                    create,
                    edit_form,
                    edit,
                    delete
                ]
            }
        }
    };
}
