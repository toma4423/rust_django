use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Djangoの `forward` マイグレーションに相当
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(User::PasswordHash).string().not_null())
                    .col(ColumnDef::new(User::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(User::IsAdmin).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await
    }

    // Djangoの `backward` マイグレーションに相当
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

/// Djangoのモデルフィールド名定義に相当する列挙型
#[derive(Iden)]
enum User {
    Table,
    Id,
    Username,
    PasswordHash,
    IsActive,
    IsAdmin,
}
