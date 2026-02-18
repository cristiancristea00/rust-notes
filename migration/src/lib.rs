pub use sea_orm_migration::prelude::*;

mod create_notes_table;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(create_notes_table::Migration)]
    }
}
