//! Skill tracker for detecting skill/slash command invocations.
//!
//! This module watches `~/.claude/history.jsonl` for changes and emits
//! [`SkillInvocationEvent`]s for each new skill invocation.
//!
//! # History.jsonl Format
//!
//! When a user invokes a skill (slash command) in Claude Code, an entry is
//! appended to `~/.claude/history.jsonl`:
//!
//! ```json
//! {
//!   "display": "/commit -m \"fix: update docs\"",
//!   "timestamp": 1738567268363,
//!   "project": "/home/ubuntu/Projects/VibeTea",
//!   "sessionId": "6e45a55c-3124-4cc8-ad85-040a5c316009"
//! }
//! ```
//!
//! # Privacy
//!
//! This module follows the privacy-first principle: only the skill name
//! (extracted from `display`) and metadata are captured. Command arguments
//! are intentionally not transmitted.
//!
//! # Architecture
//!
//! The tracker uses the [`notify`] crate to watch for file changes. Since
//! `history.jsonl` is append-only, the tracker maintains a byte offset to
//! only read new lines (tail-like behavior). No debounce is used - events
//! are processed immediately per the research.md specification.
//!
//! # Example
//!
//! ```no_run
//! use tokio::sync::mpsc;
//! use vibetea_monitor::trackers::skill_tracker::SkillTracker;
//! use vibetea_monitor::types::SkillInvocationEvent;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tx, mut rx) = mpsc::channel(100);
//!     let tracker = SkillTracker::new(tx)?;
//!
//!     while let Some(event) = rx.recv().await {
//!         println!("Skill: {}, Session: {}", event.skill_name, event.session_id);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Parsing Functions
//!
//! Lower-level parsing functions are also available for testing or custom use:
//!
//! ```
//! use vibetea_monitor::trackers::skill_tracker::{parse_history_entry, create_skill_invocation_event};
//!
//! let line = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/home/user/project", "sessionId": "abc-123"}"#;
//!
//! if let Ok(entry) = parse_history_entry(line) {
//!     if let Some(event) = create_skill_invocation_event(&entry) {
//!         assert_eq!(event.skill_name, "commit");
//!         assert_eq!(event.session_id, "abc-123");
//!     }
//! }
//! ```

use std::io::{BufRead, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

use crate::types::SkillInvocationEvent;
use crate::utils::tokenize::extract_skill_name;

/// Errors that can occur when parsing history.jsonl entries.
#[derive(Debug, Error)]
pub enum HistoryParseError {
    /// Failed to parse the JSON structure.
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// The `display` field is missing from the entry.
    #[error("missing required field: display")]
    MissingDisplay,

    /// The `timestamp` field is missing from the entry.
    #[error("missing required field: timestamp")]
    MissingTimestamp,

    /// The `project` field is missing from the entry.
    #[error("missing required field: project")]
    MissingProject,

    /// The `sessionId` field is missing from the entry.
    #[error("missing required field: sessionId")]
    MissingSessionId,
}

/// Errors that can occur during skill tracking operations.
#[derive(Error, Debug)]
pub enum SkillTrackerError {
    /// Failed to initialize the file system watcher.
    #[error("failed to create watcher: {0}")]
    WatcherInit(#[from] notify::Error),

    /// Failed to read the history file.
    #[error("failed to read history file: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse a history entry.
    #[error("failed to parse history entry: {0}")]
    Parse(#[from] HistoryParseError),

    /// The Claude directory does not exist.
    #[error("claude directory not found: {0}")]
    ClaudeDirectoryNotFound(PathBuf),

    /// Failed to send event through the channel.
    #[error("failed to send event: channel closed")]
    ChannelClosed,
}

/// Result type for skill tracker operations.
pub type Result<T> = std::result::Result<T, SkillTrackerError>;

/// A parsed entry from history.jsonl.
///
/// Represents a single skill invocation record as stored by Claude Code.
/// The JSON uses camelCase field names which are mapped to snake_case.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    /// The skill command as displayed (e.g., "/commit -m \"message\"").
    pub display: String,

    /// Unix timestamp in milliseconds when the skill was invoked.
    pub timestamp: i64,

    /// Absolute path to the project root where the skill was invoked.
    pub project: String,

    /// UUID of the Claude Code session.
    pub session_id: String,
}

impl HistoryEntry {
    /// Converts the Unix milliseconds timestamp to a [`DateTime<Utc>`].
    ///
    /// # Returns
    ///
    /// The timestamp as a UTC datetime, or `None` if the timestamp is out of range.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::Datelike;
    /// use vibetea_monitor::trackers::skill_tracker::HistoryEntry;
    ///
    /// let entry = HistoryEntry {
    ///     display: "/commit".to_string(),
    ///     timestamp: 1738567268363, // 2025-02-03T09:21:08.363Z
    ///     project: "/home/user/project".to_string(),
    ///     session_id: "abc-123".to_string(),
    /// };
    ///
    /// let dt = entry.to_datetime().unwrap();
    /// assert_eq!(dt.year(), 2025);
    /// ```
    #[must_use]
    pub fn to_datetime(&self) -> Option<DateTime<Utc>> {
        let secs = self.timestamp / 1000;
        let nsecs = ((self.timestamp % 1000) * 1_000_000) as u32;
        Utc.timestamp_opt(secs, nsecs).single()
    }

    /// Extracts the skill name from the display field.
    ///
    /// Uses the tokenizer to handle quoted skill names and arguments.
    ///
    /// # Returns
    ///
    /// The skill name if the display field contains a valid skill command,
    /// or `None` if parsing fails.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::trackers::skill_tracker::HistoryEntry;
    ///
    /// let entry = HistoryEntry {
    ///     display: "/commit -m \"fix: bug\"".to_string(),
    ///     timestamp: 1738567268363,
    ///     project: "/home/user/project".to_string(),
    ///     session_id: "abc-123".to_string(),
    /// };
    ///
    /// assert_eq!(entry.extract_skill_name(), Some("commit".to_string()));
    /// ```
    #[must_use]
    pub fn extract_skill_name(&self) -> Option<String> {
        extract_skill_name(&self.display)
    }
}

/// Parses a single line from history.jsonl into a [`HistoryEntry`].
///
/// # Arguments
///
/// * `line` - A single JSON line from history.jsonl
///
/// # Returns
///
/// * `Ok(HistoryEntry)` if parsing succeeds
/// * `Err(HistoryParseError)` if the JSON is invalid or required fields are missing
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::skill_tracker::parse_history_entry;
///
/// let line = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/home/user/project", "sessionId": "abc-123"}"#;
/// let entry = parse_history_entry(line).unwrap();
///
/// assert_eq!(entry.display, "/commit");
/// assert_eq!(entry.session_id, "abc-123");
/// ```
pub fn parse_history_entry(line: &str) -> std::result::Result<HistoryEntry, HistoryParseError> {
    // First parse as a generic JSON value to provide better error messages
    let value: serde_json::Value = serde_json::from_str(line)?;

    // Check for required fields and provide specific error messages
    if value.get("display").is_none() {
        return Err(HistoryParseError::MissingDisplay);
    }
    if value.get("timestamp").is_none() {
        return Err(HistoryParseError::MissingTimestamp);
    }
    if value.get("project").is_none() {
        return Err(HistoryParseError::MissingProject);
    }
    if value.get("sessionId").is_none() {
        return Err(HistoryParseError::MissingSessionId);
    }

    // Now parse into the struct
    Ok(serde_json::from_value(value)?)
}

/// Parses multiple lines from history.jsonl, returning successfully parsed entries.
///
/// This function is lenient: it skips invalid lines and continues parsing.
/// This is appropriate for append-only files where some lines may be corrupted
/// or from older formats.
///
/// # Arguments
///
/// * `content` - The full content of history.jsonl (or a portion of it)
///
/// # Returns
///
/// A vector of successfully parsed entries. Invalid lines are silently skipped.
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::skill_tracker::parse_history_entries;
///
/// let content = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "a"}
/// invalid json here
/// {"display": "/review-pr", "timestamp": 1738567268400, "project": "/proj", "sessionId": "b"}"#;
///
/// let entries = parse_history_entries(content);
/// assert_eq!(entries.len(), 2); // Invalid line is skipped
/// ```
#[must_use]
pub fn parse_history_entries(content: &str) -> Vec<HistoryEntry> {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| parse_history_entry(line).ok())
        .collect()
}

/// Creates a [`SkillInvocationEvent`] from a [`HistoryEntry`].
///
/// This function extracts the skill name from the display field and creates
/// a complete event structure for transmission.
///
/// # Arguments
///
/// * `entry` - The parsed history entry
///
/// # Returns
///
/// * `Some(SkillInvocationEvent)` if the skill name could be extracted
/// * `None` if the display field doesn't contain a valid skill command
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::skill_tracker::{HistoryEntry, create_skill_invocation_event};
///
/// let entry = HistoryEntry {
///     display: "/commit -m \"fix\"".to_string(),
///     timestamp: 1738567268363,
///     project: "/home/user/project".to_string(),
///     session_id: "sess-123".to_string(),
/// };
///
/// let event = create_skill_invocation_event(&entry).unwrap();
/// assert_eq!(event.skill_name, "commit");
/// assert_eq!(event.session_id, "sess-123");
/// assert_eq!(event.project, "/home/user/project");
/// ```
#[must_use]
pub fn create_skill_invocation_event(entry: &HistoryEntry) -> Option<SkillInvocationEvent> {
    let skill_name = entry.extract_skill_name()?;
    let timestamp = entry.to_datetime()?;

    Some(SkillInvocationEvent {
        session_id: entry.session_id.clone(),
        skill_name,
        project: entry.project.clone(),
        timestamp,
    })
}

// ============================================================================
// SkillTracker - File Watching Implementation
// ============================================================================

/// Configuration for the skill tracker.
#[derive(Debug, Clone, Default)]
pub struct SkillTrackerConfig {
    /// Whether to emit events for existing entries on startup.
    ///
    /// When `true`, the tracker will emit events for all existing entries
    /// in the history file when it starts. When `false` (the default),
    /// only new entries appended after the tracker starts will emit events.
    pub emit_existing_on_startup: bool,
}

/// Tracker for Claude Code's history.jsonl file.
///
/// Watches for file changes and emits [`SkillInvocationEvent`]s when new
/// skill invocations are appended. Uses tail-like behavior to only read
/// new lines, tracking the byte offset within the file.
///
/// # Thread Safety
///
/// The tracker spawns a background task for async processing of file events.
/// Communication is done via channels for thread safety. The byte offset
/// is stored in an atomic for lock-free reads from the watcher callback.
#[derive(Debug)]
pub struct SkillTracker {
    /// The underlying file system watcher.
    ///
    /// Kept alive to maintain the watch subscription.
    #[allow(dead_code)]
    watcher: RecommendedWatcher,

    /// Path to the history.jsonl file.
    history_path: PathBuf,

    /// Channel sender for emitting skill invocation events.
    #[allow(dead_code)]
    event_sender: mpsc::Sender<SkillInvocationEvent>,

    /// Current byte offset in the file (for tail-like behavior).
    #[allow(dead_code)]
    offset: Arc<AtomicU64>,
}

impl SkillTracker {
    /// Creates a new skill tracker watching the default history.jsonl location.
    ///
    /// The default location is `~/.claude/history.jsonl`.
    ///
    /// # Arguments
    ///
    /// * `event_sender` - Channel for emitting [`SkillInvocationEvent`]s
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The home directory cannot be determined
    /// - The `~/.claude` directory does not exist
    /// - The file system watcher cannot be initialized
    pub fn new(event_sender: mpsc::Sender<SkillInvocationEvent>) -> Result<Self> {
        Self::with_config(event_sender, SkillTrackerConfig::default())
    }

    /// Creates a new skill tracker with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `event_sender` - Channel for emitting [`SkillInvocationEvent`]s
    /// * `config` - Configuration options for the tracker
    ///
    /// # Errors
    ///
    /// Returns an error if the tracker cannot be initialized.
    pub fn with_config(
        event_sender: mpsc::Sender<SkillInvocationEvent>,
        config: SkillTrackerConfig,
    ) -> Result<Self> {
        let claude_dir = directories::BaseDirs::new()
            .map(|dirs| dirs.home_dir().join(".claude"))
            .ok_or_else(|| {
                SkillTrackerError::ClaudeDirectoryNotFound(PathBuf::from("~/.claude"))
            })?;

        Self::with_path_and_config(claude_dir.join("history.jsonl"), event_sender, config)
    }

    /// Creates a new skill tracker watching a specific history.jsonl file.
    ///
    /// # Arguments
    ///
    /// * `history_path` - Path to the history.jsonl file to watch
    /// * `event_sender` - Channel for emitting [`SkillInvocationEvent`]s
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory of the history file does not exist
    /// - The file system watcher cannot be initialized
    pub fn with_path(
        history_path: PathBuf,
        event_sender: mpsc::Sender<SkillInvocationEvent>,
    ) -> Result<Self> {
        Self::with_path_and_config(history_path, event_sender, SkillTrackerConfig::default())
    }

    /// Creates a new skill tracker with a specific path and configuration.
    ///
    /// # Arguments
    ///
    /// * `history_path` - Path to the history.jsonl file to watch
    /// * `event_sender` - Channel for emitting [`SkillInvocationEvent`]s
    /// * `config` - Configuration options for the tracker
    ///
    /// # Errors
    ///
    /// Returns an error if the tracker cannot be initialized.
    pub fn with_path_and_config(
        history_path: PathBuf,
        event_sender: mpsc::Sender<SkillInvocationEvent>,
        config: SkillTrackerConfig,
    ) -> Result<Self> {
        // Verify the parent directory exists (file may not exist yet)
        let watch_dir = history_path
            .parent()
            .ok_or_else(|| SkillTrackerError::ClaudeDirectoryNotFound(history_path.clone()))?;

        if !watch_dir.exists() {
            return Err(SkillTrackerError::ClaudeDirectoryNotFound(
                watch_dir.to_path_buf(),
            ));
        }

        info!(
            history_path = %history_path.display(),
            "Initializing skill tracker"
        );

        // Create atomic offset for tracking file position
        let offset = Arc::new(AtomicU64::new(0));

        // Create channel for file change notifications (unbounded effectively with large buffer)
        let (change_tx, change_rx) = mpsc::channel::<PathBuf>(1000);

        // Spawn the async processing task
        let sender_for_task = event_sender.clone();
        let path_for_task = history_path.clone();
        let offset_for_task = Arc::clone(&offset);
        tokio::spawn(async move {
            process_file_changes(change_rx, path_for_task, sender_for_task, offset_for_task).await;
        });

        // Create the file watcher
        let watcher = create_history_watcher(history_path.clone(), change_tx)?;

        // Handle initial read if file exists
        let initial_offset = if history_path.exists() {
            if config.emit_existing_on_startup {
                // Emit all existing entries
                let sender_for_init = event_sender.clone();
                let path_for_init = history_path.clone();
                tokio::spawn(async move {
                    if let Err(e) = emit_all_entries(&path_for_init, &sender_for_init).await {
                        warn!(
                            path = %path_for_init.display(),
                            error = %e,
                            "Failed to read initial history entries"
                        );
                    }
                });
            }
            // Start watching from end of file
            get_file_size(&history_path).unwrap_or(0)
        } else {
            // File doesn't exist yet, start from beginning when it's created
            0
        };

        offset.store(initial_offset, Ordering::SeqCst);
        debug!(initial_offset = initial_offset, "Set initial file offset");

        Ok(Self {
            watcher,
            history_path,
            event_sender,
            offset,
        })
    }

    /// Returns the path to the history.jsonl file being watched.
    #[must_use]
    pub fn history_path(&self) -> &Path {
        &self.history_path
    }

    /// Returns the current byte offset in the file.
    #[must_use]
    pub fn current_offset(&self) -> u64 {
        self.offset.load(Ordering::SeqCst)
    }

    /// Manually triggers a read of new entries since the last offset.
    ///
    /// This is useful for forcing a refresh without waiting for file events.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the event channel is closed.
    pub async fn refresh(&self) -> Result<()> {
        let new_offset = emit_new_entries(
            &self.history_path,
            &self.event_sender,
            self.offset.load(Ordering::SeqCst),
        )
        .await?;
        self.offset.store(new_offset, Ordering::SeqCst);
        Ok(())
    }
}

/// Creates the file system watcher for the history file.
fn create_history_watcher(
    history_path: PathBuf,
    change_tx: mpsc::Sender<PathBuf>,
) -> Result<RecommendedWatcher> {
    let watch_dir = history_path
        .parent()
        .ok_or_else(|| SkillTrackerError::ClaudeDirectoryNotFound(history_path.clone()))?
        .to_path_buf();

    let history_filename = history_path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();

    let mut watcher = RecommendedWatcher::new(
        move |res: std::result::Result<Event, notify::Error>| {
            handle_notify_event(res, &history_path, &change_tx);
        },
        Config::default(),
    )?;

    // Watch the parent directory since the file may not exist yet
    watcher.watch(&watch_dir, RecursiveMode::NonRecursive)?;

    debug!(
        watch_dir = %watch_dir.display(),
        filename = ?history_filename,
        "Started watching for history.jsonl changes"
    );

    Ok(watcher)
}

/// Handles raw notify events and sends them to the processing channel.
fn handle_notify_event(
    res: std::result::Result<Event, notify::Error>,
    history_path: &Path,
    change_tx: &mpsc::Sender<PathBuf>,
) {
    let event = match res {
        Ok(event) => event,
        Err(e) => {
            error!(error = %e, "File watcher error");
            return;
        }
    };

    trace!(kind = ?event.kind, paths = ?event.paths, "Received notify event");

    // Check if any of the event paths match our history file
    let matches_history = event.paths.iter().any(|p| p == history_path);
    if !matches_history {
        return;
    }

    // Only process relevant event kinds
    let should_process = matches!(
        event.kind,
        EventKind::Create(_)
            | EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Any)
    );

    if !should_process {
        trace!(kind = ?event.kind, "Ignoring event kind");
        return;
    }

    debug!(
        path = %history_path.display(),
        kind = ?event.kind,
        "History file changed, sending to processor"
    );

    // No debounce - send immediately (per research.md)
    if let Err(e) = change_tx.try_send(history_path.to_path_buf()) {
        warn!(error = %e, "Failed to send change notification: channel full or closed");
    }
}

/// Processes file change events, reading new lines and emitting events.
async fn process_file_changes(
    mut rx: mpsc::Receiver<PathBuf>,
    history_path: PathBuf,
    sender: mpsc::Sender<SkillInvocationEvent>,
    offset: Arc<AtomicU64>,
) {
    debug!("Starting history file change processor");

    while let Some(path) = rx.recv().await {
        if path != history_path {
            continue;
        }

        debug!(path = %path.display(), "Processing history file change");

        let current_offset = offset.load(Ordering::SeqCst);
        match emit_new_entries(&path, &sender, current_offset).await {
            Ok(new_offset) => {
                offset.store(new_offset, Ordering::SeqCst);
                trace!(
                    old_offset = current_offset,
                    new_offset = new_offset,
                    "Updated file offset"
                );
            }
            Err(e) => {
                warn!(
                    path = %path.display(),
                    error = %e,
                    "Failed to process history file change"
                );
            }
        }
    }

    debug!("History file change processor shutting down");
}

/// Emits events for all entries in the history file.
async fn emit_all_entries(path: &Path, sender: &mpsc::Sender<SkillInvocationEvent>) -> Result<()> {
    let content = tokio::fs::read_to_string(path).await?;
    let entries = parse_history_entries(&content);

    for entry in entries {
        if let Some(event) = create_skill_invocation_event(&entry) {
            trace!(
                skill = %event.skill_name,
                session = %event.session_id,
                "Emitting skill invocation event (initial read)"
            );
            sender
                .send(event)
                .await
                .map_err(|_| SkillTrackerError::ChannelClosed)?;
        }
    }

    Ok(())
}

/// Emits events for new entries since the given offset, returning the new offset.
async fn emit_new_entries(
    path: &Path,
    sender: &mpsc::Sender<SkillInvocationEvent>,
    from_offset: u64,
) -> Result<u64> {
    // Open file and seek to offset
    let file = std::fs::File::open(path)?;
    let file_len = file.metadata()?.len();

    // If file was truncated (smaller than our offset), reset to start
    let actual_offset = if file_len < from_offset {
        warn!(
            path = %path.display(),
            file_len = file_len,
            expected_offset = from_offset,
            "File appears truncated, resetting offset"
        );
        0
    } else {
        from_offset
    };

    // Seek to the offset and read new content
    let mut reader = std::io::BufReader::new(file);
    reader.seek(SeekFrom::Start(actual_offset))?;

    let mut new_content = String::new();
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        new_content.push_str(&line);
        line.clear();
    }

    let new_offset = actual_offset + new_content.len() as u64;

    // Parse and emit events for new entries
    if !new_content.is_empty() {
        let entries = parse_history_entries(&new_content);
        debug!(
            entries_count = entries.len(),
            bytes_read = new_content.len(),
            "Read new history entries"
        );

        for entry in entries {
            if let Some(event) = create_skill_invocation_event(&entry) {
                trace!(
                    skill = %event.skill_name,
                    session = %event.session_id,
                    "Emitting skill invocation event"
                );
                sender
                    .send(event)
                    .await
                    .map_err(|_| SkillTrackerError::ChannelClosed)?;
            }
        }
    }

    Ok(new_offset)
}

/// Gets the size of a file in bytes.
fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    // =========================================================================
    // T093: HistoryEntry Parsing Tests
    // =========================================================================

    #[test]
    fn parse_valid_history_entry() {
        let line = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/home/ubuntu/Projects/VibeTea", "sessionId": "6e45a55c-3124-4cc8-ad85-040a5c316009"}"#;

        let entry = parse_history_entry(line).unwrap();

        assert_eq!(entry.display, "/commit");
        assert_eq!(entry.timestamp, 1738567268363);
        assert_eq!(entry.project, "/home/ubuntu/Projects/VibeTea");
        assert_eq!(entry.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
    }

    #[test]
    fn parse_history_entry_with_args() {
        let line = r#"{"display": "/commit -m \"fix: update docs\"", "timestamp": 1738567268363, "project": "/home/user/project", "sessionId": "abc-123"}"#;

        let entry = parse_history_entry(line).unwrap();

        assert_eq!(entry.display, "/commit -m \"fix: update docs\"");
    }

    #[test]
    fn parse_history_entry_missing_display() {
        let line = r#"{"timestamp": 1738567268363, "project": "/home/user/project", "sessionId": "abc-123"}"#;

        let err = parse_history_entry(line).unwrap_err();

        assert!(matches!(err, HistoryParseError::MissingDisplay));
        assert!(err.to_string().contains("display"));
    }

    #[test]
    fn parse_history_entry_missing_timestamp() {
        let line =
            r#"{"display": "/commit", "project": "/home/user/project", "sessionId": "abc-123"}"#;

        let err = parse_history_entry(line).unwrap_err();

        assert!(matches!(err, HistoryParseError::MissingTimestamp));
        assert!(err.to_string().contains("timestamp"));
    }

    #[test]
    fn parse_history_entry_missing_project() {
        let line = r#"{"display": "/commit", "timestamp": 1738567268363, "sessionId": "abc-123"}"#;

        let err = parse_history_entry(line).unwrap_err();

        assert!(matches!(err, HistoryParseError::MissingProject));
        assert!(err.to_string().contains("project"));
    }

    #[test]
    fn parse_history_entry_missing_session_id() {
        let line = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/home/user/project"}"#;

        let err = parse_history_entry(line).unwrap_err();

        assert!(matches!(err, HistoryParseError::MissingSessionId));
        assert!(err.to_string().contains("sessionId"));
    }

    #[test]
    fn parse_history_entry_invalid_json() {
        let line = "not valid json at all";

        let err = parse_history_entry(line).unwrap_err();

        assert!(matches!(err, HistoryParseError::InvalidJson(_)));
    }

    #[test]
    fn parse_history_entry_empty_string() {
        let line = "";

        let err = parse_history_entry(line).unwrap_err();

        assert!(matches!(err, HistoryParseError::InvalidJson(_)));
    }

    #[test]
    fn parse_history_entry_empty_json_object() {
        let line = "{}";

        let err = parse_history_entry(line).unwrap_err();

        // Should fail on first missing field check
        assert!(matches!(err, HistoryParseError::MissingDisplay));
    }

    #[test]
    fn parse_history_entry_null_values() {
        let line = r#"{"display": null, "timestamp": 1738567268363, "project": "/home/user/project", "sessionId": "abc-123"}"#;

        // null is not a valid string for display
        let err = parse_history_entry(line).unwrap_err();
        assert!(matches!(err, HistoryParseError::InvalidJson(_)));
    }

    #[test]
    fn parse_history_entry_wrong_type_timestamp() {
        let line = r#"{"display": "/commit", "timestamp": "not-a-number", "project": "/proj", "sessionId": "abc"}"#;

        let err = parse_history_entry(line).unwrap_err();
        assert!(matches!(err, HistoryParseError::InvalidJson(_)));
    }

    #[test]
    fn parse_history_entry_extra_fields_ignored() {
        let line = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "abc", "extraField": "ignored", "anotherExtra": 42}"#;

        let entry = parse_history_entry(line).unwrap();

        assert_eq!(entry.display, "/commit");
        assert_eq!(entry.session_id, "abc");
    }

    #[test]
    fn parse_history_entry_unicode_in_display() {
        let line = r#"{"display": "/commit -m \"feat: add support\"", "timestamp": 1738567268363, "project": "/proj", "sessionId": "abc"}"#;

        let entry = parse_history_entry(line).unwrap();

        assert!(entry.display.contains("support"));
    }

    // =========================================================================
    // Multiple Entries Parsing Tests (append-only file simulation)
    // =========================================================================

    #[test]
    fn parse_multiple_entries_all_valid() {
        let content = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "a"}
{"display": "/review-pr", "timestamp": 1738567268400, "project": "/proj", "sessionId": "b"}
{"display": "/sdd:plan", "timestamp": 1738567268500, "project": "/proj", "sessionId": "c"}"#;

        let entries = parse_history_entries(content);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].display, "/commit");
        assert_eq!(entries[1].display, "/review-pr");
        assert_eq!(entries[2].display, "/sdd:plan");
    }

    #[test]
    fn parse_multiple_entries_with_invalid_lines() {
        let content = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "a"}
invalid json line
{"display": "/review-pr", "timestamp": 1738567268400, "project": "/proj", "sessionId": "b"}
{}
{"display": "/sdd:plan", "timestamp": 1738567268500, "project": "/proj", "sessionId": "c"}"#;

        let entries = parse_history_entries(content);

        // Invalid lines are skipped
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].display, "/commit");
        assert_eq!(entries[1].display, "/review-pr");
        assert_eq!(entries[2].display, "/sdd:plan");
    }

    #[test]
    fn parse_multiple_entries_empty_content() {
        let content = "";

        let entries = parse_history_entries(content);

        assert!(entries.is_empty());
    }

    #[test]
    fn parse_multiple_entries_only_whitespace() {
        let content = "   \n\n  \n   ";

        let entries = parse_history_entries(content);

        assert!(entries.is_empty());
    }

    #[test]
    fn parse_multiple_entries_all_invalid() {
        let content = r#"invalid
also invalid
{}"#;

        let entries = parse_history_entries(content);

        assert!(entries.is_empty());
    }

    #[test]
    fn parse_multiple_entries_with_blank_lines() {
        let content = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "a"}

{"display": "/review-pr", "timestamp": 1738567268400, "project": "/proj", "sessionId": "b"}

"#;

        let entries = parse_history_entries(content);

        assert_eq!(entries.len(), 2);
    }

    // =========================================================================
    // HistoryEntry Methods Tests
    // =========================================================================

    #[test]
    fn history_entry_to_datetime_valid() {
        let entry = HistoryEntry {
            display: "/commit".to_string(),
            timestamp: 1738567268363, // 2025-02-03T09:21:08.363Z
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let dt = entry.to_datetime().unwrap();

        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 3);
    }

    #[test]
    fn history_entry_to_datetime_preserves_milliseconds() {
        let entry = HistoryEntry {
            display: "/commit".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let dt = entry.to_datetime().unwrap();
        let millis = dt.timestamp_millis();

        assert_eq!(millis, 1738567268363);
    }

    #[test]
    fn history_entry_to_datetime_zero() {
        let entry = HistoryEntry {
            display: "/commit".to_string(),
            timestamp: 0, // Unix epoch
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let dt = entry.to_datetime().unwrap();

        assert_eq!(dt.year(), 1970);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn history_entry_to_datetime_negative() {
        let entry = HistoryEntry {
            display: "/commit".to_string(),
            timestamp: -86400000, // One day before epoch
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let dt = entry.to_datetime().unwrap();

        assert_eq!(dt.year(), 1969);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.day(), 31);
    }

    // =========================================================================
    // Skill Name Extraction Tests
    // =========================================================================

    #[test]
    fn extract_skill_name_simple() {
        let entry = HistoryEntry {
            display: "/commit".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        assert_eq!(entry.extract_skill_name(), Some("commit".to_string()));
    }

    #[test]
    fn extract_skill_name_with_args() {
        let entry = HistoryEntry {
            display: "/commit -m \"fix: update docs\"".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        assert_eq!(entry.extract_skill_name(), Some("commit".to_string()));
    }

    #[test]
    fn extract_skill_name_with_colon() {
        let entry = HistoryEntry {
            display: "/sdd:plan".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        assert_eq!(entry.extract_skill_name(), Some("sdd:plan".to_string()));
    }

    #[test]
    fn extract_skill_name_review_pr() {
        let entry = HistoryEntry {
            display: "/review-pr 123".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        assert_eq!(entry.extract_skill_name(), Some("review-pr".to_string()));
    }

    #[test]
    fn extract_skill_name_quoted() {
        let entry = HistoryEntry {
            display: "/\"my skill\" arg1".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        assert_eq!(entry.extract_skill_name(), Some("\"my skill\"".to_string()));
    }

    #[test]
    fn extract_skill_name_no_slash() {
        let entry = HistoryEntry {
            display: "not a skill command".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        assert_eq!(entry.extract_skill_name(), None);
    }

    #[test]
    fn extract_skill_name_just_slash() {
        let entry = HistoryEntry {
            display: "/".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        assert_eq!(entry.extract_skill_name(), None);
    }

    #[test]
    fn extract_skill_name_empty_display() {
        let entry = HistoryEntry {
            display: String::new(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        assert_eq!(entry.extract_skill_name(), None);
    }

    // =========================================================================
    // SkillInvocationEvent Creation Tests
    // =========================================================================

    #[test]
    fn create_event_success() {
        let entry = HistoryEntry {
            display: "/commit -m \"fix\"".to_string(),
            timestamp: 1738567268363,
            project: "/home/user/project".to_string(),
            session_id: "sess-123".to_string(),
        };

        let event = create_skill_invocation_event(&entry).unwrap();

        assert_eq!(event.skill_name, "commit");
        assert_eq!(event.session_id, "sess-123");
        assert_eq!(event.project, "/home/user/project");
        assert_eq!(event.timestamp.timestamp_millis(), 1738567268363);
    }

    #[test]
    fn create_event_with_namespaced_skill() {
        let entry = HistoryEntry {
            display: "/sdd:plan --verbose".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc-def".to_string(),
        };

        let event = create_skill_invocation_event(&entry).unwrap();

        assert_eq!(event.skill_name, "sdd:plan");
    }

    #[test]
    fn create_event_preserves_all_fields() {
        let entry = HistoryEntry {
            display: "/review-pr".to_string(),
            timestamp: 1738567268363,
            project: "/home/ubuntu/Projects/VibeTea".to_string(),
            session_id: "6e45a55c-3124-4cc8-ad85-040a5c316009".to_string(),
        };

        let event = create_skill_invocation_event(&entry).unwrap();

        assert_eq!(event.skill_name, "review-pr");
        assert_eq!(event.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
        assert_eq!(event.project, "/home/ubuntu/Projects/VibeTea");
    }

    #[test]
    fn create_event_returns_none_for_invalid_display() {
        let entry = HistoryEntry {
            display: "not a skill".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let event = create_skill_invocation_event(&entry);

        assert!(event.is_none());
    }

    #[test]
    fn create_event_returns_none_for_just_slash() {
        let entry = HistoryEntry {
            display: "/".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let event = create_skill_invocation_event(&entry);

        assert!(event.is_none());
    }

    // =========================================================================
    // HistoryEntry Trait Tests
    // =========================================================================

    #[test]
    fn history_entry_debug() {
        let entry = HistoryEntry {
            display: "/commit".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let debug_str = format!("{:?}", entry);

        assert!(debug_str.contains("HistoryEntry"));
        assert!(debug_str.contains("display"));
        assert!(debug_str.contains("timestamp"));
        assert!(debug_str.contains("project"));
        assert!(debug_str.contains("session_id"));
    }

    #[test]
    fn history_entry_clone() {
        let original = HistoryEntry {
            display: "/commit".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(cloned.display, "/commit");
    }

    #[test]
    fn history_entry_equality() {
        let a = HistoryEntry {
            display: "/commit".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let b = HistoryEntry {
            display: "/commit".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        let c = HistoryEntry {
            display: "/review-pr".to_string(),
            timestamp: 1738567268363,
            project: "/proj".to_string(),
            session_id: "abc".to_string(),
        };

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // =========================================================================
    // Error Type Tests
    // =========================================================================

    #[test]
    fn error_display_messages() {
        let json_err =
            HistoryParseError::InvalidJson(serde_json::from_str::<()>("invalid").unwrap_err());
        assert!(json_err.to_string().contains("invalid JSON"));

        let display_err = HistoryParseError::MissingDisplay;
        assert!(display_err.to_string().contains("display"));

        let timestamp_err = HistoryParseError::MissingTimestamp;
        assert!(timestamp_err.to_string().contains("timestamp"));

        let project_err = HistoryParseError::MissingProject;
        assert!(project_err.to_string().contains("project"));

        let session_err = HistoryParseError::MissingSessionId;
        assert!(session_err.to_string().contains("sessionId"));
    }

    #[test]
    fn error_is_debug() {
        let err = HistoryParseError::MissingDisplay;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("MissingDisplay"));
    }

    // =========================================================================
    // Integration Tests: Full Parse to Event Flow
    // =========================================================================

    #[test]
    fn full_flow_parse_and_create_event() {
        let line = r#"{"display": "/commit -m \"fix: update docs\"", "timestamp": 1738567268363, "project": "/home/ubuntu/Projects/VibeTea", "sessionId": "6e45a55c-3124-4cc8-ad85-040a5c316009"}"#;

        let entry = parse_history_entry(line).unwrap();
        let event = create_skill_invocation_event(&entry).unwrap();

        assert_eq!(event.skill_name, "commit");
        assert_eq!(event.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
        assert_eq!(event.project, "/home/ubuntu/Projects/VibeTea");
        assert_eq!(event.timestamp.timestamp_millis(), 1738567268363);
    }

    #[test]
    fn full_flow_multiple_entries_to_events() {
        let content = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "a"}
{"display": "/review-pr 123", "timestamp": 1738567268400, "project": "/proj", "sessionId": "b"}
{"display": "not a skill", "timestamp": 1738567268500, "project": "/proj", "sessionId": "c"}
{"display": "/sdd:plan", "timestamp": 1738567268600, "project": "/proj", "sessionId": "d"}"#;

        let entries = parse_history_entries(content);
        let events: Vec<_> = entries
            .iter()
            .filter_map(create_skill_invocation_event)
            .collect();

        // "not a skill" entry doesn't produce an event
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].skill_name, "commit");
        assert_eq!(events[1].skill_name, "review-pr");
        assert_eq!(events[2].skill_name, "sdd:plan");
    }

    // =========================================================================
    // Realistic JSONL Parsing Tests
    // =========================================================================

    #[test]
    fn parse_realistic_history_jsonl_line() {
        // Exactly as shown in the spec
        let line = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/home/ubuntu/Projects/VibeTea", "sessionId": "6e45a55c-3124-4cc8-ad85-040a5c316009"}"#;

        let entry = parse_history_entry(line).unwrap();

        assert_eq!(entry.display, "/commit");
        assert_eq!(entry.timestamp, 1738567268363);
        assert_eq!(entry.project, "/home/ubuntu/Projects/VibeTea");
        assert_eq!(entry.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
    }

    #[test]
    fn parse_history_entry_with_special_characters_in_project() {
        let line = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/home/user/My Projects/app-v2", "sessionId": "abc"}"#;

        let entry = parse_history_entry(line).unwrap();

        assert_eq!(entry.project, "/home/user/My Projects/app-v2");
    }

    #[test]
    fn parse_history_entry_windows_path() {
        let line = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "C:\\Users\\dev\\project", "sessionId": "abc"}"#;

        let entry = parse_history_entry(line).unwrap();

        assert_eq!(entry.project, "C:\\Users\\dev\\project");
    }

    // =========================================================================
    // SkillTrackerError Tests
    // =========================================================================

    #[test]
    fn skill_tracker_error_display() {
        let err = SkillTrackerError::ChannelClosed;
        assert_eq!(err.to_string(), "failed to send event: channel closed");

        let err = SkillTrackerError::ClaudeDirectoryNotFound(PathBuf::from("/test"));
        assert_eq!(err.to_string(), "claude directory not found: /test");
    }

    #[test]
    fn skill_tracker_error_is_debug() {
        let err = SkillTrackerError::ChannelClosed;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ChannelClosed"));

        let err = SkillTrackerError::ClaudeDirectoryNotFound(PathBuf::from("/test"));
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ClaudeDirectoryNotFound"));
    }

    // =========================================================================
    // SkillTrackerConfig Tests
    // =========================================================================

    #[test]
    fn skill_tracker_config_default() {
        let config = SkillTrackerConfig::default();
        assert!(!config.emit_existing_on_startup);
    }

    #[test]
    fn skill_tracker_config_clone() {
        let config = SkillTrackerConfig {
            emit_existing_on_startup: true,
        };
        let cloned = config.clone();
        assert!(cloned.emit_existing_on_startup);
    }

    // =========================================================================
    // SkillTracker File Operations Tests
    // =========================================================================

    use std::io::Write;
    use tempfile::TempDir;
    use tokio::time::{sleep, timeout, Duration};

    /// Sample history.jsonl content for testing.
    const SAMPLE_HISTORY: &str = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "sess-1"}
{"display": "/review-pr 123", "timestamp": 1738567268400, "project": "/proj", "sessionId": "sess-2"}
{"display": "/sdd:plan", "timestamp": 1738567268500, "project": "/proj", "sessionId": "sess-3"}"#;

    /// Creates a temporary directory with a history.jsonl file.
    fn create_test_history_file(content: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let history_path = temp_dir.path().join("history.jsonl");

        let mut file = std::fs::File::create(&history_path).expect("Failed to create history file");
        file.write_all(content.as_bytes())
            .expect("Failed to write history content");
        file.flush().expect("Failed to flush");

        (temp_dir, history_path)
    }

    #[tokio::test]
    async fn test_tracker_creation_missing_directory() {
        let (tx, _rx) = mpsc::channel(100);
        let result = SkillTracker::with_path(PathBuf::from("/nonexistent/path/history.jsonl"), tx);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillTrackerError::ClaudeDirectoryNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_tracker_creation_with_valid_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let history_path = temp_dir.path().join("history.jsonl");

        let (tx, _rx) = mpsc::channel(100);
        let result = SkillTracker::with_path(history_path.clone(), tx);

        assert!(result.is_ok(), "Should create tracker for valid directory");
        assert_eq!(result.unwrap().history_path(), history_path);
    }

    #[tokio::test]
    async fn test_tracker_creation_file_does_not_exist() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let history_path = temp_dir.path().join("history.jsonl");

        let (tx, _rx) = mpsc::channel(100);
        let tracker = SkillTracker::with_path(history_path, tx).expect("Should create tracker");

        // Initial offset should be 0 since file doesn't exist
        assert_eq!(tracker.current_offset(), 0);
    }

    #[tokio::test]
    async fn test_tracker_creation_file_exists() {
        let (_temp_dir, history_path) = create_test_history_file(SAMPLE_HISTORY);

        let (tx, _rx) = mpsc::channel(100);
        let tracker = SkillTracker::with_path(history_path, tx).expect("Should create tracker");

        // Initial offset should be at end of file
        assert_eq!(tracker.current_offset(), SAMPLE_HISTORY.len() as u64);
    }

    #[tokio::test]
    async fn test_tracker_with_emit_existing_on_startup() {
        let (_temp_dir, history_path) = create_test_history_file(SAMPLE_HISTORY);

        let (tx, mut rx) = mpsc::channel(100);
        let config = SkillTrackerConfig {
            emit_existing_on_startup: true,
        };
        let _tracker = SkillTracker::with_path_and_config(history_path, tx, config)
            .expect("Should create tracker");

        // Should receive initial events for all 3 entries
        let mut events = Vec::new();
        while let Ok(Some(event)) = timeout(Duration::from_millis(200), rx.recv()).await {
            events.push(event);
        }

        assert_eq!(
            events.len(),
            3,
            "Should emit events for all existing entries"
        );
        assert_eq!(events[0].skill_name, "commit");
        assert_eq!(events[1].skill_name, "review-pr");
        assert_eq!(events[2].skill_name, "sdd:plan");
    }

    #[tokio::test]
    async fn test_tracker_without_emit_existing_on_startup() {
        let (_temp_dir, history_path) = create_test_history_file(SAMPLE_HISTORY);

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = SkillTracker::with_path(history_path, tx).expect("Should create tracker");

        // Should NOT receive any initial events
        let result = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should not receive events without emit_existing_on_startup"
        );
    }

    #[tokio::test]
    async fn test_tracker_refresh() {
        let (temp_dir, history_path) = create_test_history_file(SAMPLE_HISTORY);

        let (tx, mut rx) = mpsc::channel(100);
        let tracker =
            SkillTracker::with_path(history_path.clone(), tx).expect("Should create tracker");

        // Reset offset to 0 for testing refresh
        tracker.offset.store(0, Ordering::SeqCst);

        // Call refresh
        tracker.refresh().await.expect("Should refresh");

        // Should receive events for all entries
        let mut events = Vec::new();
        while let Ok(Some(event)) = timeout(Duration::from_millis(200), rx.recv()).await {
            events.push(event);
        }

        assert_eq!(
            events.len(),
            3,
            "Refresh should emit events for all entries"
        );

        // Offset should be updated
        assert_eq!(tracker.current_offset(), SAMPLE_HISTORY.len() as u64);

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_tracker_detects_new_entries() {
        let (temp_dir, history_path) = create_test_history_file("");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker =
            SkillTracker::with_path(history_path.clone(), tx).expect("Should create tracker");

        // Give watcher time to start
        sleep(Duration::from_millis(50)).await;

        // Append a new entry
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&history_path)
            .expect("Should open file");
        writeln!(
            file,
            r#"{{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "sess-new"}}"#
        )
        .expect("Should write");
        file.flush().expect("Should flush");

        // Should receive the new event
        let result = timeout(Duration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event for new entry");

        let event = result.unwrap().unwrap();
        assert_eq!(event.skill_name, "commit");
        assert_eq!(event.session_id, "sess-new");

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_tracker_detects_multiple_new_entries() {
        let (temp_dir, history_path) = create_test_history_file("");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker =
            SkillTracker::with_path(history_path.clone(), tx).expect("Should create tracker");

        // Give watcher time to start
        sleep(Duration::from_millis(50)).await;

        // Append multiple new entries
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&history_path)
            .expect("Should open file");
        writeln!(
            file,
            r#"{{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "sess-1"}}"#
        )
        .expect("Should write");
        writeln!(
            file,
            r#"{{"display": "/review-pr", "timestamp": 1738567268400, "project": "/proj", "sessionId": "sess-2"}}"#
        )
        .expect("Should write");
        file.flush().expect("Should flush");

        // Should receive both events
        let mut events = Vec::new();
        while let Ok(Some(event)) = timeout(Duration::from_millis(500), rx.recv()).await {
            events.push(event);
            if events.len() >= 2 {
                break;
            }
        }

        assert_eq!(
            events.len(),
            2,
            "Should receive events for both new entries"
        );
        assert_eq!(events[0].skill_name, "commit");
        assert_eq!(events[1].skill_name, "review-pr");

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_tracker_handles_invalid_entries_gracefully() {
        let (temp_dir, history_path) = create_test_history_file("");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker =
            SkillTracker::with_path(history_path.clone(), tx).expect("Should create tracker");

        // Give watcher time to start
        sleep(Duration::from_millis(50)).await;

        // Append entries including invalid ones
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&history_path)
            .expect("Should open file");
        writeln!(file, "invalid json here").expect("Should write");
        writeln!(
            file,
            r#"{{"display": "/commit", "timestamp": 1738567268363, "project": "/proj", "sessionId": "sess-valid"}}"#
        )
        .expect("Should write");
        file.flush().expect("Should flush");

        // Should receive only the valid event
        let result = timeout(Duration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event for valid entry");

        let event = result.unwrap().unwrap();
        assert_eq!(event.skill_name, "commit");
        assert_eq!(event.session_id, "sess-valid");

        drop(temp_dir);
    }

    #[tokio::test]
    async fn test_get_file_size() {
        let (_temp_dir, history_path) = create_test_history_file(SAMPLE_HISTORY);

        let size = get_file_size(&history_path).expect("Should get file size");
        assert_eq!(size, SAMPLE_HISTORY.len() as u64);
    }

    #[tokio::test]
    async fn test_get_file_size_missing_file() {
        let result = get_file_size(Path::new("/nonexistent/file.jsonl"));
        assert!(result.is_err());
    }

    // =========================================================================
    // Emit Functions Tests
    // =========================================================================

    #[tokio::test]
    async fn test_emit_all_entries() {
        let (_temp_dir, history_path) = create_test_history_file(SAMPLE_HISTORY);

        let (tx, mut rx) = mpsc::channel(100);
        emit_all_entries(&history_path, &tx)
            .await
            .expect("Should emit all entries");

        // Should receive 3 events
        let mut events = Vec::new();
        while let Ok(Some(event)) = timeout(Duration::from_millis(100), rx.recv()).await {
            events.push(event);
        }

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].skill_name, "commit");
        assert_eq!(events[1].skill_name, "review-pr");
        assert_eq!(events[2].skill_name, "sdd:plan");
    }

    #[tokio::test]
    async fn test_emit_new_entries_from_offset() {
        let content = r#"{"display": "/first", "timestamp": 1738567268363, "project": "/proj", "sessionId": "a"}
{"display": "/second", "timestamp": 1738567268400, "project": "/proj", "sessionId": "b"}"#;
        let (_temp_dir, history_path) = create_test_history_file(content);

        // Calculate offset to just after first line
        let first_line_len = content.lines().next().unwrap().len() + 1; // +1 for newline

        let (tx, mut rx) = mpsc::channel(100);
        let new_offset = emit_new_entries(&history_path, &tx, first_line_len as u64)
            .await
            .expect("Should emit new entries");

        // Should receive only the second event
        let result = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(result.is_ok());
        let event = result.unwrap().unwrap();
        assert_eq!(event.skill_name, "second");

        // New offset should be at end of file
        assert_eq!(new_offset, content.len() as u64);
    }

    #[tokio::test]
    async fn test_emit_new_entries_handles_truncated_file() {
        let (_temp_dir, history_path) = create_test_history_file(SAMPLE_HISTORY);

        let (tx, mut rx) = mpsc::channel(100);

        // Use an offset larger than the file size (simulating truncation)
        let large_offset = (SAMPLE_HISTORY.len() + 1000) as u64;
        let new_offset = emit_new_entries(&history_path, &tx, large_offset)
            .await
            .expect("Should handle truncated file");

        // Should reset to beginning and read all entries
        let mut events = Vec::new();
        while let Ok(Some(event)) = timeout(Duration::from_millis(100), rx.recv()).await {
            events.push(event);
        }

        assert_eq!(
            events.len(),
            3,
            "Should read all entries after truncation reset"
        );
        assert_eq!(new_offset, SAMPLE_HISTORY.len() as u64);
    }

    #[tokio::test]
    async fn test_emit_new_entries_empty_file() {
        let (_temp_dir, history_path) = create_test_history_file("");

        let (tx, mut rx) = mpsc::channel(100);
        let new_offset = emit_new_entries(&history_path, &tx, 0)
            .await
            .expect("Should handle empty file");

        // Should receive no events
        let result = timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should timeout with no events for empty file"
        );

        assert_eq!(new_offset, 0);
    }

    #[tokio::test]
    async fn test_emit_new_entries_skips_non_skill_entries() {
        let content = r#"{"display": "not a skill", "timestamp": 1738567268363, "project": "/proj", "sessionId": "a"}
{"display": "/commit", "timestamp": 1738567268400, "project": "/proj", "sessionId": "b"}"#;
        let (_temp_dir, history_path) = create_test_history_file(content);

        let (tx, mut rx) = mpsc::channel(100);
        emit_new_entries(&history_path, &tx, 0)
            .await
            .expect("Should emit entries");

        // Should receive only the valid skill event
        let result = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(result.is_ok());
        let event = result.unwrap().unwrap();
        assert_eq!(event.skill_name, "commit");

        // No more events
        let result = timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should not receive event for non-skill entry"
        );
    }
}
