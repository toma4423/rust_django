use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        // Define standard permissions
        let permissions = vec![
            ("ユーザーを閲覧", "auth.view_user"),
            ("ユーザーを追加", "auth.add_user"),
            ("ユーザーを変更", "auth.change_user"),
            ("ユーザーを削除", "auth.delete_user"),
            ("グループを閲覧", "auth.view_group"),
            ("グループを追加", "auth.add_group"),
            ("グループを変更", "auth.change_group"),
            ("グループを削除", "auth.delete_group"),
        ];

        // Insert using raw SQL for simplicity in migration without entity access
        for (name, codename) in permissions {
            // Check if exists to be idempotent
            let exists = db.query_one(Statement::from_sql_and_values(
                manager.get_database_backend(),
                r#"SELECT id FROM permissions WHERE codename = $1"#,
                vec![codename.into()]
            )).await?;
            
            if exists.is_none() {
                 db.execute(Statement::from_sql_and_values(
                    manager.get_database_backend(),
                    r#"INSERT INTO permissions (name, codename) VALUES ($1, $2)"#,
                    vec![name.into(), codename.into()]
                )).await?;
            }
        }
        
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DELETE FROM permissions WHERE codename LIKE 'auth.%'"#.to_owned()
        )).await?;
        Ok(())
    }
}
