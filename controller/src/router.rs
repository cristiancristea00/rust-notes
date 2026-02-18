//! Application router construction.
//!
//! [`AppRouter`] provides a typed builder that converts a [`NoteService`] into
//! a fully configured Axum [`Router`] via the [`From`] trait.

use axum::{routing::get, Router};
use service::note::NoteService;

use crate::note::{create_note, delete_note, get_note, list_notes, update_note};

/// A typed router builder that converts a [`NoteService`] into an Axum
/// [`Router`].
pub struct AppRouter<Service: NoteService> {
    /// The service instance that will be installed as Axum shared state.
    service: Service,
}

impl<Service: NoteService> AppRouter<Service> {
    /// Creates a new [`AppRouter`] wrapping the given service.
    pub fn new(service: Service) -> Self {
        Self { service }
    }
}

impl<Service: NoteService> From<AppRouter<Service>> for Router {
    /// Builds the full Axum [`Router`] with all note endpoints registered
    /// and the service installed as shared state.
    fn from(app: AppRouter<Service>) -> Self {
        Router::new()
            .route("/api/notes", get(list_notes::<Service>).post(create_note::<Service>))
            .route(
                "/api/notes/{id}",
                get(get_note::<Service>).put(update_note::<Service>).delete(delete_note::<Service>),
            )
            .with_state(app.service)
    }
}
