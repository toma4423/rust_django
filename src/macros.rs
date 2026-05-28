#[macro_export]
macro_rules! impl_admin_resource {
    (
        entity: $entity:ty,
        active_model: $active_model:ty,
        form: $form:ty,
        view_prefix: $view_prefix:ident,
        base_url: $base_url:expr,
        verbose_name: $verbose_name:expr,
        verbose_name_plural: $verbose_name_plural:expr,
        list_display: [ $(($list_col:expr, $list_label:expr)),* ],
        search_fields: [ $($search_field:expr),* ],
        fields: [ $($field_meta:expr),* ]
    ) => {
        paste::paste! {
            use sea_orm::*;

            // View Structs
            pub struct [<$view_prefix ListView>];
            pub struct [<$view_prefix CreateView>];
            pub struct [<$view_prefix UpdateView>];
            pub struct [<$view_prefix DeleteView>];

            #[rocket::async_trait]
            impl crate::views::list::ListView<$entity> for [<$view_prefix ListView>] {
                fn verbose_name(&self) -> &'static str {
                    $verbose_name
                }

                fn verbose_name_plural(&self) -> &'static str {
                    $verbose_name_plural
                }

                fn list_display(&self) -> Vec<(&'static str, &'static str)> {
                    vec![ $(($list_col, $list_label)),* ]
                }

                fn search_fields(&self) -> Vec<&'static str> {
                    vec![ $($search_field),* ]
                }

                fn filter_queryset(&self, mut query: Select<$entity>, q: &str) -> Select<$entity> {
                    let mut condition = sea_orm::sea_query::Condition::any();
                    for field in self.search_fields() {
                        for col in <$entity as EntityTrait>::Column::iter() {
                            if col.as_str() == field {
                                condition = condition.add(col.contains(q));
                            }
                        }
                    }
                    query.filter(condition)
                }

                fn get_context_data(&self, _db: &DatabaseConnection) -> rocket::serde::json::serde_json::Value {
                    rocket::serde::json::serde_json::json!({
                        "active_nav": $base_url.trim_start_matches("/admin/"),
                    })
                }
            }
            
            // List Handler
            #[get("/?<page>&<q>&<sort>&<dir>")]
            pub async fn list(
                db: &rocket::State<DatabaseConnection>,
                _admin: crate::guards::auth::AdminUser,
                page: Option<usize>,
                q: Option<String>,
                sort: Option<String>,
                dir: Option<String>,
            ) -> crate::views::app_template::AppTemplate {
                use crate::views::list::ListView;
                let view = [<$view_prefix ListView>];
                let context = rocket::serde::json::serde_json::json!({
                    "base_url": $base_url,
                });
                view.list(db, page.unwrap_or(1), q, sort, dir, &std::collections::HashMap::new(), context).await
            }


            // Create View Impl
            #[rocket::async_trait]
            impl crate::views::edit::CreateView<$active_model> for [<$view_prefix CreateView>] {
                fn verbose_name(&self) -> &'static str { $verbose_name }
                fn verbose_name_plural(&self) -> &'static str { $verbose_name_plural }
                fn fields(&self) -> Vec<rocket::serde::json::serde_json::Value> {
                    vec![ $($field_meta),* ]
                }

                fn success_url(&self) -> String {
                    $base_url.to_string()
                }
                
                async fn save(&self, db: &DatabaseConnection, data: &rocket::serde::json::serde_json::Value) -> Result<<$entity as EntityTrait>::Model, DbErr> {
                     let form_struct: $form = rocket::serde::json::serde_json::from_value(data.clone()).map_err(|e| DbErr::Custom(e.to_string()))?;
                     let active_model: $active_model = form_struct.into();
                     active_model.insert(db).await
                }
            }

            #[get("/create")]
            pub async fn create_form(db: &rocket::State<DatabaseConnection>, _admin: crate::guards::auth::AdminUser) -> crate::views::app_template::AppTemplate {
                use crate::views::edit::CreateView;
                let view = [<$view_prefix CreateView>];
                let context = rocket::serde::json::serde_json::json!({
                    "active_nav": $base_url.trim_start_matches("/admin/"),
                    "base_url": $base_url,
                });
                view.get(db, context).await
            }

            #[post("/create", data = "<form>")]
            pub async fn create(
                db: &rocket::State<DatabaseConnection>,
                _admin: crate::guards::auth::AdminUser,
                csrf: crate::csrf::CsrfToken,
                form: rocket::form::Form<$form>,
            ) -> Result<rocket::response::Flash<rocket::response::Redirect>, rocket::response::Flash<rocket::response::Redirect>> {
                 use rocket::response::{Flash, Redirect};
                 use crate::views::edit::CreateView;
                 if !csrf.verify(&form.csrf_token) {
                     return Err(Flash::error(Redirect::to(format!("{}/create", $base_url)), "CSRF検証に失敗しました"));
                 }
                 
                 let view = [<$view_prefix CreateView>];
                 let json_data = rocket::serde::json::serde_json::to_value(form.into_inner()).unwrap();
                 
                 match view.save(db, &json_data).await {
                     Ok(_) => Ok(Flash::success(Redirect::to($base_url), "作成しました")),
                     Err(e) => {
                         let e_str = e.to_string();
                         if e_str.contains("duplicate") {
                             Err(Flash::error(Redirect::to(format!("{}/create", $base_url)), "既に使用されています"))
                         } else {
                             Err(Flash::error(Redirect::to(format!("{}/create", $base_url)), format!("作成に失敗しました: {}", e_str)))
                         }
                 }
                 }
            }
            
            // Update View
            #[rocket::async_trait]
            impl crate::views::edit::UpdateView<$active_model> for [<$view_prefix UpdateView>] {
                fn verbose_name(&self) -> &'static str { $verbose_name }
                fn verbose_name_plural(&self) -> &'static str { $verbose_name_plural }
                fn fields(&self) -> Vec<rocket::serde::json::serde_json::Value> {
                    vec![ $($field_meta),* ]
                }

                fn success_url(&self) -> String {
                    $base_url.to_string()
                }

                async fn get_object(&self, db: &DatabaseConnection, id: i32) -> Result<Option<<$entity as EntityTrait>::Model>, DbErr> {
                    <$entity>::find_by_id(id).one(db).await
                }

                async fn save(&self, db: &DatabaseConnection, id: i32, data: &rocket::serde::json::serde_json::Value) -> Result<<$entity as EntityTrait>::Model, DbErr> {
                     // Check existence
                     let _existing = <$entity>::find_by_id(id).one(db).await?.ok_or(DbErr::Custom("NotFound".into()))?;
                     
                     let form_struct: $form = rocket::serde::json::serde_json::from_value(data.clone()).map_err(|e| DbErr::Custom(e.to_string()))?;
                     let mut active_model: $active_model = form_struct.into();
                     
                     // Force ID to match route
                     active_model.id = Set(id);
                     
                     Ok(active_model.update(db).await?)
                }
            }
            
            #[get("/edit/<id>")]
            pub async fn edit_form(db: &rocket::State<DatabaseConnection>, _admin: crate::guards::auth::AdminUser, id: i32) -> Result<crate::views::app_template::AppTemplate, rocket::response::Flash<rocket::response::Redirect>> {
                use crate::views::edit::UpdateView;
                let view = [<$view_prefix UpdateView>];
                let context = rocket::serde::json::serde_json::json!({
                     "base_url": $base_url,
                     "active_nav": $base_url.trim_start_matches("/admin/"),
                });
                view.get(db, id, None, context).await
            }
            
            #[post("/edit/<id>", data = "<form>")]
            pub async fn edit(db: &rocket::State<DatabaseConnection>, _admin: crate::guards::auth::AdminUser, csrf: crate::csrf::CsrfToken, id: i32, form: rocket::form::Form<$form>) -> Result<rocket::response::Flash<rocket::response::Redirect>, rocket::response::Flash<rocket::response::Redirect>> {
                 use rocket::response::{Flash, Redirect};
                 use crate::views::edit::UpdateView;
                 if !csrf.verify(&form.csrf_token) {
                     return Err(Flash::error(Redirect::to(format!("{}/edit/{}", $base_url, id)), "CSRF検証に失敗しました"));
                 }
                 let view = [<$view_prefix UpdateView>];
                 let json_data = rocket::serde::json::serde_json::to_value(form.into_inner()).unwrap();
                 
                 match view.save(db, id, &json_data).await {
                     Ok(_) => Ok(Flash::success(Redirect::to($base_url), "更新しました")),
                     Err(e) => {
                         Err(Flash::error(Redirect::to(format!("{}/edit/{}", $base_url, id)), format!("更新に失敗しました: {}", e)))
                     }
                 }
            }

            // Delete View
            #[rocket::async_trait]
            impl crate::views::edit::DeleteView<$entity> for [<$view_prefix DeleteView>] {
                fn success_url(&self) -> String {
                    $base_url.to_string()
                }
            }
            
            #[post("/delete/<id>")]
            pub async fn delete(db: &rocket::State<DatabaseConnection>, _admin: crate::guards::auth::AdminUser, id: i32) -> Result<rocket::response::Flash<rocket::response::Redirect>, rocket::response::Flash<rocket::response::Redirect>> {
                use crate::views::edit::DeleteView;
                let view = [<$view_prefix DeleteView>];
                view.post(db, id).await
            }

            #[post("/action", data = "<form>")]
            pub async fn action(db: &rocket::State<DatabaseConnection>, _admin: crate::guards::auth::AdminUser, csrf: crate::csrf::CsrfToken, form: rocket::form::Form<crate::controllers::admin::UserActionForm>) -> Result<rocket::response::Flash<rocket::response::Redirect>, rocket::response::Flash<rocket::response::Redirect>> {
                use rocket::response::{Flash, Redirect};
                if !csrf.verify(&form.csrf_token) {
                    return Err(Flash::error(Redirect::to($base_url), "CSRF検証に失敗しました"));
                }
                match form.action.as_str() {
                    "delete_selected" => {
                        let result = <$entity>::delete_many().filter(<$entity as EntityTrait>::Column::Id.is_in(form.selected_ids.clone())).exec(db.inner()).await;
                        match result {
                            Ok(res) => Ok(Flash::success(Redirect::to($base_url), format!("{} 件削除しました", res.rows_affected))),
                            Err(e) => Err(Flash::error(Redirect::to($base_url), format!("失敗: {}", e))),
                        }
                    }
                    _ => Ok(Flash::warning(Redirect::to($base_url), "不明なアクション")),
                }
            }

            pub fn routes() -> Vec<rocket::Route> {
                routes![
                    list,
                    create_form,
                    create,
                    edit_form,
                    edit,
                    delete,
                    action
                ]
            }
        }
    };
}
