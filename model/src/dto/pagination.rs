//! Generic pagination primitives used across all list endpoints.

use serde::{Deserialize, Serialize};

/// Query parameters for paginated, optionally filtered searches.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchParams {
    /// An optional title substring to filter results by.
    pub title: Option<String>,
    /// The one-based page number to retrieve.
    pub page: Option<u64>,
    /// The maximum number of items per page.
    pub per_page: Option<u64>,
}

/// A generic paginated response envelope.
///
/// Wraps a `Vec<T>` of items together with metadata describing the current
/// page, items per page, and total item count.
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    /// The items for the current page.
    pub data: Vec<T>,
    /// The total number of items matching the query across all pages.
    pub total: u64,
    /// The one-based page number that was returned.
    pub page: u64,
    /// The maximum number of items per page.
    pub per_page: u64,
}
