use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // TODOテーブルを作成
        // Djangoの models.py でクラスを定義するのに相当
        manager
            .create_table(
                Table::create()
                    .table(Todo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Todo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Todo::Title).string_len(200).not_null())
                    .col(ColumnDef::new(Todo::Description).text())
                    .col(ColumnDef::new(Todo::Completed).boolean().not_null().default(false))
                    .col(ColumnDef::new(Todo::Priority).integer().not_null().default(1))
                    .col(
                        ColumnDef::new(Todo::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Todo::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Todo::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_todo_user")
                            .from(Todo::Table, Todo::UserId)
                            .to(Alias::new("user"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Todo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Todo {
    Table,
    Id,
    Title,
    Description,
    Completed,
    Priority,
    UserId,
    CreatedAt,
    UpdatedAt,
}
