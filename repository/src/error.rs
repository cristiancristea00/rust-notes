//! Error types for the repository layer.
//!
//! [`RepositoryError`] is the generic, entity-agnostic error, whilst
//! [`NoteRepositoryError`] is specific to note persistence operations.
//! A [`From`] conversion between the two is provided for convenience.

use thiserror::Error;

/// A generic repository error that is not tied to any specific entity.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// An error originating from the underlying database driver.
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    /// The requested entity could not be found.
    #[error("{entity} with ID {id} not found")]
    NotFound {
        /// The human-readable name of the entity (e.g. `"Note"`).
        entity: String,
        /// The primary-key identifier that was looked up.
        id: i64,
    },
}

/// An error specific to note repository operations.
#[derive(Debug, Error)]
pub enum NoteRepositoryError {
    /// An error originating from the underlying database driver.
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    /// The note with the given ID could not be found.
    #[error("Note with ID {0} not found")]
    NotFound(i64),
}

impl From<NoteRepositoryError> for RepositoryError {
    fn from(error: NoteRepositoryError) -> Self {
        match error {
            NoteRepositoryError::DatabaseError(err) => RepositoryError::DatabaseError(err),
            NoteRepositoryError::NotFound(id) => RepositoryError::NotFound { entity: "Note".into(), id },
        }
    }
}
