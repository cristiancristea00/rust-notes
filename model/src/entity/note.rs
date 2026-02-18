//! SeaORM entity for the `notes` table.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Derives the SeaORM model, relation, and active-model boilerplate.
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "notes")]
pub struct Model {
    /// Auto-incrementing primary key.
    #[sea_orm(primary_key)]
    pub id: i64,

    /// The title of the note (unique within the `item` key group).
    #[sea_orm(column_type = "Text", unique_key = "item")]
    pub title: String,

    /// The main body content of the note.
    #[sea_orm(column_type = "Text")]
    pub content: String,

    /// Timestamp set to the current UTC time when the row is first inserted.
    #[sea_orm(default_value = "Expr::current_timestamp()", unique_key = "item")]
    pub created_at: ChronoDateTimeUtc,

    /// Timestamp updated to the current UTC time whenever the row is modified.
    #[sea_orm(default_value = "Expr::current_timestamp()")]
    pub updated_at: ChronoDateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}
