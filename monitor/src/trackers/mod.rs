//! Enhanced data tracking modules for Claude Code monitoring.
//!
//! This module contains specialized trackers for various Claude Code data sources:
//!
//! - [`stats_tracker`]: Token usage, session metrics, activity patterns, model distribution
//! - [`agent_tracker`]: Task tool agent spawn events
//! - [`skill_tracker`]: Skill/slash command invocations from history.jsonl
//! - [`todo_tracker`]: Todo list progress and abandonment detection
//! - [`file_history_tracker`]: File edit line change tracking
//! - [`project_tracker`]: Active project session tracking
//!
//! # Privacy
//!
//! All trackers follow the privacy-first principle: only metadata and aggregate
//! metrics are extracted. No code content, prompts, or file contents are transmitted.

pub mod agent_tracker;
pub mod file_history_tracker;
pub mod project_tracker;
pub mod skill_tracker;
pub mod stats_tracker;
pub mod todo_tracker;

// Re-export StatsEvent for convenience
pub use stats_tracker::StatsEvent;
