//! Business-logic layer for the notes application.
//!
//! This crate sits between the controller (HTTP) and the repository (database)
//! layers, providing validation, default pagination, and error translation.

pub mod error;
pub mod note;
