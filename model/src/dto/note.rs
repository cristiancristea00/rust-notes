//! Note-specific request and response DTOs.

use chrono::{DateTime, Utc};
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
pub struct NoteResponse {
    /// The unique identifier of the note.
    pub id: i64,
    /// The title of the note.
    pub title: String,
    /// The main body content of the note.
    pub content: String,
    /// The timestamp at which the note was originally created (UTC).
    pub created_at: DateTime<Utc>,
    /// The timestamp at which the note was last updated (UTC).
    pub updated_at: DateTime<Utc>,
}
