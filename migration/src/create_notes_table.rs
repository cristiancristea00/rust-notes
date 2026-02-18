//! Migration that creates the `notes` table and its title/created-at index.

use sea_orm_migration::prelude::*;

/// The name of the table managed by this migration.
pub const TABLE_NAME: &str = "notes";

/// The name of the composite index on `title` and `created_at`.
const TITLE_INDEX: &str = "notes_title_idx";

/// Column identifiers used by the migration DSL.
#[derive(DeriveIden)]
enum Notes {
    /// Primary-key column.
    Id,
    /// Note title column.
    Title,
    /// Note content column.
    Content,
    /// Row creation timestamp column.
    CreatedAt,
    /// Row last-updated timestamp column.
    UpdatedAt,
}

/// Creates (and drops) the `notes` table together with a composite index
/// on `(title, created_at)`.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies the migration: creates the `notes` table and the
    /// `notes_title_idx` index if they do not already exist.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut id = ColumnDef::new(Notes::Id);
        let mut title = ColumnDef::new(Notes::Title);
        let mut content = ColumnDef::new(Notes::Content);
        let mut created_at = ColumnDef::new(Notes::CreatedAt);
        let mut updated_at = ColumnDef::new(Notes::UpdatedAt);

        let table_create_statement: TableCreateStatement = Table::create()
            .table(TABLE_NAME)
            .if_not_exists()
            .col(id.integer().not_null().auto_increment().primary_key())
            .col(title.string().not_null())
            .col(content.text().not_null())
            .col(created_at.date_time().not_null().default(Expr::current_timestamp()))
            .col(updated_at.date_time().not_null().default(Expr::current_timestamp()))
            .to_owned();

        let item_index_create_statement: IndexCreateStatement = Index::create()
            .if_not_exists()
            .name(TITLE_INDEX)
            .table(TABLE_NAME)
            .col(Notes::Title)
            .col(Notes::CreatedAt)
            .to_owned();

        manager.create_table(table_create_statement).await?;
        manager.create_index(item_index_create_statement).await?;

        Ok(())
    }

    /// Rolls back the migration: drops the `notes_title_idx` index and then
    /// the `notes` table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let index_drop_statement: IndexDropStatement = Index::drop().name(TITLE_INDEX).table(TABLE_NAME).to_owned();

        let table_drop_statement: TableDropStatement = Table::drop().table(TABLE_NAME).to_owned();

        manager.drop_index(index_drop_statement).await?;
        manager.drop_table(table_drop_statement).await?;

        Ok(())
    }
}
