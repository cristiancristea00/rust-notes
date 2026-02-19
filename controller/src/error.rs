//! Axum-compatible error types.
//!
//! [`AppError`] unifies service-layer errors and Axum extraction rejections
//! into a single type that implements [`IntoResponse`], producing a JSON
//! error body with the appropriate HTTP status code.

use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use model::dto::pagination::SearchParams;
use service::error::ServiceError;

/// Unified application error that can originate from either the service
/// layer or from Axum extraction failures.
#[derive(Debug)]
pub enum AppError {
    /// An error propagated from the service layer.
    Service(ServiceError),
    /// A bad-request error caused by an invalid extractor input
    /// (query string, path parameter, or JSON body).
    BadRequest(String),
}

impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        AppError::Service(err)
    }
}

impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> Self {
        let body = rejection.body_text();
        let prefix = if let Some(start) = body.find("unknown field `") {
            let rest = &body[start + "unknown field `".len()..];
            let field = rest.split('`').next().unwrap_or("unknown");
            format!("Invalid query parameter '{field}'.")
        } else {
            "Invalid query parameters.".to_owned()
        };

        AppError::BadRequest(format!("{prefix} {}", SearchParams::params_hint()))
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::BadRequest(rejection.body_text())
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        AppError::BadRequest(rejection.body_text())
    }
}

impl IntoResponse for AppError {
    /// Maps each [`AppError`] variant to an HTTP status code and a JSON
    /// body of the form `{ "error": "<message>" }`.
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Service(service_error) => match service_error {
                ServiceError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
                ServiceError::NotFound { entity, id } => (StatusCode::NOT_FOUND, format!("{entity} with ID {id} not found")),
                ServiceError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            },
        };

        if status.is_client_error() {
            tracing::warn!(status = %status, error = %message, "Client error");
        } else {
            tracing::error!(status = %status, error = %message, "Server error");
        }

        let body = Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}
