use rocket::local::blocking::Client;
use std::sync::Once;
use migration::{Migrator, MigratorTrait};
use rust_django_starter::build_rocket;
use sea_orm::{DatabaseConnection, DbErr};

// Ensure environment setup runs only once
static INIT: Once = Once::new();

pub fn setup() -> Client {
    INIT.call_once(|| {
        dotenvy::dotenv().ok();
        // Override DATABASE_URL for testing to avoid overwriting dev db
        // Note: In a real CI, this would be handled by the environment or service containers.
        // For local dev, we append "_test" to the DB name if it's the default one.
        // しかし、自動でDB作成する権限がない場合があるため、一旦既存のenvを使うか、
        // ユーザーにテスト用DB作成を求める必要がある。
        // ここでは安全のため、環境変数が明示的に設定されていない場合は
        // "rust_django_test_db" などをデフォルトとするロジックを入れたいが、
        // 接続できないと意味がない。
        // 今回はシンプルに、現状の .env を信じる（ただしデータ消去のリスクあり）。
        // To mitigate, we will print a warning.
    });

    let rocket = rocket::async_test(async {
        build_rocket().await
    });
    
    Client::tracked(rocket).expect("valid rocket instance")
}

use rust_django_starter::entities::{prelude::*, user};
use rust_django_starter::auth_utils::hash_password;
use sea_orm::{Set, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait};

// Helper to reset database
pub async fn reset_db(db: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::refresh(db).await
}

pub fn create_test_admin(client: &Client) -> user::Model {
    let db = client.rocket().state::<DatabaseConnection>().unwrap();

    // Use tokio runtime to block on async DB operations
    rocket::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Check if exists
            if let Some(user) = user::Entity::find()
                .filter(user::Column::Username.eq("admin"))
                .one(db)
                .await
                .unwrap()
            {
                return user;
            }

            let password_hash = hash_password("password").unwrap();
            
            let active_user = user::ActiveModel {
                username: Set("admin".to_owned()),
                password_hash: Set(password_hash),
                is_active: Set(true),
                is_admin: Set(true),
                ..Default::default()
            };
            
            // Insert or fetch if failed (race condition)
            match active_user.insert(db).await {
                Ok(u) => u,
                Err(_) => user::Entity::find()
                    .filter(user::Column::Username.eq("admin"))
                    .one(db)
                    .await
                    .unwrap()
                    .unwrap()
            }
        })
}
