use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;

/// データベース接続をセットアップします。
/// Djangoでは `settings.py` の `DATABASES` 設定に相当します。
pub async fn set_up_db() -> Result<DatabaseConnection, DbErr> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    // Database::connect は接続プールを自動的に作成します。
    // DjangoのDBエンジンと同様、内部でコネクション管理を行ってくれます。
    let db = Database::connect(db_url).await?;

    Ok(db)
}
