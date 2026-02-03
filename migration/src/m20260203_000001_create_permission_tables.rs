use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Permission Table
        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Permission::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Permission::Name).string().not_null())
                    .col(ColumnDef::new(Permission::Codename).string().not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        // 2. GroupPermission Table (Many-to-Many)
        manager
            .create_table(
                Table::create()
                    .table(GroupPermission::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GroupPermission::GroupId).integer().not_null())
                    .col(ColumnDef::new(GroupPermission::PermissionId).integer().not_null())
                    // Composite Primary Key
                    .primary_key(
                        Index::create()
                            .name("pk-group_permission")
                            .col(GroupPermission::GroupId)
                            .col(GroupPermission::PermissionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group_permission-group_id")
                            .from(GroupPermission::Table, GroupPermission::GroupId)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-group_permission-permission_id")
                            .from(GroupPermission::Table, GroupPermission::PermissionId)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. UserPermission Table (Many-to-Many)
        manager
            .create_table(
                Table::create()
                    .table(UserPermission::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserPermission::UserId).integer().not_null())
                    .col(ColumnDef::new(UserPermission::PermissionId).integer().not_null())
                    // Composite Primary Key
                    .primary_key(
                        Index::create()
                            .name("pk-user_permission")
                            .col(UserPermission::UserId)
                            .col(UserPermission::PermissionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_permission-user_id")
                            .from(UserPermission::Table, UserPermission::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_permission-permission_id")
                            .from(UserPermission::Table, UserPermission::PermissionId)
                            .to(Permission::Table, Permission::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserPermission::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GroupPermission::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Permission {
    Table,
    Id,
    Name,
    Codename,
}

#[derive(Iden)]
enum GroupPermission {
    Table,
    GroupId,
    PermissionId,
}

#[derive(Iden)]
enum UserPermission {
    Table,
    UserId,
    PermissionId,
}

#[derive(Iden)]
enum Group {
    Table,
    Id,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}
