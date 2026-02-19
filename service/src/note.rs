//! Note service trait, its implementation, and request validation.
//!
//! The [`NoteService`] trait defines the business operations exposed to the
//! controller layer, whilst [`NoteServiceImpl`] provides the concrete
//! implementation backed by a [`NoteRepository`].

use model::dto::{
    note::{CreateNoteRequest, NoteResponse, UpdateNoteRequest},
    pagination::{PaginatedResponse, SearchParams, SortDirection, SortField, SortFieldName},
};
use repository::note::NoteRepository;
use std::future::Future;

use crate::error::ServiceError;

/// Maximum allowed length for a note title, in characters.
const MAX_TITLE_LEN: usize = 255;

/// Default page number when none is provided by the client.
const DEFAULT_PAGE: u64 = 1;

/// Default page size when none is provided by the client.
const DEFAULT_SIZE: u64 = 20;

/// Hard upper limit on page size to prevent excessively large responses.
const MAX_SIZE: u64 = 100;

/// Validates that a string filter parameter is not blank when present.
///
/// Returns `Ok(())` immediately when the parameter is absent. `name` is the
/// query-string key used verbatim in the error message.
fn validate_string_filter(raw: &Option<String>, name: &str) -> Result<(), ServiceError> {
    let Some(value) = raw else {
        return Ok(());
    };

    if value.trim().is_empty() {
        tracing::warn!(parameter = name, "Validation failed: string filter is blank");
        return Err(ServiceError::Validation(format!("Parameter '{name}' must not be blank")));
    }

    Ok(())
}

/// Validates and parses the `page` query parameter.
///
/// Returns [`DEFAULT_PAGE`] when the parameter is absent. Returns a
/// [`ServiceError::Validation`] when the value is blank or not a valid
/// positive integer. The result is always floored at `1`.
fn validate_page(raw: &Option<String>) -> Result<u64, ServiceError> {
    let Some(raw) = raw else {
        return Ok(DEFAULT_PAGE);
    };

    let trimmed = raw.trim();

    if trimmed.is_empty() {
        tracing::warn!("Validation failed: page is blank");
        return Err(ServiceError::Validation("Parameter 'page' must not be blank".into()));
    }

    trimmed.parse::<u64>().map(|value| value.max(1)).map_err(|_| {
        tracing::warn!(value = trimmed, "Validation failed: page is not a valid positive integer");
        ServiceError::Validation(format!("Parameter 'page' must be a positive integer, got '{trimmed}'"))
    })
}

/// Validates and parses the `size` query parameter.
///
/// Returns [`DEFAULT_SIZE`] when the parameter is absent. Returns a
/// [`ServiceError::Validation`] when the value is blank, not a valid positive
/// integer, or exceeds [`MAX_SIZE`].
fn validate_size(raw: &Option<String>) -> Result<u64, ServiceError> {
    let Some(raw) = raw else {
        return Ok(DEFAULT_SIZE);
    };

    let trimmed = raw.trim();

    if trimmed.is_empty() {
        tracing::warn!("Validation failed: size is blank");
        return Err(ServiceError::Validation("Parameter 'size' must not be blank".into()));
    }

    let value = trimmed.parse::<u64>().map_err(|_| {
        tracing::warn!(value = trimmed, "Validation failed: size is not a valid positive integer");
        ServiceError::Validation(format!("Parameter 'size' must be a positive integer, got '{trimmed}'"))
    })?;

    if value > MAX_SIZE {
        tracing::warn!(size = value, max = MAX_SIZE, "Validation failed: page size too large");
        return Err(ServiceError::Validation(format!("Parameter 'size' must not exceed {MAX_SIZE}")));
    }

    Ok(value)
}

/// Validates and parses the `orderBy` query parameter.
///
/// Returns `Ok(None)` immediately when the parameter is absent. Returns a
/// [`ServiceError::Validation`] when the string is blank or contains only
/// commas, or when a field name is unrecognised. Each token may be prefixed
/// with `+` (ascending, default) or `-` (descending).
fn validate_order_by(raw: &Option<String>) -> Result<Option<Vec<SortField>>, ServiceError> {
    let Some(raw) = raw else {
        return Ok(None);
    };

    let fields: Vec<SortField> = raw
        .split(',')
        .map(str::trim)
        .filter(|string| !string.is_empty())
        .map(|token| {
            let (direction, name) = if let Some(rest) = token.strip_prefix('-') {
                (SortDirection::Descending, rest)
            } else {
                (SortDirection::Ascending, token.strip_prefix('+').unwrap_or(token))
            };

            let name: SortFieldName = name.parse().map_err(|err: String| {
                tracing::warn!(field = name, "Validation failed: unknown sort field");
                ServiceError::Validation(err)
            })?;

            Ok(SortField { name, direction })
        })
        .collect::<Result<Vec<SortField>, ServiceError>>()?;

    if fields.is_empty() {
        tracing::warn!("Validation failed: orderBy is present but contains no fields");
        return Err(ServiceError::Validation(format!(
            "Parameter 'orderBy' must contain at least one field. Valid fields: {}",
            SortFieldName::all_names()
        )));
    }

    Ok(Some(fields))
}

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

/// Internal validation trait implemented by request DTOs and query
/// parameter types.
trait Validate {
    /// Validates (and, where appropriate, normalises) the receiver.
    ///
    /// Returns `Ok(())` when the payload is valid, or a
    /// [`ServiceError::Validation`] describing the first violation found.
    fn validate(&mut self) -> Result<(), ServiceError>;
}

impl Validate for CreateNoteRequest {
    fn validate(&mut self) -> Result<(), ServiceError> {
        if self.title.trim().is_empty() {
            tracing::warn!("Validation failed: title is empty");
            return Err(ServiceError::Validation("Field 'title' must not be empty".into()));
        }

        if self.title.len() > MAX_TITLE_LEN {
            tracing::warn!(length = self.title.len(), max = MAX_TITLE_LEN, "Validation failed: title too long");
            return Err(ServiceError::Validation(format!(
                "Field 'title' must be at most {MAX_TITLE_LEN} characters"
            )));
        }

        if self.content.trim().is_empty() {
            tracing::warn!("Validation failed: content is empty");
            return Err(ServiceError::Validation("Field 'content' must not be empty".into()));
        }

        Ok(())
    }
}

impl Validate for UpdateNoteRequest {
    fn validate(&mut self) -> Result<(), ServiceError> {
        if let Some(ref title) = self.title {
            if title.trim().is_empty() {
                tracing::warn!("Validation failed: title is empty");
                return Err(ServiceError::Validation("Field 'title' must not be empty".into()));
            }

            if title.len() > MAX_TITLE_LEN {
                tracing::warn!(length = title.len(), max = MAX_TITLE_LEN, "Validation failed: title too long");
                return Err(ServiceError::Validation(format!(
                    "Field 'title' must be at most {MAX_TITLE_LEN} characters"
                )));
            }
        }

        if let Some(ref content) = self.content {
            if content.trim().is_empty() {
                tracing::warn!("Validation failed: content is empty");
                return Err(ServiceError::Validation("Field 'content' must not be empty".into()));
            }
        }

        Ok(())
    }
}

impl Validate for SearchParams {
    fn validate(&mut self) -> Result<(), ServiceError> {
        validate_string_filter(&self.title, "title")?;
        validate_string_filter(&self.content, "content")?;

        self.parsed_page = validate_page(&self.page)?;
        self.parsed_size = validate_size(&self.size)?;
        self.sort_fields = validate_order_by(&self.order_by)?.unwrap_or_default();

        Ok(())
    }
}

impl<Repo: NoteRepository> NoteService for NoteServiceImpl<Repo> {
    /// Validates the incoming request and delegates to the repository to
    /// persist the new note.
    #[tracing::instrument(skip_all)]
    async fn create(&self, mut request: CreateNoteRequest) -> Result<NoteResponse, ServiceError> {
        request.validate()?;

        self.repository.create(request).await.map_err(ServiceError::from)
    }

    /// Fetches a single note by ID, translating repository errors into
    /// service-layer errors.
    #[tracing::instrument(skip_all)]
    async fn find_by_id(&self, id: i64) -> Result<NoteResponse, ServiceError> {
        self.repository.find_by_id(id).await.map_err(ServiceError::from)
    }

    /// Validates and normalises search parameters, then delegates to the
    /// repository.
    #[tracing::instrument(skip_all)]
    async fn find_all(&self, mut parameters: SearchParams) -> Result<PaginatedResponse<NoteResponse>, ServiceError> {
        parameters.validate()?;

        self.repository.find_all(parameters).await.map_err(ServiceError::from)
    }

    /// Validates the incoming request and delegates to the repository to
    /// update the existing note.
    #[tracing::instrument(skip_all)]
    async fn update(&self, id: i64, mut request: UpdateNoteRequest) -> Result<NoteResponse, ServiceError> {
        request.validate()?;

        self.repository.update(id, request).await.map_err(ServiceError::from)
    }

    /// Delegates the deletion to the repository, translating any resulting
    /// error.
    #[tracing::instrument(skip_all)]
    async fn delete(&self, id: i64) -> Result<(), ServiceError> {
        self.repository.delete(id).await.map_err(ServiceError::from)
    }
}
