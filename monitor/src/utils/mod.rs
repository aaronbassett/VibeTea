//! Utility modules for the VibeTea monitor.
//!
//! This module contains shared utilities used across the monitor crate.
//!
//! # Modules
//!
//! - [`debounce`]: Event debouncing for coalescing rapid file system events

pub mod debounce;

pub use debounce::{Debouncer, DebouncerError, DEFAULT_DEBOUNCE_MS};
