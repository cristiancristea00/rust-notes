//! Service-layer error types.
//!
//! [`ServiceError`] unifies validation failures, not-found conditions, and
//! internal errors into a single enum that the controller layer can map to
//! appropriate HTTP status codes.

use repository::error::{NoteRepositoryError, RepositoryError};
use thiserror::Error;

/// Enumerates all errors that can originate from the service layer.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// A request failed input validation.
    #[error("Validation error: {0}")]
    Validation(String),

    /// The requested entity could not be found.
    #[error("{entity} with ID {id} not found")]
    NotFound {
        /// The human-readable name of the entity (e.g. `"Note"`).
        entity: String,
        /// The primary-key identifier that was looked up.
        id: i64,
    },

    /// An unexpected internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<RepositoryError> for ServiceError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::NotFound { entity, id } => ServiceError::NotFound { entity, id },
            RepositoryError::DatabaseError(e) => ServiceError::Internal(e.to_string()),
        }
    }
}

impl From<NoteRepositoryError> for ServiceError {
    fn from(error: NoteRepositoryError) -> Self {
        match error {
            NoteRepositoryError::NotFound(id) => ServiceError::NotFound { entity: "Note".into(), id },
            NoteRepositoryError::DatabaseError(err) => ServiceError::Internal(err.to_string()),
        }
    }
}
