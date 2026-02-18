//! Axum-compatible error wrapper.
//!
//! [`AppError`] wraps a [`ServiceError`] and implements [`IntoResponse`] so
//! that it can be returned directly from handler functions, producing a
//! JSON error body with the appropriate HTTP status code.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use service::error::ServiceError;

/// A new-type wrapper around [`ServiceError`] that implements
/// [`IntoResponse`].
#[derive(Debug)]
pub struct AppError(ServiceError);

impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        AppError(err)
    }
}

impl IntoResponse for AppError {
    /// Maps each [`ServiceError`] variant to an HTTP status code and a JSON
    /// body of the form `{ "error": "<message>" }`.
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            ServiceError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            ServiceError::NotFound { entity, id } => (StatusCode::NOT_FOUND, format!("{entity} with ID {id} not found")),
            ServiceError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}
