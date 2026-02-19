//! Pagination and sorting primitives used across list endpoints.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The direction to sort results in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Sort results in ascending order.
    Ascending,
    /// Sort results in descending order.
    Descending,
}

/// The field to sort notes by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortFieldName {
    /// Sort by note ID.
    Id,
    /// Sort by note title.
    Title,
    /// Sort by note content.
    Content,
    /// Sort by creation timestamp.
    CreatedAt,
    /// Sort by last-updated timestamp.
    UpdatedAt,
}

impl SortFieldName {
    /// All variants of the enum, in declaration order.
    pub const ALL: &[SortFieldName] = &[Self::Id, Self::Title, Self::Content, Self::CreatedAt, Self::UpdatedAt];

    /// Returns a comma-separated list of all valid field names (e.g.
    /// `"id, title, content, createdAt, updatedAt"`).
    pub fn all_names() -> String {
        Self::ALL.iter().map(|field| field.to_string()).collect::<Vec<_>>().join(", ")
    }
}

impl fmt::Display for SortFieldName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Id => "id",
            Self::Title => "title",
            Self::Content => "content",
            Self::CreatedAt => "createdAt",
            Self::UpdatedAt => "updatedAt",
        };
        formatter.write_str(name)
    }
}

impl FromStr for SortFieldName {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "id" => Ok(Self::Id),
            "title" => Ok(Self::Title),
            "content" => Ok(Self::Content),
            "createdAt" => Ok(Self::CreatedAt),
            "updatedAt" => Ok(Self::UpdatedAt),
            other => Err(format!("Unknown sort field: '{other}'. Valid fields: {}", Self::all_names())),
        }
    }
}

/// A single parsed sort criterion, combining a field name with a direction.
#[derive(Debug, Clone, Copy)]
pub struct SortField {
    /// The field to sort by.
    pub name: SortFieldName,
    /// The direction in which to sort.
    pub direction: SortDirection,
}

/// Query parameters for paginated, optionally filtered, and sorted searches.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchParams {
    /// An optional title substring to filter results by.
    pub title: Option<String>,
    /// The one-based page number to retrieve.
    pub page: Option<u64>,
    /// The maximum number of items per page.
    pub size: Option<u64>,
    /// Comma-separated sort fields with an optional `+` (ascending) or `-`
    /// (descending) prefix, e.g. `title,-createdAt`.
    ///
    /// Defaults to ascending when no prefix is supplied. Unknown field names
    /// are rejected by the service layer with a validation error.
    #[serde(rename = "orderBy")]
    pub order_by: Option<String>,
    /// Parsed sort fields, populated by the service layer after validating
    /// [`order_by`](Self::order_by). Not deserialised from the query string.
    #[serde(skip)]
    pub sort_fields: Vec<SortField>,
}

/// Metadata describing the pagination state of a response.
///
/// Serialised with camelCase field names to match the API contract
/// (e.g. `totalElements`, `totalPages`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    /// The maximum number of items per page.
    pub size: u64,
    /// The one-based page number that was returned.
    pub number: u64,
    /// The total number of items matching the query across all pages.
    pub total_elements: u64,
    /// The total number of pages available.
    pub total_pages: u64,
}

/// A paginated response envelope containing notes and page metadata.
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    /// The note items for the current page.
    pub notes: Vec<T>,
    /// Pagination metadata.
    pub page: PageInfo,
}
