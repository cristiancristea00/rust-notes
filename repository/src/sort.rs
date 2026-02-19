//! Conversion traits bridging model sort types to SeaORM query primitives.
//!
//! [`IntoColumn`] converts a [`SortFieldName`] into the corresponding
//! [`note::Column`], and [`IntoOrder`] converts a [`SortDirection`] into a
//! SeaORM [`Order`]. Both traits are implemented here, keeping the mapping
//! logic in one place and out of the repository method bodies.

use model::{
    dto::pagination::{SortDirection, SortFieldName},
    entity::note,
};
use sea_orm::Order;

/// Converts a sort field name into the corresponding SeaORM column.
pub(crate) trait IntoColumn {
    /// Returns the SeaORM [`note::Column`] that corresponds to this field.
    fn into_column(self) -> note::Column;
}

/// Converts a sort direction into a SeaORM [`Order`].
pub(crate) trait IntoOrder {
    /// Returns the SeaORM [`Order`] that corresponds to this direction.
    fn into_order(self) -> Order;
}

impl IntoColumn for SortFieldName {
    fn into_column(self) -> note::Column {
        match self {
            SortFieldName::Id => note::Column::Id,
            SortFieldName::Title => note::Column::Title,
            SortFieldName::Content => note::Column::Content,
            SortFieldName::CreatedAt => note::Column::CreatedAt,
            SortFieldName::UpdatedAt => note::Column::UpdatedAt,
        }
    }
}

impl IntoOrder for SortDirection {
    fn into_order(self) -> Order {
        match self {
            SortDirection::Ascending => Order::Asc,
            SortDirection::Descending => Order::Desc,
        }
    }
}
