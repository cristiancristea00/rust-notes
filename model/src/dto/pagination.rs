//! Pagination primitives used across list endpoints.

use serde::{Deserialize, Serialize};

/// Query parameters for paginated, optionally filtered searches.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchParams {
    /// An optional title substring to filter results by.
    pub title: Option<String>,
    /// The one-based page number to retrieve.
    pub page: Option<u64>,
    /// The maximum number of items per page.
    pub size: Option<u64>,
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
