//! Utility modules for the VibeTea monitor.
//!
//! This module contains shared utilities used across the monitor crate.
//!
//! # Modules
//!
//! - [`debounce`]: Event debouncing for coalescing rapid file system events
//! - [`session_filename`]: Session filename parser for Claude Code file paths
//! - [`tokenize`]: Shell-like tokenizer for skill name extraction

pub mod debounce;
pub mod session_filename;
pub mod tokenize;

pub use debounce::{Debouncer, DebouncerError, DEFAULT_DEBOUNCE_MS};
pub use session_filename::{
    parse_file_history_path, parse_session_jsonl_path, parse_todo_filename, FileHistoryInfo,
    SessionJsonlInfo,
};
pub use tokenize::extract_skill_name;
