//! Note-specific request and response DTOs.

use crate::dto::datetime::FormattedDateTime;
use serde::{Deserialize, Serialize};

/// Request body for creating a new note.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateNoteRequest {
    /// The title of the note.
    pub title: String,
    /// The main body content of the note.
    pub content: String,
}

/// Request body for partially updating an existing note.
///
/// Only the fields that are [`Some`] will be applied; omitted fields remain
/// unchanged.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateNoteRequest {
    /// An optional new title for the note.
    pub title: Option<String>,
    /// An optional new body content for the note.
    pub content: Option<String>,
}

/// Serialisable representation of a note returned to the client.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteResponse {
    /// The unique identifier of the note.
    pub id: i64,
    /// The title of the note.
    pub title: String,
    /// The main body content of the note.
    pub content: String,
    /// The timestamp at which the note was originally created (UTC),
    /// formatted as e.g. `Friday, 3rd August 2034, 12:45:34 PM UTC`.
    pub created_at: FormattedDateTime,
    /// The timestamp at which the note was last updated (UTC),
    /// formatted as e.g. `Friday, 3rd August 2034, 12:45:34 PM UTC`.
    pub updated_at: FormattedDateTime,
}
