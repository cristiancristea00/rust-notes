//! Application entry point for the notes REST API.
//!
//! Reads configuration from environment variables, initialises the database
//! connection, runs pending migrations, and starts the Axum HTTP server.

mod logging;

use anyhow::Result;
use axum::Router;
use controller::AppRouter;
use migration::MigratorTrait;
use repository::database::DatabaseManager;
use repository::note::NoteRepositoryImpl;
use service::note::NoteServiceImpl;
use tokio::net::TcpListener;

/// Environment variable key for the database connection URL.
const ENV_DATABASE_URL: &str = "DATABASE_URL";

/// Environment variable key for the server bind hostname.
const ENV_SERVER_HOSTNAME: &str = "SERVER_HOSTNAME";

/// Environment variable key for the server bind port.
const ENV_SERVER_PORT: &str = "SERVER_PORT";

/// Fallback database URL when `DATABASE_URL` is not set (in-memory SQLite).
const DEFAULT_DATABASE_URL: &str = "sqlite::memory:";

/// Fallback hostname when `SERVER_HOSTNAME` is not set.
const DEFAULT_SERVER_HOSTNAME: &str = "localhost";

/// Fallback port when `SERVER_PORT` is not set.
const DEFAULT_SERVER_PORT: &str = "8080";

/// Bootstraps the database, runs migrations, wires all layers together, and
/// starts serving HTTP requests.
#[tokio::main]
async fn main() -> Result<()> {
    logging::init();

    let database_url: String = std::env::var(ENV_DATABASE_URL).unwrap_or_else(|_| DEFAULT_DATABASE_URL.into());

    tracing::info!(url = %database_url, "Connecting to database");
    let database_manager = DatabaseManager::new(&database_url).await?;

    tracing::info!("Running database migrations");
    migration::Migrator::up(database_manager.connection(), None).await?;

    let repository = NoteRepositoryImpl::new(database_manager.into_connection());
    let service = NoteServiceImpl::new(repository);
    let router: Router = AppRouter::new(service).into();

    let server_hostname: String = std::env::var(ENV_SERVER_HOSTNAME).unwrap_or_else(|_| DEFAULT_SERVER_HOSTNAME.into());
    let server_port: String = std::env::var(ENV_SERVER_PORT).unwrap_or_else(|_| DEFAULT_SERVER_PORT.into());
    let server_url: String = format!("{server_hostname}:{server_port}");

    let listener = TcpListener::bind(server_url.as_str()).await?;
    tracing::info!(address = %server_url, "Server listening");
    axum::serve(listener, router).await?;

    Ok(())
}
