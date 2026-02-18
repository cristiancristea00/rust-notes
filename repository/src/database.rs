//! Database connection management.
//!
//! Provides [`DatabaseManager`], a thin wrapper around a SeaORM
//! [`DatabaseConnection`] that applies backend-specific pool settings
//! for SQLite and PostgreSQL.

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;

/// Manages the database connection lifecycle with backend-aware configuration.
///
/// On construction the manager inspects the URL scheme to determine whether the
/// target database is SQLite or PostgreSQL, then configures connection-pool
/// parameters that are appropriate for each backend (e.g. a single connection
/// for SQLite, a larger pool for PostgreSQL).
#[derive(Debug, Clone)]
pub struct DatabaseManager {
    /// The underlying SeaORM database connection.
    connection: DatabaseConnection,
}

impl DatabaseManager {
    /// Creates a new [`DatabaseManager`] by connecting to the database at the
    /// given URL and applying backend-specific pool optimisations.
    ///
    /// # Errors
    ///
    /// Returns a [`DbErr`] if the connection cannot be established.
    pub async fn new(database_url: &str) -> Result<Self, DbErr> {
        let is_sqlite = database_url.starts_with("sqlite");
        let connection_options = Self::build_options(database_url, is_sqlite);
        let connection = Database::connect(connection_options).await?;

        Ok(Self { connection })
    }

    /// Returns a shared reference to the underlying database connection.
    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    /// Consumes the manager, returning the underlying database connection.
    pub fn into_connection(self) -> DatabaseConnection {
        self.connection
    }

    /// Builds backend-specific [`ConnectOptions`].
    ///
    /// * **SQLite** – single connection, long idle timeout, no maximum
    ///   lifetime, and SQLx logging disabled.
    /// * **PostgreSQL** – larger pool (2–20), shorter idle timeout, 30-minute
    ///   maximum lifetime, and SQLx logging disabled.
    fn build_options(database_url: &str, is_sqlite: bool) -> ConnectOptions {
        let mut options = ConnectOptions::new(database_url);

        if is_sqlite {
            options
                .min_connections(1)
                .max_connections(1)
                .connect_timeout(Duration::from_secs(5))
                .acquire_timeout(Duration::from_secs(5))
                .idle_timeout(Duration::from_secs(600))
                .max_lifetime(None)
                .sqlx_logging(false);
        } else {
            options
                .min_connections(2)
                .max_connections(20)
                .connect_timeout(Duration::from_secs(10))
                .acquire_timeout(Duration::from_secs(10))
                .idle_timeout(Duration::from_secs(300))
                .max_lifetime(Duration::from_secs(1800))
                .sqlx_logging(false);
        }

        options
    }
}
