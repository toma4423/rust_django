#[macro_export]
macro_rules! impl_admin_resource {
    // Mode 1: Full Implementation (Default)
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
        $crate::impl_admin_resource!(
            @inner
            entity: $entity,
            active_model: $active_model,
            form: $form,
            view_prefix: $view_prefix,
            base_url: $base_url,
            template_dir: $template_dir,
            search_field: $search_field,
            order_field: $order_field,
            skip_list: false,
            skip_create: false,
            skip_update: false
        );
    };

    // Mode 2: Custom Implementation
    (
        entity: $entity:ty,
        active_model: $active_model:ty,
        form: $form:ty,
        view_prefix: $view_prefix:ident,
        base_url: $base_url:expr,
        template_dir: $template_dir:expr,
        search_field: $search_field:expr,
        order_field: $order_field:expr,
        // Flags
        skip_list: $skip_list:tt,
        skip_create: $skip_create:tt,
        skip_update: $skip_update:tt
    ) => {
        $crate::impl_admin_resource!(
            @inner
            entity: $entity,
            active_model: $active_model,
            form: $form,
            view_prefix: $view_prefix,
            base_url: $base_url,
            template_dir: $template_dir,
            search_field: $search_field,
            order_field: $order_field,
            skip_list: $skip_list,
            skip_create: $skip_create,
            skip_update: $skip_update
        );
    };

    // Internal implementation
    (
        @inner
        entity: $entity:ty,
        active_model: $active_model:ty,
        form: $form:ty,
        view_prefix: $view_prefix:ident,
        base_url: $base_url:expr,
        template_dir: $template_dir:expr,
        search_field: $search_field:expr,
        order_field: $order_field:expr,
        skip_list: $skip_list:tt,
        skip_create: $skip_create:tt,
        skip_update: $skip_update:tt
    ) => {


        paste::paste! {
            // View Structs
            pub struct [<$view_prefix ListView>];
            pub struct [<$view_prefix CreateView>];
            pub struct [<$view_prefix UpdateView>];
            pub struct [<$view_prefix DeleteView>];
        }
        
        // Conditional Macros
        $crate::impl_admin_resource_list!($skip_list, $entity, $view_prefix, $template_dir, $search_field, $base_url);
        $crate::impl_admin_resource_create!($skip_create, $entity, $active_model, $form, $view_prefix, $base_url, $template_dir);
        $crate::impl_admin_resource_update!($skip_update, $entity, $active_model, $form, $view_prefix, $base_url);
        
        // Delete View (Always generated for now)
        paste::paste! {
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

// Sub-macros for conditional generation
#[macro_export]
macro_rules! impl_admin_resource_list {
    (false, $entity:ty, $view_prefix:ident, $template_dir:expr, $search_field:expr, $base_url:expr) => {
        paste::paste! {
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
            #[get("/?<page>&<q>&<sort>&<dir>")]
            pub async fn list(db: &State<DatabaseConnection>, _admin: AdminUser, page: Option<usize>, q: Option<String>, sort: Option<String>, dir: Option<String>) -> AppTemplate {
                let view = [<$view_prefix ListView>];
                let context = serde_json::json!({ "base_url": $base_url });
                view.list(db, page.unwrap_or(1), q, sort, dir, &std::collections::HashMap::new(), context).await
            }
        }
    };
    (true, $($rest:tt)*) => {}; 
}

#[macro_export]
macro_rules! impl_admin_resource_create {
    (false, $entity:ty, $active_model:ty, $form:ty, $view_prefix:ident, $base_url:expr, $template_dir:expr) => {
        paste::paste! {
            #[rocket::async_trait]
            impl CreateView<$active_model> for [<$view_prefix CreateView>] {
                fn success_url(&self) -> String { $base_url.to_string() }
                async fn save(&self, db: &DatabaseConnection, data: &serde_json::Value) -> Result<<$entity as EntityTrait>::Model, DbErr> {
                     let form_struct: $form = serde_json::from_value(data.clone()).map_err(|e| DbErr::Custom(e.to_string()))?;
                     let active_model: $active_model = form_struct.into();
                     active_model.insert(db).await
                }
            }
            #[get("/create")]
            pub fn create_form(_admin: AdminUser) -> AppTemplate {
                AppTemplate::new(concat!($template_dir, "_form"), context! { active_nav: $base_url.trim_start_matches("/admin/"), base_url: $base_url })
            }
            #[post("/create", data = "<form>")]
            pub async fn create(db: &State<DatabaseConnection>, _admin: AdminUser, csrf: CsrfToken, form: Form<$form>) -> Result<Flash<Redirect>, AppTemplate> {
                 let view = [<$view_prefix CreateView>];
                 let form_inner = form.into_inner();
                 if !csrf.verify(&form_inner.csrf_token) {
                     // CSRF failure still redirects for now, or render? 
                     // Render is better but let's stick to simple change first.
                     // But if signature changes to AppTemplate, I can't return Default Redirect easily unless strict matching.
                     // Actually Result<Flash<Redirect>, AppTemplate> allows Redirect in Ok.
                     // For CSRF error, I should render too? or Redirect.
                     // Let's use Flash error redirect for CSRF for simplicity (or changing signature requires it).
                     // Wait, Err(AppTemplate) is for Error. Ok(Flash) is for Success.
                     // If I want to Redirect on Error (CSRF), I need to return Ok(Flash::error)? 
                     // But Flash::error is just Flash details.
                     // Result<Flash<Redirect>, AppTemplate> means:
                     // Ok -> Flash<Redirect> (Success or Error Redirect)
                     // Err -> AppTemplate (Rendered View)
                     return Ok(Flash::error(Redirect::to(format!("{}/create", $base_url)), "CSRF検証に失敗しました")); 
                 }
                 
                 let json_data = serde_json::to_value(&form_inner).unwrap();
                 match view.save(db, &json_data).await {
                     Ok(_) => Ok(Flash::success(Redirect::to($base_url), "作成しました")),
                     Err(e) => {
                         let error_msg = if e.to_string().contains("duplicate") { "既に使用されています".to_string() }
                                         else { format!("作成に失敗しました: {}", e) };
                         
                         let mut context_value = serde_json::json!({
                             "error": error_msg,
                             "form": form_inner,
                             "active_nav": $base_url.trim_start_matches("/admin/"),
                             "base_url": $base_url
                         });
                         
                         if let serde_json::Value::Object(ref mut map) = context_value {
                             if let serde_json::Value::Object(extra) = view.get_context_data(db).await {
                                 for (k, v) in extra { map.insert(k, v); }
                             }
                         }
                         
                         Err(AppTemplate::new(view.template_name(), context_value))
                 }}
            }
        }
    };
    (true, $($rest:tt)*) => {};
}

#[macro_export]
macro_rules! impl_admin_resource_update {
    (false, $entity:ty, $active_model:ty, $form:ty, $view_prefix:ident, $base_url:expr) => {
        paste::paste! {
            #[rocket::async_trait]
            impl UpdateView<$active_model> for [<$view_prefix UpdateView>] {
                fn success_url(&self) -> String { $base_url.to_string() }
                async fn get_object(&self, db: &DatabaseConnection, id: i32) -> Result<Option<<$entity as EntityTrait>::Model>, DbErr> { <$entity>::find_by_id(id).one(db).await }
                async fn save(&self, db: &DatabaseConnection, id: i32, data: &serde_json::Value) -> Result<<$entity as EntityTrait>::Model, DbErr> {
                     let _existing = <$entity>::find_by_id(id).one(db).await?.ok_or(DbErr::Custom("NotFound".into()))?;
                     let form_struct: $form = serde_json::from_value(data.clone()).map_err(|e| DbErr::Custom(e.to_string()))?;
                     let mut active_model: $active_model = form_struct.into();
                     active_model.id = Set(id);
                     Ok(active_model.update(db).await?)
                }
            }
            #[get("/edit/<id>")]
            pub async fn edit_form(db: &State<DatabaseConnection>, _admin: AdminUser, id: i32) -> Result<AppTemplate, Flash<Redirect>> {
                let view = [<$view_prefix UpdateView>];
                let context = serde_json::json!({ "base_url": $base_url, "active_nav": $base_url.trim_start_matches("/admin/") });
                view.get(db, id, None, context).await
            }
            #[post("/edit/<id>", data = "<form>")]
            pub async fn edit(db: &State<DatabaseConnection>, _admin: AdminUser, csrf: CsrfToken, id: i32, form: Form<$form>) -> Result<Flash<Redirect>, AppTemplate> {
                 let view = [<$view_prefix UpdateView>];
                 let form_inner = form.into_inner();
                 if !csrf.verify(&form_inner.csrf_token) { 
                     return Ok(Flash::error(Redirect::to(format!("{}/edit/{}", $base_url, id)), "CSRF検証に失敗しました")); 
                 }
                 
                 let json_data = serde_json::to_value(&form_inner).unwrap();
                 match view.save(db, id, &json_data).await {
                     Ok(_) => Ok(Flash::success(Redirect::to($base_url), "更新しました")),
                     Err(e) => {
                         let mut context_value = serde_json::json!({
                            "error": format!("更新に失敗しました: {}", e),
                            "form": form_inner,
                            "is_edit": true,
                            "id": id,
                            "base_url": $base_url,
                            "active_nav": $base_url.trim_start_matches("/admin/")
                        });
                        
                        if let serde_json::Value::Object(ref mut map) = context_value {
                             if let serde_json::Value::Object(extra) = view.get_context_data(db).await {
                                 for (k, v) in extra { map.insert(k, v); }
                             }
                         }

                        Err(AppTemplate::new(view.template_name(), context_value))
                     }
                 }
            }
        }
    };
    (true, $($rest:tt)*) => {};
}
