//! Database migration definitions for the notes application.
//!
//! [`Migrator`] collects all migration steps and implements
//! [`MigratorTrait`] so that they can be applied (or rolled back) via
//! `Migrator::up` and `Migrator::down`.

pub use sea_orm_migration::prelude::*;

mod create_notes_table;

/// Top-level migrator that registers every migration in the correct order.
pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(create_notes_table::Migration)]
    }
}
