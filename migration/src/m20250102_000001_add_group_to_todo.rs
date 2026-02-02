use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Todo::Table)
                    .add_column(ColumnDef::new(Todo::GroupId).integer().null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk-todo-group_id")
                            .from_tbl(Todo::Table)
                            .from_col(Todo::GroupId)
                            .to_tbl(Group::Table)
                            .to_col(Group::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop column is not universally supported in SeaORM migration for all DBs immediately
        // and often omitted in simple down migrations.
        Ok(())
    }
}

#[derive(Iden)]
enum Todo {
    Table,
    GroupId,
}

#[derive(Iden)]
enum Group {
    Table,
    Id,
}
