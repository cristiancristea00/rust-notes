//! Application-wide logging initialisation.
//!
//! Configures a [`tracing_subscriber`] layer that outputs log events to the
//! console using the default formatter. The log level can be controlled via
//! the `RUST_LOG` environment variable (e.g. `RUST_LOG=warn`); if unset, it
//! defaults to `debug`.

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialises the global tracing subscriber with the default formatter.
///
/// The log level can be controlled via the `RUST_LOG` environment variable
/// (e.g. `RUST_LOG=warn`). If unset, it defaults to `debug`.
pub fn init() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")))
        .with(fmt::layer())
        .init();
}
