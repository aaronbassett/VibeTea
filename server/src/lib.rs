//! VibeTea Server - Real-time event hub.
//!
//! This crate provides the server component of VibeTea, responsible for:
//! - Receiving events from monitors
//! - Authenticating and validating events
//! - Broadcasting events to subscribed clients
//!
//! # Architecture
//!
//! The server acts as a hub between monitors (event producers) and clients
//! (event consumers). Events are validated and broadcast in real-time without
//! persistent storage.
//!
//! # HTTP API
//!
//! The server exposes the following endpoints:
//!
//! - `POST /events` - Ingest events from monitors (requires authentication)
//! - `GET /ws` - WebSocket subscription for clients (requires token)
//! - `GET /health` - Health check endpoint (no authentication)
//!
//! # Example
//!
//! ```rust,no_run
//! use vibetea_server::routes::{create_router, AppState};
//! use vibetea_server::config::Config;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::from_env()?;
//!     let state = AppState::new(config);
//!     let app = create_router(state);
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
//!     axum::serve(listener, app).await?;
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod broadcast;
pub mod config;
pub mod error;
pub mod rate_limit;
pub mod routes;
pub mod session;
pub mod supabase;
pub mod types;
