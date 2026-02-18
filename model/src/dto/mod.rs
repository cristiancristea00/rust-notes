//! Data Transfer Objects for request, response, and pagination payloads.
//!
//! * [`datetime`] – [`FormattedDateTime`](datetime::FormattedDateTime), a
//!   UTC timestamp newtype with human-readable serialisation.
//! * [`note`] – Request and response DTOs for note operations.
//! * [`pagination`] – Generic pagination request and response types.

pub mod datetime;
pub mod note;
pub mod pagination;
