//! Axum handler functions for note CRUD endpoints.
//!
//! Each handler extracts the relevant parts of the HTTP request (path
//! parameters, query strings, and JSON bodies), delegates to the
//! [`NoteService`], and returns a typed Axum response.

use axum::{
    Json,
    extract::{Path, Query, State},
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
pub async fn create_note<Service: NoteService>(
    State(service): State<Service>,
    Json(req): Json<CreateNoteRequest>,
) -> Result<impl IntoResponse, AppError> {
    let note = service.create(req).await.map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(note)))
}

/// `GET /api/notes/{id}` – retrieves a single note by its primary key.
pub async fn get_note<Service: NoteService>(State(service): State<Service>, Path(id): Path<i64>) -> Result<impl IntoResponse, AppError> {
    let note = service.find_by_id(id).await.map_err(AppError::from)?;

    Ok(Json(note))
}

/// `GET /api/notes` – returns a paginated, optionally filtered list of notes.
pub async fn list_notes<Service: NoteService>(
    State(service): State<Service>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    let result = service.find_all(params).await.map_err(AppError::from)?;

    Ok(Json(result))
}

/// `PUT /api/notes/{id}` – partially updates an existing note.
pub async fn update_note<Service: NoteService>(
    State(service): State<Service>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateNoteRequest>,
) -> Result<impl IntoResponse, AppError> {
    let note = service.update(id, req).await.map_err(AppError::from)?;

    Ok(Json(note))
}

/// `DELETE /api/notes/{id}` – deletes a note and returns `204 No Content`.
pub async fn delete_note<Service: NoteService>(State(service): State<Service>, Path(id): Path<i64>) -> Result<StatusCode, AppError> {
    service.delete(id).await.map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
