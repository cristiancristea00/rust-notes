//! Note service trait, its implementation, and request validation.
//!
//! The [`NoteService`] trait defines the business operations exposed to the
//! controller layer, whilst [`NoteServiceImpl`] provides the concrete
//! implementation backed by a [`NoteRepository`].

use model::dto::{
    note::{CreateNoteRequest, NoteResponse, UpdateNoteRequest},
    pagination::{PaginatedResponse, SearchParams},
};
use repository::note::NoteRepository;
use std::future::Future;

use crate::error::ServiceError;

/// Maximum allowed length for a note title, in characters.
const MAX_TITLE_LEN: usize = 255;

/// Default page number when none is provided by the client.
const DEFAULT_PAGE: u64 = 1;

/// Default number of items per page when none is provided by the client.
const DEFAULT_PER_PAGE: u64 = 20;

/// Hard upper limit on items per page to prevent excessively large responses.
const MAX_PER_PAGE: u64 = 100;

/// Trait abstracting CRUD business operations for notes.
///
/// Implementations must be [`Send`], [`Sync`], [`Clone`], and `'static` so
/// that they can be used as Axum shared state.
pub trait NoteService: Send + Sync + Clone + 'static {
    /// Validates and creates a new note.
    fn create(&self, request: CreateNoteRequest) -> impl Future<Output = Result<NoteResponse, ServiceError>> + Send;

    /// Retrieves a single note by its primary key.
    fn find_by_id(&self, id: i64) -> impl Future<Output = Result<NoteResponse, ServiceError>> + Send;

    /// Returns a paginated, optionally filtered list of notes.
    fn find_all(&self, params: SearchParams) -> impl Future<Output = Result<PaginatedResponse<NoteResponse>, ServiceError>> + Send;

    /// Validates and partially updates an existing note.
    fn update(&self, id: i64, request: UpdateNoteRequest) -> impl Future<Output = Result<NoteResponse, ServiceError>> + Send;

    /// Deletes a note by its primary key.
    fn delete(&self, id: i64) -> impl Future<Output = Result<(), ServiceError>> + Send;
}

/// Concrete [`NoteService`] backed by a generic [`NoteRepository`].
#[derive(Clone)]
pub struct NoteServiceImpl<Repo: NoteRepository> {
    /// The repository used for data access.
    repository: Repo,
}

impl<Repo: NoteRepository> NoteServiceImpl<Repo> {
    /// Creates a new [`NoteServiceImpl`] wrapping the given repository.
    pub fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

/// Internal validation trait implemented by request DTOs.
trait Validate {
    /// Returns `Ok(())` when the payload is valid, or a
    /// [`ServiceError::Validation`] describing the first violation found.
    fn validate(&self) -> Result<(), ServiceError>;
}

impl Validate for CreateNoteRequest {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.title.trim().is_empty() {
            return Err(ServiceError::Validation("Title must not be empty".into()));
        }

        if self.title.len() > MAX_TITLE_LEN {
            return Err(ServiceError::Validation(format!("Title must be at most {MAX_TITLE_LEN} characters")));
        }

        if self.content.trim().is_empty() {
            return Err(ServiceError::Validation("Content must not be empty".into()));
        }

        Ok(())
    }
}

impl Validate for UpdateNoteRequest {
    fn validate(&self) -> Result<(), ServiceError> {
        if let Some(ref title) = self.title {
            if title.trim().is_empty() {
                return Err(ServiceError::Validation("Title must not be empty".into()));
            }

            if title.len() > MAX_TITLE_LEN {
                return Err(ServiceError::Validation(format!("Title must be at most {MAX_TITLE_LEN} characters")));
            }
        }

        if let Some(ref content) = self.content {
            if content.trim().is_empty() {
                return Err(ServiceError::Validation("Content must not be empty".into()));
            }
        }

        Ok(())
    }
}

impl<Repo: NoteRepository> NoteService for NoteServiceImpl<Repo> {
    /// Validates the incoming request and delegates to the repository to
    /// persist the new note.
    async fn create(&self, request: CreateNoteRequest) -> Result<NoteResponse, ServiceError> {
        request.validate()?;

        self.repository.create(request).await.map_err(ServiceError::from)
    }

    /// Fetches a single note by ID, translating repository errors into
    /// service-layer errors.
    async fn find_by_id(&self, id: i64) -> Result<NoteResponse, ServiceError> {
        self.repository.find_by_id(id).await.map_err(ServiceError::from)
    }

    /// Applies sensible pagination defaults (page ≥ 1, per_page ≤ 100), then
    /// delegates to the repository.
    async fn find_all(&self, mut parameters: SearchParams) -> Result<PaginatedResponse<NoteResponse>, ServiceError> {
        parameters.page = Some(parameters.page.unwrap_or(DEFAULT_PAGE).max(1));
        parameters.per_page = Some(parameters.per_page.unwrap_or(DEFAULT_PER_PAGE).min(MAX_PER_PAGE));

        self.repository.find_all(parameters).await.map_err(ServiceError::from)
    }

    /// Validates the incoming request and delegates to the repository to
    /// update the existing note.
    async fn update(&self, id: i64, request: UpdateNoteRequest) -> Result<NoteResponse, ServiceError> {
        request.validate()?;

        self.repository.update(id, request).await.map_err(ServiceError::from)
    }

    /// Delegates the deletion to the repository, translating any resulting
    /// error.
    async fn delete(&self, id: i64) -> Result<(), ServiceError> {
        self.repository.delete(id).await.map_err(ServiceError::from)
    }
}
