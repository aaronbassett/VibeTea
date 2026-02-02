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

pub mod config;
pub mod error;
pub mod types;
