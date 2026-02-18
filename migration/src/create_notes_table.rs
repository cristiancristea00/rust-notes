use sea_orm_migration::prelude::*;

const TITLE_INDEX: &str = "notes_title_idx";

#[derive(DeriveIden)]
enum Notes {
    Table,
    Id,
    Title,
    Content,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut id = ColumnDef::new(Notes::Id);
        let mut title = ColumnDef::new(Notes::Title);
        let mut content = ColumnDef::new(Notes::Content);
        let mut created_at = ColumnDef::new(Notes::CreatedAt);
        let mut updated_at = ColumnDef::new(Notes::UpdatedAt);

        let table_create_statement: TableCreateStatement = Table::create()
            .table(Notes::Table)
            .if_not_exists()
            .col(id.unsigned().not_null().auto_increment().primary_key())
            .col(title.string().not_null())
            .col(content.text().not_null())
            .col(created_at.date_time().not_null().default(Expr::current_timestamp()))
            .col(updated_at.date_time().not_null().default(Expr::current_timestamp()))
            .to_owned();

        let title_index_create_statement: IndexCreateStatement = Index::create()
            .if_not_exists()
            .name(TITLE_INDEX)
            .table(Notes::Table)
            .col(Notes::Title)
            .to_owned();

        manager.create_table(table_create_statement).await?;
        manager.create_index(title_index_create_statement).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let index_drop_statement: IndexDropStatement = Index::drop().name(TITLE_INDEX).table(Notes::Table).to_owned();

        let table_drop_statement: TableDropStatement = Table::drop().table(Notes::Table).to_owned();

        manager.drop_index(index_drop_statement).await?;
        manager.drop_table(table_drop_statement).await?;

        Ok(())
    }
}
