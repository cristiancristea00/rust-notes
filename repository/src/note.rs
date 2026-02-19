//! Note repository trait and its SeaORM-backed implementation.
//!
//! The [`NoteRepository`] trait defines the persistence contract for notes,
//! whilst [`NoteRepositoryImpl`] fulfils it using a [`DatabaseConnection`].

use chrono::Utc;
use model::{
    dto::{
        note::{CreateNoteRequest, NoteResponse, UpdateNoteRequest},
        pagination::{PageInfo, PaginatedResponse, SearchParams},
    },
    entity::note,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction, DeleteResult, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, Select, TransactionTrait,
};
use std::future::Future;

use crate::{
    error::NoteRepositoryError,
    sort::{IntoColumn, IntoOrder},
};

/// Trait abstracting CRUD operations for notes.
///
/// Implementations must be [`Send`], [`Sync`], [`Clone`], and `'static` so
/// that they can be shared across Axum handler threads.
pub trait NoteRepository: Send + Sync + Clone + 'static {
    /// Persists a new note and returns its full representation.
    fn create(&self, req: CreateNoteRequest) -> impl Future<Output = Result<NoteResponse, NoteRepositoryError>> + Send;

    /// Retrieves a single note by its primary key.
    fn find_by_id(&self, id: i64) -> impl Future<Output = Result<NoteResponse, NoteRepositoryError>> + Send;

    /// Returns a paginated list of notes matching the given search parameters.
    fn find_all(&self, parameters: SearchParams) -> impl Future<Output = Result<PaginatedResponse<NoteResponse>, NoteRepositoryError>> + Send;

    /// Partially updates an existing note and returns its updated representation.
    fn update(&self, id: i64, req: UpdateNoteRequest) -> impl Future<Output = Result<NoteResponse, NoteRepositoryError>> + Send;

    /// Deletes a note by its primary key.
    fn delete(&self, id: i64) -> impl Future<Output = Result<(), NoteRepositoryError>> + Send;
}

/// Concrete [`NoteRepository`] backed by a SeaORM [`DatabaseConnection`].
#[derive(Clone)]
pub struct NoteRepositoryImpl {
    /// The SeaORM database connection used for all queries.
    database: DatabaseConnection,
}

impl NoteRepositoryImpl {
    /// Creates a new [`NoteRepositoryImpl`] wrapping the given database connection.
    pub fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }

    /// Converts a SeaORM [`note::Model`] into a [`NoteResponse`] DTO.
    fn to_response(&self, model: note::Model) -> NoteResponse {
        NoteResponse {
            id: model.id,
            title: model.title,
            content: model.content,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }

    /// Builds a filtered and sorted [`Select`] query from the given
    /// [`SearchParams`].
    ///
    /// Applies an optional title substring filter and the caller-supplied sort
    /// fields in order. Falls back to ascending ID order when no sort fields
    /// are present.
    fn build_note_query(&self, parameters: &SearchParams) -> Select<note::Entity> {
        let mut query = note::Entity::find();

        if let Some(ref title) = parameters.title {
            query = query.filter(note::Column::Title.contains(title.as_str()));
        }

        if let Some(ref content) = parameters.content {
            query = query.filter(note::Column::Content.contains(content.as_str()));
        }

        if parameters.sort_fields.is_empty() {
            return query.order_by(note::Column::Id, Order::Asc);
        }

        for sort_field in &parameters.sort_fields {
            query = query.order_by(sort_field.name.into_column(), sort_field.direction.into_order());
        }

        query
    }

    /// Constructs a [`PageInfo`] from pagination state and the total element
    /// count.
    fn build_page_info(page: u64, size: u64, total: u64) -> PageInfo {
        let total_pages = total.div_ceil(size);
        PageInfo {
            size,
            number: if total_pages == 0 { 0 } else { page },
            total_elements: total,
            total_pages,
        }
    }

    /// Fetches a note by ID within an active transaction and returns it as an
    /// [`ActiveModel`](note::ActiveModel) ready for mutation.
    ///
    /// Returns [`NoteRepositoryError::NotFound`] when no matching row exists.
    async fn find_note_in_transaction(&self, id: i64, transaction: &DatabaseTransaction) -> Result<note::ActiveModel, NoteRepositoryError> {
        let model = note::Entity::find_by_id(id)
            .one(transaction)
            .await?
            .ok_or(NoteRepositoryError::NotFound(id))?;

        Ok(model.into())
    }

    /// Applies the fields from an [`UpdateNoteRequest`] to an active model,
    /// stamping `updated_at` to the current UTC time regardless of which
    /// fields were provided.
    fn apply_update_fields(active: &mut note::ActiveModel, req: UpdateNoteRequest) {
        active.updated_at = Set(Utc::now());

        if let Some(title) = req.title {
            active.title = Set(title);
        }

        if let Some(content) = req.content {
            active.content = Set(content);
        }
    }
}

impl NoteRepository for NoteRepositoryImpl {
    /// Inserts a new note row and returns the created record as a response DTO.
    #[tracing::instrument(skip_all)]
    async fn create(&self, req: CreateNoteRequest) -> Result<NoteResponse, NoteRepositoryError> {
        let new_note = note::ActiveModel {
            title: Set(req.title),
            content: Set(req.content),
            ..Default::default()
        };

        let note_model: note::Model = new_note.insert(&self.database).await?;
        tracing::debug!(id = note_model.id, "Note inserted");

        Ok(self.to_response(note_model))
    }

    /// Fetches a single note by ID, returning [`NoteRepositoryError::NotFound`]
    /// if no matching row exists.
    #[tracing::instrument(skip_all)]
    async fn find_by_id(&self, id: i64) -> Result<NoteResponse, NoteRepositoryError> {
        tracing::debug!(id, "Fetching note by ID");

        let note_model = note::Entity::find_by_id(id)
            .one(&self.database)
            .await?
            .ok_or(NoteRepositoryError::NotFound(id))?;

        Ok(self.to_response(note_model))
    }

    /// Queries notes with optional filtering and caller-specified ordering,
    /// and returns a paginated response.
    #[tracing::instrument(skip_all)]
    async fn find_all(&self, parameters: SearchParams) -> Result<PaginatedResponse<NoteResponse>, NoteRepositoryError> {
        let page = parameters.parsed_page;
        let size = parameters.parsed_size;

        tracing::debug!(page, size, "Fetching paginated notes");

        let paginator = self.build_note_query(&parameters).paginate(&self.database, size);
        let total = paginator.num_items().await?;
        let models = paginator.fetch_page(page - 1).await?;

        tracing::debug!(total, count = models.len(), "Query completed");

        let notes = models.into_iter().map(|m| self.to_response(m)).collect();

        Ok(PaginatedResponse {
            notes,
            page: Self::build_page_info(page, size, total),
        })
    }

    /// Updates a note inside a transaction, touching only the fields present
    /// in the request, and stamps the current UTC time on `updated_at`.
    #[tracing::instrument(skip_all)]
    async fn update(&self, id: i64, req: UpdateNoteRequest) -> Result<NoteResponse, NoteRepositoryError> {
        tracing::debug!(id, "Updating note");

        let transaction = self.database.begin().await?;
        let mut active = self.find_note_in_transaction(id, &transaction).await?;

        Self::apply_update_fields(&mut active, req);

        let updated = active.update(&transaction).await?;
        transaction.commit().await?;

        tracing::debug!(id, "Note updated");

        Ok(self.to_response(updated))
    }

    /// Deletes a note by ID, returning [`NoteRepositoryError::NotFound`] if no
    /// rows were affected.
    #[tracing::instrument(skip_all)]
    async fn delete(&self, id: i64) -> Result<(), NoteRepositoryError> {
        tracing::debug!(id, "Deleting note");

        let delete_result: DeleteResult = note::Entity::delete_by_id(id).exec(&self.database).await?;

        if delete_result.rows_affected == 0 {
            return Err(NoteRepositoryError::NotFound(id));
        }

        Ok(())
    }
}
