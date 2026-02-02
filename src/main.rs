#[macro_use]
extern crate rocket;

use migration::{Migrator, MigratorTrait};
use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

mod controllers;
mod db;
mod entities;
mod guards;
mod auth_utils;
mod errors;
mod services;
pub mod csrf;
pub mod validation;

/// アプリケーションのメインエントリーポイント。
/// Djangoの `manage.py runserver` 実行時の動きに相当します。
#[launch]
async fn rocket() -> _ {
    // 1. データベース接続（Djangoの初期化プロセスに相当）
    let db = db::set_up_db().await.expect("Failed to connect to DB");

    // 2. マイグレーションの実行（Djangoの `migrate` コマンドに相当）
    // アプリ起動時に自動でテーブルを作成するようにしています。
    Migrator::up(&db, None).await.expect("Failed to run migrations");

    // 3. Rocketインスタンスの構築
    rocket::build()
        // DB接続をRocketの管理下に置く（Djangoの `request.db` のようにどこからでも参照可能にする）
        .manage(db)
        // テンプレートエンジンの登録（Djangoの `TEMPLATES` 設定に相当）
        .attach(Template::fairing())
        // ルーティングの登録（Djangoの `urls.py` に相当）
        .mount("/", routes![index])
        .mount("/auth", routes![controllers::auth::login, controllers::auth::logout])
        .mount("/admin", controllers::admin::routes())
        .mount("/admin", controllers::admin_groups::routes())
        // 静的ファイルの配信（Djangoの `STATIC_URL` 設定に相当）
        .mount("/static", FileServer::from(relative!("static")))
}

#[get("/")]
fn index() -> Template {
    // Django風のトップページ
    Template::render("index", rocket_dyn_templates::context! {
        title: "Home",
    })
}
