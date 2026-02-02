pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_user_table;
mod m20220102_000001_create_admin_user;
mod m20240101_000001_create_todo_table;
mod m20250101_000001_create_group_table;
mod m20250102_000001_add_group_to_todo;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_user_table::Migration),
            Box::new(m20220102_000001_create_admin_user::Migration),
            Box::new(m20240101_000001_create_todo_table::Migration),
            Box::new(m20250101_000001_create_group_table::Migration),
            Box::new(m20250102_000001_add_group_to_todo::Migration),
        ]
    }
}

