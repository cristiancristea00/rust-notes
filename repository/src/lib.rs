//! Data access layer for the notes application.
//!
//! This crate provides the [`DatabaseManager`](database::DatabaseManager) for managing
//! database connections, the [`NoteRepository`](note::NoteRepository) trait for abstracting
//! persistence operations, and its concrete implementation
//! [`NoteRepositoryImpl`](note::NoteRepositoryImpl).

pub mod database;
pub mod error;
pub mod note;
