pub use sea_orm_migration::prelude::*;

mod m20240523_173708_create_conversation_table;
mod m20240523_214940_add_dates_to_conversation_table;
mod m20240523_225333_add_updated_at_to_conversation_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240523_173708_create_conversation_table::Migration),
            Box::new(m20240523_214940_add_dates_to_conversation_table::Migration),
            Box::new(m20240523_225333_add_updated_at_to_conversation_table::Migration),
        ]
    }
}
