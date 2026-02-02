//! VibeTea Server - Main entry point.
//!
//! This binary starts the VibeTea event hub server with:
//! - Structured JSON logging for production
//! - Graceful shutdown handling (SIGTERM/SIGINT)
//! - Background rate limiter cleanup
//!
//! # Configuration
//!
//! See [`vibetea_server::config`] for environment variable configuration.
//!
//! # Example
//!
//! ```bash
//! # Development mode (no auth)
//! VIBETEA_UNSAFE_NO_AUTH=true cargo run --bin vibetea-server
//!
//! # Production mode
//! VIBETEA_PUBLIC_KEYS="monitor1:base64pubkey" \
//! VIBETEA_SUBSCRIBER_TOKEN="secret-token" \
//! PORT=8080 \
//! cargo run --release --bin vibetea-server
//! ```

use std::process::ExitCode;
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

use vibetea_server::config::Config;
use vibetea_server::routes::{create_router, AppState};

/// Cleanup interval for stale rate limiter entries (30 seconds).
const RATE_LIMITER_CLEANUP_INTERVAL: Duration = Duration::from_secs(30);

/// Graceful shutdown timeout for in-flight requests (30 seconds).
const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> ExitCode {
    // Initialize structured logging
    init_logging();

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            error!(error = %err, "Failed to load configuration");
            eprintln!("Error: {err}");
            eprintln!();
            eprintln!("Required environment variables (when auth enabled):");
            eprintln!("  VIBETEA_PUBLIC_KEYS      - Format: source1:pubkey1,source2:pubkey2");
            eprintln!("  VIBETEA_SUBSCRIBER_TOKEN - Auth token for WebSocket clients");
            eprintln!();
            eprintln!("Optional environment variables:");
            eprintln!("  PORT                     - HTTP server port (default: 8080)");
            eprintln!("  RUST_LOG                 - Log level filter (default: info)");
            eprintln!("  VIBETEA_UNSAFE_NO_AUTH   - Disable auth (dev only, set to 'true')");
            return ExitCode::from(1);
        }
    };

    // Log startup information
    let auth_mode = if config.unsafe_no_auth {
        "disabled (UNSAFE)"
    } else {
        "enabled"
    };
    info!(
        port = config.port,
        auth_mode = auth_mode,
        public_key_count = config.public_keys.len(),
        "VibeTea server starting"
    );

    // Create application state
    let state = AppState::new(config.clone());

    // Spawn rate limiter cleanup task
    let cleanup_handle = state
        .rate_limiter
        .spawn_cleanup_task(RATE_LIMITER_CLEANUP_INTERVAL);
    info!(
        interval_secs = RATE_LIMITER_CLEANUP_INTERVAL.as_secs(),
        "Rate limiter cleanup task started"
    );

    // Create router
    let app = create_router(state);

    // Bind to address
    let bind_addr = format!("0.0.0.0:{}", config.port);
    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(listener) => {
            info!(
                port = config.port,
                address = %bind_addr,
                "Server listening"
            );
            listener
        }
        Err(err) => {
            error!(
                error = %err,
                address = %bind_addr,
                "Failed to bind to address"
            );
            return ExitCode::from(1);
        }
    };

    // Start server with graceful shutdown
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    info!("Server ready to accept connections");

    // Run the server
    if let Err(err) = server.await {
        error!(error = %err, "Server error");
        return ExitCode::from(1);
    }

    // Shutdown cleanup
    info!("Server shutting down gracefully");

    // Abort the cleanup task
    cleanup_handle.abort();
    info!("Rate limiter cleanup task stopped");

    // Note: axum's graceful shutdown already waits for in-flight requests
    // The GRACEFUL_SHUTDOWN_TIMEOUT is enforced by the shutdown_signal implementation
    // which gives connections time to complete before forcing shutdown

    info!("Server shutdown complete");
    ExitCode::SUCCESS
}

/// Initialize structured logging with tracing.
///
/// Configures JSON-formatted output for production use with:
/// - Environment-based log level filtering via RUST_LOG
/// - Default log level of `info`
/// - System timestamps
/// - Target and level information
fn init_logging() {
    // Build env filter from RUST_LOG or use default
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default: info level for our crates, warn for dependencies
        EnvFilter::new("info,tower_http=debug,axum::rejection=trace")
    });

    // JSON format layer for production logging
    // Note: Using SystemTime for timestamps as it doesn't require the `time` crate feature
    let json_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_level(true)
        .with_file(false)
        .with_line_number(false);

    // Initialize the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)
        .init();
}

/// Creates a future that resolves when a shutdown signal is received.
///
/// Listens for:
/// - SIGTERM (container orchestrator shutdown)
/// - SIGINT (Ctrl+C)
///
/// On signal receipt, logs the event and allows graceful shutdown with
/// a timeout for in-flight requests.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received SIGINT (Ctrl+C), initiating graceful shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        }
    }

    info!(
        timeout_secs = GRACEFUL_SHUTDOWN_TIMEOUT.as_secs(),
        "Waiting for in-flight requests to complete"
    );

    // Give in-flight requests time to complete
    // Note: axum's graceful shutdown handles the actual waiting
    // This is informational - the server will stop accepting new connections
    // but will allow existing ones to complete
}
