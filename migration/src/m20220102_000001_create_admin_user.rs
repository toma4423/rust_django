use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // 初期管理者ユーザーを作成
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Argon2でハッシュ化された "admin" のパスワード
        // 本番環境では環境変数などから取得することを推奨
        let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$gbLN0HdzIAg3N/2UmMqJYQ$5xWILs4rN6xIJJE9uPQSAggsNMlPCFlRnS3iqv63Juk";
        
        let insert = Query::insert()
            .into_table(Alias::new("user"))
            .columns([
                Alias::new("username"),
                Alias::new("password_hash"),
                Alias::new("is_active"),
                Alias::new("is_admin"),
            ])
            .values_panic([
                "admin".into(),
                password_hash.into(),
                true.into(),
                true.into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let delete = Query::delete()
            .from_table(Alias::new("user"))
            .and_where(Expr::col(Alias::new("username")).eq("admin"))
            .to_owned();

        manager.exec_stmt(delete).await
    }
}
