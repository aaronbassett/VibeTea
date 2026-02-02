//! VibeTea Monitor - Claude Code session watcher.
//!
//! This crate provides functionality for monitoring Claude Code sessions
//! and reporting events to the VibeTea server.

pub mod types;

pub use types::{Event, EventPayload, EventType, SessionAction, ToolStatus};
