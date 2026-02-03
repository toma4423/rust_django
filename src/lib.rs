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
        .mount("/", routes![index])
        .mount("/auth", routes![controllers::auth::login, controllers::auth::logout])
        .mount("/admin", controllers::admin::routes())
        .mount("/admin", controllers::admin_groups::routes())
        .mount("/static", FileServer::from(relative!("static")))
}

#[get("/")]
fn index() -> Template {
    Template::render("index", rocket_dyn_templates::context! {
        title: "Home",
    })
}
