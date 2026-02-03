use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Rename 'permission' to 'permissions' if it exists
        // We use raw SQL because SeaORM's AlterTable rename is verbose for just renaming
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"DO $$
            BEGIN
              IF EXISTS(SELECT * FROM information_schema.tables WHERE table_name = 'permission' AND table_schema = 'public') THEN
                ALTER TABLE permission RENAME TO permissions;
              END IF;
            END
            $$;"#.to_owned()
        )).await?;
        
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"ALTER TABLE IF EXISTS permissions RENAME TO permission"#.to_owned()
        )).await?;
        Ok(())
    }
}
