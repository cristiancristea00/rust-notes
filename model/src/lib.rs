//! Shared domain models for the notes application.
//!
//! This crate houses two sub-modules:
//!
//! * [`dto`] – Data Transfer Objects used at the API boundary (requests,
//!   responses, and pagination helpers).
//! * [`entity`] – SeaORM entity definitions that map directly to database
//!   tables.

pub mod dto;
pub mod entity;
