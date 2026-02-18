//! HTTP controller layer for the notes application.
//!
//! This crate wires Axum route handlers to the [`NoteService`](service::note::NoteService)
//! trait, translating HTTP requests into service calls and service errors into
//! JSON error responses.

pub mod error;
pub mod note;
pub mod router;

pub use router::AppRouter;
