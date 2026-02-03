#[macro_use]
extern crate rocket;

use migration::{Migrator, MigratorTrait};
use rocket::fs::{FileServer, relative};
use rocket::Build;
use rocket_dyn_templates::Template;

pub mod controllers;
pub mod db;
pub mod entities;
pub mod guards;
pub mod auth_utils;
pub mod errors;
pub mod services;
pub mod views;
use crate::views::app_template::AppTemplate;
pub mod fairings;
pub mod macros;
pub mod csrf;
pub mod validation;

/// Rocketインスタンスを構築する関数。
/// テスト時にも利用できるように分離しています。
pub async fn build_rocket() -> rocket::Rocket<Build> {
    // .envファイルを読み込む (環境変数の読み込み)
    dotenvy::dotenv().ok();

    // 1. データベース接続
    let db = db::set_up_db().await.expect("Failed to connect to DB");

    // 2. マイグレーションの実行
    Migrator::up(&db, None).await.expect("Failed to run migrations");

    // 3. Rocketインスタンスの構築
    rocket::build()
        .manage(db)
        .attach(Template::fairing())
        .attach(fairings::context::ContextFairing)
        .mount("/", routes![index, setup_admin])
        .mount("/auth", routes![controllers::auth::login, controllers::auth::logout, controllers::auth::login_form])
        .mount("/admin", controllers::admin::routes())
        .mount("/admin", controllers::admin_groups::routes())
        .mount("/static", FileServer::from(relative!("static")))
}

#[get("/")]
fn index() -> AppTemplate {
    AppTemplate::new("welcome", rocket_dyn_templates::context! {
        title: "Welcome",
    })
}

#[get("/setup_admin")]
async fn setup_admin(db: &rocket::State<sea_orm::DatabaseConnection>) -> String {
    use sea_orm::*;
    use crate::entities::user;
    use crate::auth_utils::hash_password;
    
    let username = "admin";
    let password = "password";
    
    // Check if exists
    let existing = user::Entity::find().filter(user::Column::Username.eq(username)).one(db.inner()).await;
    
    let hash = hash_password(password).unwrap();
    
    if let Ok(Some(u)) = existing {
        let mut active: user::ActiveModel = u.into();
        active.password_hash = Set(hash);
        match active.update(db.inner()).await {
             Ok(_) => "Admin password updated".to_string(),
             Err(e) => format!("Error updating: {}", e),
        }
    } else {
        let user = user::ActiveModel {
            username: Set(username.to_owned()),
            password_hash: Set(hash),
            is_admin: Set(true),
            is_active: Set(true),
            ..Default::default()
        };
        
        match user.insert(db.inner()).await {
            Ok(_) => "Admin created".to_string(),
            Err(e) => format!("Error creating: {}", e),
        }
    }
}


