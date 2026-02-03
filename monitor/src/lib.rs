//! VibeTea Monitor - Claude Code session watcher.
//!
//! This crate provides functionality for monitoring Claude Code sessions
//! and reporting events to the VibeTea server.
//!
//! # Overview
//!
//! The monitor watches `~/.claude/projects/**/*.jsonl` files for changes and emits
//! events when files are created, modified, or removed. These events can be used
//! to track Claude Code session activity in real-time.
//!
//! # Privacy
//!
//! All parsing is privacy-first: only metadata (tool names, timestamps, file basenames)
//! is extracted, never code content, prompts, or assistant responses. The [`privacy`]
//! module provides additional sanitization before events are transmitted.
//!
//! # Modules
//!
//! - [`types`]: Event types for session monitoring
//! - [`watcher`]: File system watcher for JSONL files
//! - [`parser`]: Claude Code JSONL parsing
//! - [`config`]: Configuration from environment variables
//! - [`error`]: Error types for monitor operations
//! - [`privacy`]: Privacy pipeline for sanitizing event payloads
//! - [`crypto`]: Ed25519 keypair generation and event signing
//! - [`sender`]: HTTP client with retry, buffering, and rate limiting
//! - [`trackers`]: Enhanced data tracking modules
//! - [`tui`]: Terminal user interface for interactive monitoring
//! - [`utils`]: Shared utilities (debouncing, etc.)

pub mod config;
pub mod crypto;
pub mod error;
pub mod parser;
pub mod privacy;
pub mod sender;
pub mod trackers;
pub mod tui;
pub mod types;
pub mod utils;
pub mod watcher;

pub use config::Config;
pub use crypto::{Crypto, CryptoError};
pub use error::{MonitorError, Result};
pub use parser::{ParsedEvent, ParsedEventKind, SessionParser};
pub use privacy::{extract_basename, PrivacyConfig, PrivacyPipeline};
pub use sender::{RetryPolicy, Sender, SenderConfig, SenderError};
pub use types::{Event, EventPayload, EventType, SessionAction, ToolStatus};
pub use utils::{Debouncer, DebouncerError, DEFAULT_DEBOUNCE_MS};
pub use watcher::{check_inotify_usage, FileWatcher, InotifyUsage, WatchEvent, WatcherError};
