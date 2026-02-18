//! Note repository trait and its SeaORM-backed implementation.
//!
//! The [`NoteRepository`] trait defines the persistence contract for notes,
//! whilst [`NoteRepositoryImpl`] fulfils it using a [`DatabaseConnection`].

use chrono::{DateTime, Utc};
use model::{
    dto::{
        note::{CreateNoteRequest, NoteResponse, UpdateNoteRequest},
        pagination::{PaginatedResponse, SearchParams},
    },
    entity::note,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction, DeleteResult, EntityTrait, PaginatorTrait, QueryFilter,
    TransactionTrait,
};
use std::future::Future;

use crate::error::NoteRepositoryError;

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
    fn to_response(&self, note_model: note::Model) -> NoteResponse {
        NoteResponse {
            id: note_model.id,
            title: note_model.title,
            content: note_model.content,
            created_at: note_model.created_at.into(),
            updated_at: note_model.updated_at.into(),
        }
    }
}

impl NoteRepository for NoteRepositoryImpl {
    /// Inserts a new note row and returns the created record as a response DTO.
    async fn create(&self, req: CreateNoteRequest) -> Result<NoteResponse, NoteRepositoryError> {
        let new_note = note::ActiveModel {
            title: Set(req.title),
            content: Set(req.content),
            ..Default::default()
        };

        let note_model: note::Model = new_note.insert(&self.database).await?;

        Ok(self.to_response(note_model))
    }

    /// Fetches a single note by ID, returning [`NoteRepositoryError::NotFound`]
    /// if no matching row exists.
    async fn find_by_id(&self, id: i64) -> Result<NoteResponse, NoteRepositoryError> {
        let note_model = note::Entity::find_by_id(id)
            .one(&self.database)
            .await?
            .ok_or(NoteRepositoryError::NotFound(id))?;

        Ok(self.to_response(note_model))
    }

    /// Queries notes with optional title filtering, ordered by ID ascending,
    /// and returns a paginated response.
    async fn find_all(&self, parameters: SearchParams) -> Result<PaginatedResponse<NoteResponse>, NoteRepositoryError> {
        let page: u64 = parameters.page.unwrap();
        let per_page: u64 = parameters.per_page.unwrap();

        let mut query = note::Entity::find();

        if let Some(ref title) = parameters.title {
            query = query.filter(note::Column::Title.contains(title.as_str()));
        }

        let sorted_query = query.order_by_id_asc();

        let paginator = sorted_query.paginate(&self.database, per_page);

        let total: u64 = paginator.num_items().await?;
        let notes: Vec<note::Model> = paginator.fetch_page(page - 1).await?;

        let data: Vec<NoteResponse> = notes.into_iter().map(|model: note::Model| self.to_response(model)).collect();

        Ok(PaginatedResponse { data, total, page, per_page })
    }

    /// Updates a note inside a transaction, touching only the fields present
    /// in the request, and stamps the current UTC time on `updated_at`.
    async fn update(&self, id: i64, req: UpdateNoteRequest) -> Result<NoteResponse, NoteRepositoryError> {
        let transaction: DatabaseTransaction = self.database.begin().await?;

        let note_model: note::Model = note::Entity::find_by_id(id)
            .one(&transaction)
            .await?
            .ok_or(NoteRepositoryError::NotFound(id))?;

        let mut active_note_model: note::ActiveModel = note_model.into();

        let now: DateTime<Utc> = Utc::now();
        active_note_model.updated_at = Set(now);

        if let Some(title) = req.title {
            active_note_model.title = Set(title);
        }

        if let Some(content) = req.content {
            active_note_model.content = Set(content);
        }

        let updated_note_model: note::Model = active_note_model.update(&transaction).await?;
        transaction.commit().await?;

        Ok(self.to_response(updated_note_model))
    }

    /// Deletes a note by ID, returning [`NoteRepositoryError::NotFound`] if no
    /// rows were affected.
    async fn delete(&self, id: i64) -> Result<(), NoteRepositoryError> {
        let delete_result: DeleteResult = note::Entity::delete_by_id(id).exec(&self.database).await?;

        if delete_result.rows_affected == 0 {
            return Err(NoteRepositoryError::NotFound(id));
        }

        Ok(())
    }
}
