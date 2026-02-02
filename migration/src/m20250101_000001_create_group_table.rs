use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Group Table
        manager
            .create_table(
                Table::create()
                    .table(Group::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Group::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Group::Name).string().not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        // GroupUser Table (Many-to-Many)
        manager
            .create_table(
                Table::create()
                    .table(GroupUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupUser::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GroupUser::GroupId).integer().not_null())
                    .col(ColumnDef::new(GroupUser::UserId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group_user-group_id")
                            .from(GroupUser::Table, GroupUser::GroupId)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group_user-user_id")
                            .from(GroupUser::Table, GroupUser::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupUser::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Group::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Group {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum GroupUser {
    Table,
    Id,
    GroupId,
    UserId,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}
