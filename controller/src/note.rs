//! Axum handler functions for note CRUD endpoints.
//!
//! Each handler extracts the relevant parts of the HTTP request (path
//! parameters, query strings, and JSON bodies), delegates to the
//! [`NoteService`], and returns a typed Axum response. Extraction failures
//! are propagated as [`AppError::BadRequest`] via the `From` impls on
//! [`AppError`].

use axum::{
    Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
    response::IntoResponse,
};
use model::dto::{
    note::{CreateNoteRequest, UpdateNoteRequest},
    pagination::SearchParams,
};
use service::note::NoteService;

use crate::error::AppError;

/// `POST /api/notes` – creates a new note and returns it with `201 Created`.
#[tracing::instrument(skip_all)]
pub async fn create_note<Service: NoteService>(
    State(service): State<Service>,
    body: Result<Json<CreateNoteRequest>, JsonRejection>,
) -> Result<impl IntoResponse, AppError> {
    let Json(req) = body?;
    tracing::info!("Creating note");
    let note = service.create(req).await.map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(note)))
}

/// `GET /api/notes/{id}` – retrieves a single note by its primary key.
#[tracing::instrument(skip_all)]
pub async fn get_note<Service: NoteService>(
    State(service): State<Service>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<impl IntoResponse, AppError> {
    let Path(id) = path?;
    tracing::info!(id, "Fetching note");
    let note = service.find_by_id(id).await.map_err(AppError::from)?;

    Ok(Json(note))
}

/// `GET /api/notes` – returns a paginated, optionally filtered list of notes.
#[tracing::instrument(skip_all)]
pub async fn list_notes<Service: NoteService>(
    State(service): State<Service>,
    query: Result<Query<SearchParams>, QueryRejection>,
) -> Result<impl IntoResponse, AppError> {
    let Query(params) = query?;
    tracing::info!("Listing notes");
    let result = service.find_all(params).await.map_err(AppError::from)?;

    Ok(Json(result))
}

/// `PUT /api/notes/{id}` – partially updates an existing note.
#[tracing::instrument(skip_all)]
pub async fn update_note<Service: NoteService>(
    State(service): State<Service>,
    path: Result<Path<i64>, PathRejection>,
    body: Result<Json<UpdateNoteRequest>, JsonRejection>,
) -> Result<impl IntoResponse, AppError> {
    let Path(id) = path?;
    let Json(req) = body?;
    tracing::info!(id, "Updating note");
    let note = service.update(id, req).await.map_err(AppError::from)?;

    Ok(Json(note))
}

/// `DELETE /api/notes/{id}` – deletes a note and returns `204 No Content`.
#[tracing::instrument(skip_all)]
pub async fn delete_note<Service: NoteService>(
    State(service): State<Service>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, AppError> {
    let Path(id) = path?;
    tracing::info!(id, "Deleting note");
    service.delete(id).await.map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
