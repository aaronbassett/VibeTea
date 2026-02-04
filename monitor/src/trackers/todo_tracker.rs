//! Todo tracker for monitoring task list progress per session.
//!
//! This module watches `~/.claude/todos/` for changes and emits
//! [`TodoProgressEvent`]s for each session's todo list updates.
//!
//! # Todo File Format
//!
//! Todo files are stored at `~/.claude/todos/<session-uuid>-agent-<session-uuid>.json`
//! and contain a JSON array of todo entries:
//!
//! ```json
//! [
//!   {
//!     "content": "Task description text",
//!     "status": "completed",
//!     "activeForm": "Completing task..."
//!   },
//!   {
//!     "content": "Another task",
//!     "status": "in_progress",
//!     "activeForm": "Working on task..."
//!   },
//!   {
//!     "content": "Pending task",
//!     "status": "pending",
//!     "activeForm": null
//!   }
//! ]
//! ```
//!
//! # Status Values
//!
//! - `completed`: Task finished successfully
//! - `in_progress`: Task currently being worked on
//! - `pending`: Task waiting to start
//!
//! # Abandonment Detection
//!
//! A todo list is considered "abandoned" when:
//! - The session has ended (summary event received)
//! - There are still `in_progress` or `pending` tasks remaining
//!
//! This is detected by correlating todo file state with session summary events.
//!
//! # Privacy
//!
//! This module follows the privacy-first principle: only status counts
//! and metadata are captured. Task content is never transmitted.
//!
//! # Example
//!
//! ```
//! use vibetea_monitor::trackers::todo_tracker::{
//!     parse_todo_file,
//!     count_todo_statuses,
//!     create_todo_progress_event,
//! };
//!
//! let json = r#"[
//!   {"content": "Task 1", "status": "completed", "activeForm": null},
//!   {"content": "Task 2", "status": "in_progress", "activeForm": "Working..."},
//!   {"content": "Task 3", "status": "pending", "activeForm": null}
//! ]"#;
//!
//! let entries = parse_todo_file(json).unwrap();
//! let counts = count_todo_statuses(&entries);
//!
//! assert_eq!(counts.completed, 1);
//! assert_eq!(counts.in_progress, 1);
//! assert_eq!(counts.pending, 1);
//!
//! let event = create_todo_progress_event("session-123", &counts, false);
//! assert_eq!(event.completed, 1);
//! assert!(!event.abandoned);
//! ```
//!
//! # File Watching Example
//!
//! ```no_run
//! use tokio::sync::mpsc;
//! use vibetea_monitor::trackers::todo_tracker::TodoTracker;
//! use vibetea_monitor::types::TodoProgressEvent;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tx, mut rx) = mpsc::channel(100);
//!     let tracker = TodoTracker::new(tx)?;
//!
//!     while let Some(event) = rx.recv().await {
//!         println!(
//!             "Session {}: {}/{}/{} tasks (abandoned: {})",
//!             event.session_id,
//!             event.completed,
//!             event.in_progress,
//!             event.pending,
//!             event.abandoned
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, trace, warn};

use crate::types::TodoProgressEvent;
use crate::utils::debounce::Debouncer;
use crate::utils::session_filename::parse_todo_filename;

/// Errors that can occur when parsing todo files.
#[derive(Debug, Error)]
pub enum TodoParseError {
    /// Failed to parse the JSON structure.
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// The todo file is not a valid JSON array.
    #[error("todo file must be a JSON array")]
    NotAnArray,

    /// A todo entry is missing the required `content` field.
    #[error("todo entry missing required field: content")]
    MissingContent,

    /// A todo entry is missing the required `status` field.
    #[error("todo entry missing required field: status")]
    MissingStatus,

    /// The `status` field contains an invalid value.
    #[error("invalid status value: {0}")]
    InvalidStatus(String),

    /// The filename does not match the expected todo file pattern.
    #[error("invalid todo filename: expected <session-uuid>-agent-<session-uuid>.json")]
    InvalidFilename,
}

/// Result type for todo parsing operations.
pub type Result<T> = std::result::Result<T, TodoParseError>;

// ============================================================================
// TodoTracker Types
// ============================================================================

/// Errors that can occur during todo tracking operations.
#[derive(Error, Debug)]
pub enum TodoTrackerError {
    /// Failed to initialize the file system watcher.
    #[error("failed to create watcher: {0}")]
    WatcherInit(#[from] notify::Error),

    /// Failed to read a todo file.
    #[error("failed to read todo file: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse a todo file.
    #[error("failed to parse todo file: {0}")]
    Parse(#[from] TodoParseError),

    /// The todos directory does not exist.
    #[error("claude todos directory not found: {0}")]
    ClaudeDirectoryNotFound(PathBuf),

    /// Failed to send event through the channel.
    #[error("failed to send event: channel closed")]
    ChannelClosed,
}

/// Result type for todo tracker operations.
pub type TrackerResult<T> = std::result::Result<T, TodoTrackerError>;

/// Configuration for the todo tracker.
#[derive(Debug, Clone)]
pub struct TodoTrackerConfig {
    /// Debounce duration in milliseconds. Default: 100ms.
    ///
    /// Per research.md, 100ms is the recommended debounce interval for todo files
    /// to coalesce rapid writes during status updates.
    pub debounce_ms: u64,
}

impl Default for TodoTrackerConfig {
    fn default() -> Self {
        Self { debounce_ms: 100 }
    }
}

/// Status of a todo item.
///
/// Represents the three possible states of a todo item as stored by Claude Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    /// Task finished successfully.
    Completed,
    /// Task currently being worked on.
    InProgress,
    /// Task waiting to start.
    Pending,
}

impl TodoStatus {
    /// Attempts to parse a status from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - The status string to parse
    ///
    /// # Returns
    ///
    /// * `Some(TodoStatus)` if the string is a valid status
    /// * `None` if the string is not recognized
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::trackers::todo_tracker::TodoStatus;
    ///
    /// assert_eq!(TodoStatus::parse("completed"), Some(TodoStatus::Completed));
    /// assert_eq!(TodoStatus::parse("in_progress"), Some(TodoStatus::InProgress));
    /// assert_eq!(TodoStatus::parse("pending"), Some(TodoStatus::Pending));
    /// assert_eq!(TodoStatus::parse("invalid"), None);
    /// ```
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "completed" => Some(TodoStatus::Completed),
            "in_progress" => Some(TodoStatus::InProgress),
            "pending" => Some(TodoStatus::Pending),
            _ => None,
        }
    }
}

/// A single todo entry from the todo file.
///
/// Represents one task in a session's todo list. The JSON uses camelCase
/// field names which are mapped to snake_case.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoEntry {
    /// The task description text.
    pub content: String,

    /// The current status of the task.
    pub status: TodoStatus,

    /// The active form of the task (shown during execution).
    /// May be `null` for pending tasks.
    #[serde(default)]
    pub active_form: Option<String>,
}

/// Counts of todo items by status.
///
/// Aggregates the number of tasks in each state for a session's todo list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TodoStatusCounts {
    /// Number of completed tasks.
    pub completed: u32,
    /// Number of in-progress tasks.
    pub in_progress: u32,
    /// Number of pending tasks.
    pub pending: u32,
}

impl TodoStatusCounts {
    /// Returns the total number of todos.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::trackers::todo_tracker::TodoStatusCounts;
    ///
    /// let counts = TodoStatusCounts {
    ///     completed: 2,
    ///     in_progress: 1,
    ///     pending: 3,
    /// };
    /// assert_eq!(counts.total(), 6);
    /// ```
    #[must_use]
    pub fn total(&self) -> u32 {
        self.completed + self.in_progress + self.pending
    }

    /// Returns true if there are any incomplete tasks (in_progress or pending).
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::trackers::todo_tracker::TodoStatusCounts;
    ///
    /// let counts = TodoStatusCounts { completed: 5, in_progress: 0, pending: 0 };
    /// assert!(!counts.has_incomplete());
    ///
    /// let counts = TodoStatusCounts { completed: 3, in_progress: 1, pending: 0 };
    /// assert!(counts.has_incomplete());
    /// ```
    #[must_use]
    pub fn has_incomplete(&self) -> bool {
        self.in_progress > 0 || self.pending > 0
    }
}

/// Parses a single todo entry from a JSON value.
///
/// # Arguments
///
/// * `value` - A JSON value representing a todo entry
///
/// # Returns
///
/// * `Ok(TodoEntry)` if parsing succeeds
/// * `Err(TodoParseError)` if required fields are missing or invalid
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::todo_tracker::parse_todo_entry;
///
/// let json: serde_json::Value = serde_json::from_str(
///     r#"{"content": "Fix bug", "status": "completed", "activeForm": null}"#
/// ).unwrap();
///
/// let entry = parse_todo_entry(&json).unwrap();
/// assert_eq!(entry.content, "Fix bug");
/// ```
pub fn parse_todo_entry(value: &serde_json::Value) -> Result<TodoEntry> {
    // Validate required fields
    if value.get("content").is_none() {
        return Err(TodoParseError::MissingContent);
    }
    if value.get("status").is_none() {
        return Err(TodoParseError::MissingStatus);
    }

    // Validate status value before deserializing
    if let Some(status_str) = value.get("status").and_then(|v| v.as_str()) {
        if TodoStatus::parse(status_str).is_none() {
            return Err(TodoParseError::InvalidStatus(status_str.to_string()));
        }
    }

    // Deserialize the entry
    Ok(serde_json::from_value(value.clone())?)
}

/// Parses a todo file's JSON content into a vector of entries.
///
/// The todo file must be a JSON array of todo entry objects.
///
/// # Arguments
///
/// * `content` - The JSON content of the todo file
///
/// # Returns
///
/// * `Ok(Vec<TodoEntry>)` if parsing succeeds
/// * `Err(TodoParseError)` if the JSON is invalid or entries are malformed
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::todo_tracker::parse_todo_file;
///
/// let json = r#"[
///   {"content": "Task 1", "status": "completed", "activeForm": null},
///   {"content": "Task 2", "status": "pending", "activeForm": null}
/// ]"#;
///
/// let entries = parse_todo_file(json).unwrap();
/// assert_eq!(entries.len(), 2);
/// ```
pub fn parse_todo_file(content: &str) -> Result<Vec<TodoEntry>> {
    let value: serde_json::Value = serde_json::from_str(content)?;

    let array = value.as_array().ok_or(TodoParseError::NotAnArray)?;

    array.iter().map(parse_todo_entry).collect()
}

/// Parses a todo file, returning successfully parsed entries and skipping invalid ones.
///
/// This function is lenient: it skips entries that fail to parse and continues
/// with the remaining entries. This is useful for handling potentially corrupted
/// or partially written files.
///
/// # Arguments
///
/// * `content` - The JSON content of the todo file
///
/// # Returns
///
/// A vector of successfully parsed entries. Invalid entries are silently skipped.
/// Returns an empty vector if the content is not a valid JSON array.
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::todo_tracker::parse_todo_file_lenient;
///
/// // Even with an invalid entry, valid ones are returned
/// let json = r#"[
///   {"content": "Valid", "status": "completed", "activeForm": null},
///   {"content": "Missing status"},
///   {"content": "Also valid", "status": "pending", "activeForm": null}
/// ]"#;
///
/// let entries = parse_todo_file_lenient(json);
/// assert_eq!(entries.len(), 2);
/// ```
#[must_use]
pub fn parse_todo_file_lenient(content: &str) -> Vec<TodoEntry> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return Vec::new();
    };

    let Some(array) = value.as_array() else {
        return Vec::new();
    };

    array
        .iter()
        .filter_map(|v| parse_todo_entry(v).ok())
        .collect()
}

/// Counts the number of todo entries in each status.
///
/// # Arguments
///
/// * `entries` - A slice of todo entries to count
///
/// # Returns
///
/// A [`TodoStatusCounts`] struct with the counts for each status.
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::todo_tracker::{TodoEntry, TodoStatus, count_todo_statuses};
///
/// let entries = vec![
///     TodoEntry { content: "A".to_string(), status: TodoStatus::Completed, active_form: None },
///     TodoEntry { content: "B".to_string(), status: TodoStatus::Completed, active_form: None },
///     TodoEntry { content: "C".to_string(), status: TodoStatus::InProgress, active_form: Some("Working...".to_string()) },
///     TodoEntry { content: "D".to_string(), status: TodoStatus::Pending, active_form: None },
/// ];
///
/// let counts = count_todo_statuses(&entries);
/// assert_eq!(counts.completed, 2);
/// assert_eq!(counts.in_progress, 1);
/// assert_eq!(counts.pending, 1);
/// ```
#[must_use]
pub fn count_todo_statuses(entries: &[TodoEntry]) -> TodoStatusCounts {
    let mut counts = TodoStatusCounts::default();

    for entry in entries {
        match entry.status {
            TodoStatus::Completed => counts.completed += 1,
            TodoStatus::InProgress => counts.in_progress += 1,
            TodoStatus::Pending => counts.pending += 1,
        }
    }

    counts
}

/// Extracts the session ID from a todo filename.
///
/// This is a convenience wrapper around [`parse_todo_filename`] that returns
/// a [`Result`] with a descriptive error.
///
/// # Arguments
///
/// * `path` - The path to the todo file
///
/// # Returns
///
/// * `Ok(String)` containing the session ID if parsing succeeds
/// * `Err(TodoParseError::InvalidFilename)` if the filename doesn't match the pattern
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use vibetea_monitor::trackers::todo_tracker::extract_session_id_from_filename;
///
/// let path = Path::new("/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json");
/// let session_id = extract_session_id_from_filename(path).unwrap();
/// assert_eq!(session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
/// ```
pub fn extract_session_id_from_filename(path: &std::path::Path) -> Result<String> {
    parse_todo_filename(path).ok_or(TodoParseError::InvalidFilename)
}

/// Creates a [`TodoProgressEvent`] from status counts.
///
/// # Arguments
///
/// * `session_id` - The session UUID
/// * `counts` - The todo status counts
/// * `abandoned` - Whether the todo list was abandoned (session ended with incomplete tasks)
///
/// # Returns
///
/// A [`TodoProgressEvent`] ready for transmission.
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::todo_tracker::{TodoStatusCounts, create_todo_progress_event};
///
/// let counts = TodoStatusCounts { completed: 3, in_progress: 0, pending: 2 };
/// let event = create_todo_progress_event("sess-123", &counts, true);
///
/// assert_eq!(event.session_id, "sess-123");
/// assert_eq!(event.completed, 3);
/// assert_eq!(event.in_progress, 0);
/// assert_eq!(event.pending, 2);
/// assert!(event.abandoned);
/// ```
#[must_use]
pub fn create_todo_progress_event(
    session_id: &str,
    counts: &TodoStatusCounts,
    abandoned: bool,
) -> TodoProgressEvent {
    TodoProgressEvent {
        session_id: session_id.to_string(),
        completed: counts.completed,
        in_progress: counts.in_progress,
        pending: counts.pending,
        abandoned,
    }
}

/// Determines if a todo list should be marked as abandoned.
///
/// A todo list is abandoned when:
/// - The session has ended (indicated by `session_has_ended`)
/// - There are still incomplete tasks (in_progress or pending)
///
/// # Arguments
///
/// * `counts` - The todo status counts
/// * `session_has_ended` - Whether the session has received a summary event
///
/// # Returns
///
/// `true` if the todo list should be marked as abandoned, `false` otherwise.
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::todo_tracker::{TodoStatusCounts, is_abandoned};
///
/// // Session ended with pending tasks - abandoned
/// let counts = TodoStatusCounts { completed: 3, in_progress: 0, pending: 2 };
/// assert!(is_abandoned(&counts, true));
///
/// // Session ended with all tasks complete - not abandoned
/// let counts = TodoStatusCounts { completed: 5, in_progress: 0, pending: 0 };
/// assert!(!is_abandoned(&counts, true));
///
/// // Session still active - not abandoned regardless of status
/// let counts = TodoStatusCounts { completed: 0, in_progress: 1, pending: 5 };
/// assert!(!is_abandoned(&counts, false));
/// ```
#[must_use]
pub fn is_abandoned(counts: &TodoStatusCounts, session_has_ended: bool) -> bool {
    session_has_ended && counts.has_incomplete()
}

// ============================================================================
// TodoTracker - File Watching Implementation
// ============================================================================

/// Tracker for Claude Code's todo files.
///
/// Watches `~/.claude/todos/` for changes and emits [`TodoProgressEvent`]s
/// when todo files are created or modified. Uses debouncing to coalesce
/// rapid writes during status updates.
///
/// # Session Lifecycle
///
/// The tracker maintains a set of "ended" sessions to enable abandonment
/// detection. When a session ends (summary event received), call
/// [`mark_session_ended`](Self::mark_session_ended) to update the tracker's
/// state. Subsequent todo file updates for that session will have their
/// `abandoned` flag set if there are incomplete tasks.
///
/// # Thread Safety
///
/// The tracker spawns a background task for async processing of file events.
/// Communication is done via channels for thread safety. The ended sessions
/// set uses `RwLock` for concurrent access.
#[derive(Debug)]
pub struct TodoTracker {
    /// The underlying file system watcher.
    ///
    /// Kept alive to maintain the watch subscription.
    #[allow(dead_code)]
    watcher: RecommendedWatcher,

    /// Path to the todos directory being watched.
    todos_dir: PathBuf,

    /// Channel sender for emitting todo progress events.
    #[allow(dead_code)]
    event_sender: mpsc::Sender<TodoProgressEvent>,

    /// Sessions that have ended (summary event received).
    ///
    /// Used for abandonment detection: if a session has ended and
    /// its todo file still has incomplete tasks, the session is
    /// marked as abandoned.
    ended_sessions: Arc<RwLock<HashSet<String>>>,
}

impl TodoTracker {
    /// Creates a new todo tracker watching the default todos directory.
    ///
    /// The default location is `~/.claude/todos/`.
    ///
    /// # Arguments
    ///
    /// * `event_sender` - Channel for emitting [`TodoProgressEvent`]s
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The home directory cannot be determined
    /// - The `~/.claude/todos` directory does not exist
    /// - The file system watcher cannot be initialized
    pub fn new(event_sender: mpsc::Sender<TodoProgressEvent>) -> TrackerResult<Self> {
        Self::with_config(event_sender, TodoTrackerConfig::default())
    }

    /// Creates a new todo tracker with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `event_sender` - Channel for emitting [`TodoProgressEvent`]s
    /// * `config` - Configuration options for the tracker
    ///
    /// # Errors
    ///
    /// Returns an error if the tracker cannot be initialized.
    pub fn with_config(
        event_sender: mpsc::Sender<TodoProgressEvent>,
        config: TodoTrackerConfig,
    ) -> TrackerResult<Self> {
        let claude_dir = directories::BaseDirs::new()
            .map(|dirs| dirs.home_dir().join(".claude"))
            .ok_or_else(|| TodoTrackerError::ClaudeDirectoryNotFound(PathBuf::from("~/.claude")))?;

        Self::with_path_and_config(claude_dir.join("todos"), event_sender, config)
    }

    /// Creates a new todo tracker watching a specific directory.
    ///
    /// # Arguments
    ///
    /// * `todos_dir` - Path to the todos directory to watch
    /// * `event_sender` - Channel for emitting [`TodoProgressEvent`]s
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The specified directory does not exist
    /// - The file system watcher cannot be initialized
    pub fn with_path(
        todos_dir: PathBuf,
        event_sender: mpsc::Sender<TodoProgressEvent>,
    ) -> TrackerResult<Self> {
        Self::with_path_and_config(todos_dir, event_sender, TodoTrackerConfig::default())
    }

    /// Creates a new todo tracker with a specific path and configuration.
    ///
    /// # Arguments
    ///
    /// * `todos_dir` - Path to the todos directory to watch
    /// * `event_sender` - Channel for emitting [`TodoProgressEvent`]s
    /// * `config` - Configuration options for the tracker
    ///
    /// # Errors
    ///
    /// Returns an error if the tracker cannot be initialized.
    pub fn with_path_and_config(
        todos_dir: PathBuf,
        event_sender: mpsc::Sender<TodoProgressEvent>,
        config: TodoTrackerConfig,
    ) -> TrackerResult<Self> {
        // Verify the directory exists
        if !todos_dir.exists() {
            return Err(TodoTrackerError::ClaudeDirectoryNotFound(todos_dir));
        }

        info!(
            todos_dir = %todos_dir.display(),
            debounce_ms = config.debounce_ms,
            "Initializing todo tracker"
        );

        // Create the ended sessions set
        let ended_sessions = Arc::new(RwLock::new(HashSet::new()));

        // Create channel for debounced file change notifications
        let (debounce_tx, debounce_rx) = mpsc::channel::<(PathBuf, PathBuf)>(1000);

        // Create the debouncer
        let debouncer = Debouncer::new(Duration::from_millis(config.debounce_ms), debounce_tx);

        // Spawn the async processing task
        let sender_for_task = event_sender.clone();
        let ended_for_task = Arc::clone(&ended_sessions);
        tokio::spawn(async move {
            process_debounced_changes(debounce_rx, sender_for_task, ended_for_task).await;
        });

        // Create the file watcher
        let watcher = create_todos_watcher(todos_dir.clone(), debouncer)?;

        Ok(Self {
            watcher,
            todos_dir,
            event_sender,
            ended_sessions,
        })
    }

    /// Marks a session as ended.
    ///
    /// Call this method when a summary event is detected for a session.
    /// Subsequent todo file updates for this session will have their
    /// `abandoned` flag set if there are incomplete tasks.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session UUID to mark as ended
    pub async fn mark_session_ended(&self, session_id: &str) {
        let mut sessions = self.ended_sessions.write().await;
        sessions.insert(session_id.to_string());
        debug!(session_id = %session_id, "Marked session as ended");
    }

    /// Checks if a session has ended.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session UUID to check
    ///
    /// # Returns
    ///
    /// `true` if the session has been marked as ended, `false` otherwise.
    pub async fn is_session_ended(&self, session_id: &str) -> bool {
        let sessions = self.ended_sessions.read().await;
        sessions.contains(session_id)
    }

    /// Clears the ended status for a session.
    ///
    /// This can be called if a session is restarted or the ended status
    /// was set in error.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session UUID to clear
    pub async fn clear_session_ended(&self, session_id: &str) {
        let mut sessions = self.ended_sessions.write().await;
        sessions.remove(session_id);
        trace!(session_id = %session_id, "Cleared session ended status");
    }

    /// Returns the path to the todos directory being watched.
    #[must_use]
    pub fn todos_dir(&self) -> &Path {
        &self.todos_dir
    }

    /// Returns the number of sessions marked as ended.
    pub async fn ended_sessions_count(&self) -> usize {
        let sessions = self.ended_sessions.read().await;
        sessions.len()
    }
}

/// Creates the file system watcher for the todos directory.
fn create_todos_watcher(
    todos_dir: PathBuf,
    debouncer: Debouncer<PathBuf, PathBuf>,
) -> TrackerResult<RecommendedWatcher> {
    let watch_dir = todos_dir.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: std::result::Result<Event, notify::Error>| {
            handle_notify_event(res, &todos_dir, &debouncer);
        },
        Config::default(),
    )?;

    // Watch the todos directory (non-recursive)
    watcher.watch(&watch_dir, RecursiveMode::NonRecursive)?;

    debug!(
        watch_dir = %watch_dir.display(),
        "Started watching for todo file changes"
    );

    Ok(watcher)
}

/// Handles raw notify events and sends them to the debouncer.
fn handle_notify_event(
    res: std::result::Result<Event, notify::Error>,
    todos_dir: &Path,
    debouncer: &Debouncer<PathBuf, PathBuf>,
) {
    let event = match res {
        Ok(event) => event,
        Err(e) => {
            error!(error = %e, "File watcher error");
            return;
        }
    };

    trace!(kind = ?event.kind, paths = ?event.paths, "Received notify event");

    // Only process create and modify events
    let should_process = matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_));

    if !should_process {
        trace!(kind = ?event.kind, "Ignoring event kind");
        return;
    }

    // Process each path in the event
    for path in &event.paths {
        // Only process .json files
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        // Verify the path is in the todos directory
        if path.parent() != Some(todos_dir) {
            continue;
        }

        // Try to extract session ID to validate filename format
        if parse_todo_filename(path).is_none() {
            trace!(path = %path.display(), "Ignoring non-todo file");
            continue;
        }

        debug!(
            path = %path.display(),
            kind = ?event.kind,
            "Todo file changed, sending to debouncer"
        );

        // Send to debouncer (key is the path, value is also the path)
        if !debouncer.try_send(path.clone(), path.clone()) {
            warn!(path = %path.display(), "Failed to send to debouncer: channel full or closed");
        }
    }
}

/// Processes debounced file change events, parsing todo files and emitting events.
async fn process_debounced_changes(
    mut rx: mpsc::Receiver<(PathBuf, PathBuf)>,
    sender: mpsc::Sender<TodoProgressEvent>,
    ended_sessions: Arc<RwLock<HashSet<String>>>,
) {
    debug!("Starting todo file change processor");

    while let Some((path, _)) = rx.recv().await {
        debug!(path = %path.display(), "Processing debounced todo file change");

        // Extract session ID from filename
        let session_id = match parse_todo_filename(&path) {
            Some(id) => id,
            None => {
                warn!(path = %path.display(), "Could not extract session ID from filename");
                continue;
            }
        };

        // Read and parse the todo file
        let content = match tokio::fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(e) => {
                // File might have been deleted between event and processing
                if e.kind() == std::io::ErrorKind::NotFound {
                    trace!(path = %path.display(), "Todo file was deleted");
                } else {
                    warn!(path = %path.display(), error = %e, "Failed to read todo file");
                }
                continue;
            }
        };

        // Parse using lenient parsing to handle partially written files
        let entries = parse_todo_file_lenient(&content);
        let counts = count_todo_statuses(&entries);

        // Check if session has ended (for abandonment detection)
        let session_ended = {
            let sessions = ended_sessions.read().await;
            sessions.contains(&session_id)
        };

        let abandoned = is_abandoned(&counts, session_ended);
        let event = create_todo_progress_event(&session_id, &counts, abandoned);

        trace!(
            session_id = %session_id,
            completed = counts.completed,
            in_progress = counts.in_progress,
            pending = counts.pending,
            abandoned = abandoned,
            "Emitting todo progress event"
        );

        if let Err(e) = sender.send(event).await {
            error!(error = %e, "Failed to send todo progress event: channel closed");
            break;
        }
    }

    debug!("Todo file change processor shutting down");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // =========================================================================
    // T118: Todo Filename Parsing Tests
    // =========================================================================

    #[test]
    fn extract_session_id_valid_filename() {
        let path = Path::new(
            "/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json",
        );
        let session_id = extract_session_id_from_filename(path).unwrap();
        assert_eq!(session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
    }

    #[test]
    fn extract_session_id_different_uuids() {
        // The two UUIDs can be different
        let path = Path::new(
            "/home/user/.claude/todos/a1b2c3d4-e5f6-7890-abcd-ef1234567890-agent-f1e2d3c4-b5a6-0987-fedc-ba9876543210.json",
        );
        let session_id = extract_session_id_from_filename(path).unwrap();
        assert_eq!(session_id, "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
    }

    #[test]
    fn extract_session_id_missing_agent_separator() {
        let path = Path::new("/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009.json");
        let result = extract_session_id_from_filename(path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TodoParseError::InvalidFilename
        ));
    }

    #[test]
    fn extract_session_id_wrong_extension() {
        let path = Path::new(
            "/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.txt",
        );
        let result = extract_session_id_from_filename(path);
        assert!(result.is_err());
    }

    #[test]
    fn extract_session_id_invalid_uuid_format() {
        let path = Path::new("/home/user/.claude/todos/not-a-valid-uuid-agent-something.json");
        let result = extract_session_id_from_filename(path);
        assert!(result.is_err());
    }

    #[test]
    fn extract_session_id_empty_path() {
        let path = Path::new("");
        let result = extract_session_id_from_filename(path);
        assert!(result.is_err());
    }

    #[test]
    fn extract_session_id_just_filename() {
        let path = Path::new(
            "6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json",
        );
        let session_id = extract_session_id_from_filename(path).unwrap();
        assert_eq!(session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
    }

    #[test]
    fn extract_session_id_uppercase_uuid() {
        let path = Path::new(
            "/home/user/.claude/todos/6E45A55C-3124-4CC8-AD85-040A5C316009-agent-something.json",
        );
        let session_id = extract_session_id_from_filename(path).unwrap();
        assert_eq!(session_id, "6E45A55C-3124-4CC8-AD85-040A5C316009");
    }

    // =========================================================================
    // T119: Todo Status Counting Tests
    // =========================================================================

    #[test]
    fn count_statuses_mixed_entries() {
        let entries = vec![
            TodoEntry {
                content: "Task A".to_string(),
                status: TodoStatus::Completed,
                active_form: None,
            },
            TodoEntry {
                content: "Task B".to_string(),
                status: TodoStatus::Completed,
                active_form: None,
            },
            TodoEntry {
                content: "Task C".to_string(),
                status: TodoStatus::InProgress,
                active_form: Some("Working on C...".to_string()),
            },
            TodoEntry {
                content: "Task D".to_string(),
                status: TodoStatus::Pending,
                active_form: None,
            },
            TodoEntry {
                content: "Task E".to_string(),
                status: TodoStatus::Pending,
                active_form: None,
            },
            TodoEntry {
                content: "Task F".to_string(),
                status: TodoStatus::Pending,
                active_form: None,
            },
        ];

        let counts = count_todo_statuses(&entries);

        assert_eq!(counts.completed, 2);
        assert_eq!(counts.in_progress, 1);
        assert_eq!(counts.pending, 3);
        assert_eq!(counts.total(), 6);
    }

    #[test]
    fn count_statuses_empty_file() {
        let entries: Vec<TodoEntry> = vec![];
        let counts = count_todo_statuses(&entries);

        assert_eq!(counts.completed, 0);
        assert_eq!(counts.in_progress, 0);
        assert_eq!(counts.pending, 0);
        assert_eq!(counts.total(), 0);
    }

    #[test]
    fn count_statuses_only_completed() {
        let entries = vec![
            TodoEntry {
                content: "A".to_string(),
                status: TodoStatus::Completed,
                active_form: None,
            },
            TodoEntry {
                content: "B".to_string(),
                status: TodoStatus::Completed,
                active_form: None,
            },
            TodoEntry {
                content: "C".to_string(),
                status: TodoStatus::Completed,
                active_form: None,
            },
        ];

        let counts = count_todo_statuses(&entries);

        assert_eq!(counts.completed, 3);
        assert_eq!(counts.in_progress, 0);
        assert_eq!(counts.pending, 0);
        assert!(!counts.has_incomplete());
    }

    #[test]
    fn count_statuses_only_in_progress() {
        let entries = vec![
            TodoEntry {
                content: "A".to_string(),
                status: TodoStatus::InProgress,
                active_form: Some("Working...".to_string()),
            },
            TodoEntry {
                content: "B".to_string(),
                status: TodoStatus::InProgress,
                active_form: Some("Processing...".to_string()),
            },
        ];

        let counts = count_todo_statuses(&entries);

        assert_eq!(counts.completed, 0);
        assert_eq!(counts.in_progress, 2);
        assert_eq!(counts.pending, 0);
        assert!(counts.has_incomplete());
    }

    #[test]
    fn count_statuses_only_pending() {
        let entries = vec![
            TodoEntry {
                content: "A".to_string(),
                status: TodoStatus::Pending,
                active_form: None,
            },
            TodoEntry {
                content: "B".to_string(),
                status: TodoStatus::Pending,
                active_form: None,
            },
            TodoEntry {
                content: "C".to_string(),
                status: TodoStatus::Pending,
                active_form: None,
            },
            TodoEntry {
                content: "D".to_string(),
                status: TodoStatus::Pending,
                active_form: None,
            },
        ];

        let counts = count_todo_statuses(&entries);

        assert_eq!(counts.completed, 0);
        assert_eq!(counts.in_progress, 0);
        assert_eq!(counts.pending, 4);
        assert!(counts.has_incomplete());
    }

    #[test]
    fn count_statuses_from_json() {
        let json = r#"[
            {"content": "Task 1", "status": "completed", "activeForm": null},
            {"content": "Task 2", "status": "in_progress", "activeForm": "Working on task 2..."},
            {"content": "Task 3", "status": "pending", "activeForm": null},
            {"content": "Task 4", "status": "completed", "activeForm": null}
        ]"#;

        let entries = parse_todo_file(json).unwrap();
        let counts = count_todo_statuses(&entries);

        assert_eq!(counts.completed, 2);
        assert_eq!(counts.in_progress, 1);
        assert_eq!(counts.pending, 1);
    }

    // =========================================================================
    // T120: Abandonment Detection Tests
    // =========================================================================

    #[test]
    fn abandonment_session_active_with_incomplete_tasks() {
        let counts = TodoStatusCounts {
            completed: 2,
            in_progress: 1,
            pending: 3,
        };

        // Session is still active - not abandoned
        assert!(!is_abandoned(&counts, false));
    }

    #[test]
    fn abandonment_session_ended_with_incomplete_tasks() {
        let counts = TodoStatusCounts {
            completed: 2,
            in_progress: 0,
            pending: 3,
        };

        // Session ended with pending tasks - abandoned
        assert!(is_abandoned(&counts, true));
    }

    #[test]
    fn abandonment_session_ended_with_in_progress_tasks() {
        let counts = TodoStatusCounts {
            completed: 5,
            in_progress: 1,
            pending: 0,
        };

        // Session ended with in_progress task - abandoned
        assert!(is_abandoned(&counts, true));
    }

    #[test]
    fn abandonment_session_ended_all_complete() {
        let counts = TodoStatusCounts {
            completed: 10,
            in_progress: 0,
            pending: 0,
        };

        // Session ended with all tasks complete - NOT abandoned
        assert!(!is_abandoned(&counts, true));
    }

    #[test]
    fn abandonment_session_ended_empty_todo_list() {
        let counts = TodoStatusCounts {
            completed: 0,
            in_progress: 0,
            pending: 0,
        };

        // Empty todo list - NOT abandoned (nothing to abandon)
        assert!(!is_abandoned(&counts, true));
    }

    #[test]
    fn abandonment_session_active_all_complete() {
        let counts = TodoStatusCounts {
            completed: 5,
            in_progress: 0,
            pending: 0,
        };

        // Session active, all complete - NOT abandoned
        assert!(!is_abandoned(&counts, false));
    }

    #[test]
    fn create_event_abandoned_true() {
        let counts = TodoStatusCounts {
            completed: 3,
            in_progress: 1,
            pending: 2,
        };

        let event = create_todo_progress_event("sess-123", &counts, true);

        assert_eq!(event.session_id, "sess-123");
        assert_eq!(event.completed, 3);
        assert_eq!(event.in_progress, 1);
        assert_eq!(event.pending, 2);
        assert!(event.abandoned);
    }

    #[test]
    fn create_event_abandoned_false() {
        let counts = TodoStatusCounts {
            completed: 5,
            in_progress: 2,
            pending: 1,
        };

        let event = create_todo_progress_event("session-456", &counts, false);

        assert_eq!(event.session_id, "session-456");
        assert_eq!(event.completed, 5);
        assert_eq!(event.in_progress, 2);
        assert_eq!(event.pending, 1);
        assert!(!event.abandoned);
    }

    #[test]
    fn create_event_empty_counts() {
        let counts = TodoStatusCounts::default();

        let event = create_todo_progress_event("empty-session", &counts, false);

        assert_eq!(event.session_id, "empty-session");
        assert_eq!(event.completed, 0);
        assert_eq!(event.in_progress, 0);
        assert_eq!(event.pending, 0);
        assert!(!event.abandoned);
    }

    // =========================================================================
    // Todo Entry Parsing Tests
    // =========================================================================

    #[test]
    fn parse_entry_valid_completed() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"content": "Fix bug", "status": "completed", "activeForm": null}"#,
        )
        .unwrap();

        let entry = parse_todo_entry(&json).unwrap();

        assert_eq!(entry.content, "Fix bug");
        assert_eq!(entry.status, TodoStatus::Completed);
        assert_eq!(entry.active_form, None);
    }

    #[test]
    fn parse_entry_valid_in_progress() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"content": "Add feature", "status": "in_progress", "activeForm": "Adding feature..."}"#,
        )
        .unwrap();

        let entry = parse_todo_entry(&json).unwrap();

        assert_eq!(entry.content, "Add feature");
        assert_eq!(entry.status, TodoStatus::InProgress);
        assert_eq!(entry.active_form, Some("Adding feature...".to_string()));
    }

    #[test]
    fn parse_entry_valid_pending() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"content": "Write tests", "status": "pending", "activeForm": null}"#,
        )
        .unwrap();

        let entry = parse_todo_entry(&json).unwrap();

        assert_eq!(entry.content, "Write tests");
        assert_eq!(entry.status, TodoStatus::Pending);
        assert_eq!(entry.active_form, None);
    }

    #[test]
    fn parse_entry_missing_content() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"status": "completed", "activeForm": null}"#).unwrap();

        let result = parse_todo_entry(&json);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TodoParseError::MissingContent
        ));
    }

    #[test]
    fn parse_entry_missing_status() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"content": "Task", "activeForm": null}"#).unwrap();

        let result = parse_todo_entry(&json);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TodoParseError::MissingStatus));
    }

    #[test]
    fn parse_entry_invalid_status() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"content": "Task", "status": "invalid_status", "activeForm": null}"#,
        )
        .unwrap();

        let result = parse_todo_entry(&json);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TodoParseError::InvalidStatus(_)
        ));
    }

    #[test]
    fn parse_entry_missing_active_form_field() {
        // activeForm is optional, so missing field should default to None
        let json: serde_json::Value =
            serde_json::from_str(r#"{"content": "Task", "status": "pending"}"#).unwrap();

        let entry = parse_todo_entry(&json).unwrap();

        assert_eq!(entry.content, "Task");
        assert_eq!(entry.status, TodoStatus::Pending);
        assert_eq!(entry.active_form, None);
    }

    #[test]
    fn parse_entry_extra_fields_ignored() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"content": "Task", "status": "completed", "activeForm": null, "extraField": "ignored", "anotherExtra": 42}"#,
        )
        .unwrap();

        let entry = parse_todo_entry(&json).unwrap();

        assert_eq!(entry.content, "Task");
        assert_eq!(entry.status, TodoStatus::Completed);
    }

    // =========================================================================
    // Todo File Parsing Tests
    // =========================================================================

    #[test]
    fn parse_file_valid_multiple_entries() {
        let json = r#"[
            {"content": "Task 1", "status": "completed", "activeForm": null},
            {"content": "Task 2", "status": "in_progress", "activeForm": "Working..."},
            {"content": "Task 3", "status": "pending", "activeForm": null}
        ]"#;

        let entries = parse_todo_file(json).unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].status, TodoStatus::Completed);
        assert_eq!(entries[1].status, TodoStatus::InProgress);
        assert_eq!(entries[2].status, TodoStatus::Pending);
    }

    #[test]
    fn parse_file_empty_array() {
        let json = "[]";
        let entries = parse_todo_file(json).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_file_single_entry() {
        let json = r#"[{"content": "Solo task", "status": "pending", "activeForm": null}]"#;
        let entries = parse_todo_file(json).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "Solo task");
    }

    #[test]
    fn parse_file_not_an_array() {
        let json = r#"{"content": "Not an array", "status": "completed"}"#;
        let result = parse_todo_file(json);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TodoParseError::NotAnArray));
    }

    #[test]
    fn parse_file_invalid_json() {
        let json = "not valid json at all";
        let result = parse_todo_file(json);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TodoParseError::InvalidJson(_)
        ));
    }

    #[test]
    fn parse_file_empty_string() {
        let json = "";
        let result = parse_todo_file(json);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TodoParseError::InvalidJson(_)
        ));
    }

    #[test]
    fn parse_file_with_invalid_entry_fails() {
        // Strict parsing should fail if any entry is invalid
        let json = r#"[
            {"content": "Valid", "status": "completed", "activeForm": null},
            {"content": "Missing status"},
            {"content": "Also valid", "status": "pending", "activeForm": null}
        ]"#;

        let result = parse_todo_file(json);
        assert!(result.is_err());
    }

    // =========================================================================
    // Lenient Parsing Tests
    // =========================================================================

    #[test]
    fn parse_file_lenient_with_invalid_entries() {
        let json = r#"[
            {"content": "Valid 1", "status": "completed", "activeForm": null},
            {"content": "Missing status"},
            {"status": "pending"},
            {"content": "Valid 2", "status": "pending", "activeForm": null}
        ]"#;

        let entries = parse_todo_file_lenient(json);

        // Only the 2 valid entries should be returned
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].content, "Valid 1");
        assert_eq!(entries[1].content, "Valid 2");
    }

    #[test]
    fn parse_file_lenient_invalid_json() {
        let json = "not valid json";
        let entries = parse_todo_file_lenient(json);
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_file_lenient_not_array() {
        let json = r#"{"content": "Not an array"}"#;
        let entries = parse_todo_file_lenient(json);
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_file_lenient_empty_array() {
        let json = "[]";
        let entries = parse_todo_file_lenient(json);
        assert!(entries.is_empty());
    }

    // =========================================================================
    // TodoStatus Tests
    // =========================================================================

    #[test]
    fn todo_status_parse_valid() {
        assert_eq!(TodoStatus::parse("completed"), Some(TodoStatus::Completed));
        assert_eq!(
            TodoStatus::parse("in_progress"),
            Some(TodoStatus::InProgress)
        );
        assert_eq!(TodoStatus::parse("pending"), Some(TodoStatus::Pending));
    }

    #[test]
    fn todo_status_parse_invalid() {
        assert_eq!(TodoStatus::parse("invalid"), None);
        assert_eq!(TodoStatus::parse("COMPLETED"), None);
        assert_eq!(TodoStatus::parse("Complete"), None);
        assert_eq!(TodoStatus::parse(""), None);
        assert_eq!(TodoStatus::parse("inprogress"), None);
    }

    #[test]
    fn todo_status_deserialize() {
        let completed: TodoStatus = serde_json::from_str(r#""completed""#).unwrap();
        assert_eq!(completed, TodoStatus::Completed);

        let in_progress: TodoStatus = serde_json::from_str(r#""in_progress""#).unwrap();
        assert_eq!(in_progress, TodoStatus::InProgress);

        let pending: TodoStatus = serde_json::from_str(r#""pending""#).unwrap();
        assert_eq!(pending, TodoStatus::Pending);
    }

    // =========================================================================
    // TodoStatusCounts Tests
    // =========================================================================

    #[test]
    fn todo_status_counts_default() {
        let counts = TodoStatusCounts::default();
        assert_eq!(counts.completed, 0);
        assert_eq!(counts.in_progress, 0);
        assert_eq!(counts.pending, 0);
        assert_eq!(counts.total(), 0);
        assert!(!counts.has_incomplete());
    }

    #[test]
    fn todo_status_counts_total() {
        let counts = TodoStatusCounts {
            completed: 5,
            in_progress: 3,
            pending: 7,
        };
        assert_eq!(counts.total(), 15);
    }

    #[test]
    fn todo_status_counts_has_incomplete() {
        // No incomplete
        let counts = TodoStatusCounts {
            completed: 5,
            in_progress: 0,
            pending: 0,
        };
        assert!(!counts.has_incomplete());

        // Has in_progress
        let counts = TodoStatusCounts {
            completed: 5,
            in_progress: 1,
            pending: 0,
        };
        assert!(counts.has_incomplete());

        // Has pending
        let counts = TodoStatusCounts {
            completed: 5,
            in_progress: 0,
            pending: 1,
        };
        assert!(counts.has_incomplete());

        // Has both
        let counts = TodoStatusCounts {
            completed: 0,
            in_progress: 2,
            pending: 3,
        };
        assert!(counts.has_incomplete());
    }

    // =========================================================================
    // TodoEntry Trait Tests
    // =========================================================================

    #[test]
    fn todo_entry_debug() {
        let entry = TodoEntry {
            content: "Test task".to_string(),
            status: TodoStatus::InProgress,
            active_form: Some("Testing...".to_string()),
        };

        let debug_str = format!("{:?}", entry);

        assert!(debug_str.contains("TodoEntry"));
        assert!(debug_str.contains("content"));
        assert!(debug_str.contains("status"));
        assert!(debug_str.contains("active_form"));
    }

    #[test]
    fn todo_entry_clone() {
        let original = TodoEntry {
            content: "Clone me".to_string(),
            status: TodoStatus::Pending,
            active_form: None,
        };

        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(cloned.content, "Clone me");
    }

    #[test]
    fn todo_entry_equality() {
        let a = TodoEntry {
            content: "Task".to_string(),
            status: TodoStatus::Completed,
            active_form: None,
        };

        let b = TodoEntry {
            content: "Task".to_string(),
            status: TodoStatus::Completed,
            active_form: None,
        };

        let c = TodoEntry {
            content: "Different".to_string(),
            status: TodoStatus::Completed,
            active_form: None,
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
            TodoParseError::InvalidJson(serde_json::from_str::<()>("invalid").unwrap_err());
        assert!(json_err.to_string().contains("invalid JSON"));

        let not_array_err = TodoParseError::NotAnArray;
        assert!(not_array_err.to_string().contains("array"));

        let missing_content_err = TodoParseError::MissingContent;
        assert!(missing_content_err.to_string().contains("content"));

        let missing_status_err = TodoParseError::MissingStatus;
        assert!(missing_status_err.to_string().contains("status"));

        let invalid_status_err = TodoParseError::InvalidStatus("bad_status".to_string());
        assert!(invalid_status_err.to_string().contains("bad_status"));

        let invalid_filename_err = TodoParseError::InvalidFilename;
        assert!(invalid_filename_err.to_string().contains("filename"));
    }

    #[test]
    fn error_is_debug() {
        let err = TodoParseError::MissingContent;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("MissingContent"));

        let err = TodoParseError::InvalidStatus("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidStatus"));
    }

    // =========================================================================
    // Integration Tests: Full Parse to Event Flow
    // =========================================================================

    #[test]
    fn full_flow_parse_count_create_event() {
        let json = r#"[
            {"content": "Task 1", "status": "completed", "activeForm": null},
            {"content": "Task 2", "status": "completed", "activeForm": null},
            {"content": "Task 3", "status": "in_progress", "activeForm": "Working on task 3..."},
            {"content": "Task 4", "status": "pending", "activeForm": null},
            {"content": "Task 5", "status": "pending", "activeForm": null}
        ]"#;

        let entries = parse_todo_file(json).unwrap();
        let counts = count_todo_statuses(&entries);
        let session_ended = false;
        let abandoned = is_abandoned(&counts, session_ended);
        let event = create_todo_progress_event("test-session-123", &counts, abandoned);

        assert_eq!(event.session_id, "test-session-123");
        assert_eq!(event.completed, 2);
        assert_eq!(event.in_progress, 1);
        assert_eq!(event.pending, 2);
        assert!(!event.abandoned);
    }

    #[test]
    fn full_flow_abandoned_session() {
        let json = r#"[
            {"content": "Completed task", "status": "completed", "activeForm": null},
            {"content": "Abandoned task", "status": "pending", "activeForm": null}
        ]"#;

        let entries = parse_todo_file(json).unwrap();
        let counts = count_todo_statuses(&entries);
        let session_ended = true; // Session has ended (summary event received)
        let abandoned = is_abandoned(&counts, session_ended);
        let event = create_todo_progress_event("abandoned-session", &counts, abandoned);

        assert_eq!(event.completed, 1);
        assert_eq!(event.pending, 1);
        assert!(event.abandoned);
    }

    #[test]
    fn full_flow_all_complete_session_ended() {
        let json = r#"[
            {"content": "Task 1", "status": "completed", "activeForm": null},
            {"content": "Task 2", "status": "completed", "activeForm": null},
            {"content": "Task 3", "status": "completed", "activeForm": null}
        ]"#;

        let entries = parse_todo_file(json).unwrap();
        let counts = count_todo_statuses(&entries);
        let session_ended = true;
        let abandoned = is_abandoned(&counts, session_ended);
        let event = create_todo_progress_event("complete-session", &counts, abandoned);

        assert_eq!(event.completed, 3);
        assert_eq!(event.in_progress, 0);
        assert_eq!(event.pending, 0);
        assert!(!event.abandoned); // All complete, not abandoned
    }

    // =========================================================================
    // Realistic Todo File Tests (matching research.md format)
    // =========================================================================

    #[test]
    fn parse_realistic_todo_file() {
        // Exactly as shown in research.md
        let json = r#"[
          {
            "content": "Task description text",
            "status": "completed",
            "activeForm": "Completing task..."
          },
          {
            "content": "Another task",
            "status": "in_progress",
            "activeForm": "Working on task..."
          },
          {
            "content": "Pending task",
            "status": "pending",
            "activeForm": null
          }
        ]"#;

        let entries = parse_todo_file(json).unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].content, "Task description text");
        assert_eq!(entries[0].status, TodoStatus::Completed);
        assert_eq!(
            entries[0].active_form,
            Some("Completing task...".to_string())
        );
        assert_eq!(entries[1].content, "Another task");
        assert_eq!(entries[1].status, TodoStatus::InProgress);
        assert_eq!(
            entries[1].active_form,
            Some("Working on task...".to_string())
        );
        assert_eq!(entries[2].content, "Pending task");
        assert_eq!(entries[2].status, TodoStatus::Pending);
        assert_eq!(entries[2].active_form, None);
    }

    #[test]
    fn parse_unicode_in_content() {
        let json = r#"[
            {"content": "Fix bug with emoji support", "status": "completed", "activeForm": "Fixing..."}
        ]"#;

        let entries = parse_todo_file(json).unwrap();
        assert!(entries[0].content.contains("emoji"));
    }

    #[test]
    fn parse_long_content() {
        let long_content = "A".repeat(10000);
        let json = format!(
            r#"[{{"content": "{}", "status": "pending", "activeForm": null}}]"#,
            long_content
        );

        let entries = parse_todo_file(&json).unwrap();
        assert_eq!(entries[0].content.len(), 10000);
    }

    // =========================================================================
    // TodoTrackerError Tests
    // =========================================================================

    #[test]
    fn todo_tracker_error_display() {
        let err = TodoTrackerError::ChannelClosed;
        assert_eq!(err.to_string(), "failed to send event: channel closed");

        let err = TodoTrackerError::ClaudeDirectoryNotFound(PathBuf::from("/test/todos"));
        assert_eq!(
            err.to_string(),
            "claude todos directory not found: /test/todos"
        );
    }

    #[test]
    fn todo_tracker_error_is_debug() {
        let err = TodoTrackerError::ChannelClosed;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ChannelClosed"));

        let err = TodoTrackerError::ClaudeDirectoryNotFound(PathBuf::from("/test"));
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ClaudeDirectoryNotFound"));
    }

    // =========================================================================
    // TodoTrackerConfig Tests
    // =========================================================================

    #[test]
    fn todo_tracker_config_default() {
        let config = TodoTrackerConfig::default();
        assert_eq!(config.debounce_ms, 100);
    }

    #[test]
    fn todo_tracker_config_clone() {
        let config = TodoTrackerConfig { debounce_ms: 200 };
        let cloned = config.clone();
        assert_eq!(cloned.debounce_ms, 200);
    }

    // =========================================================================
    // TodoTracker File Operations Tests
    // =========================================================================

    use std::io::Write;
    use tempfile::TempDir;
    use tokio::time::{sleep, timeout, Duration as TokioDuration};

    /// Sample todo file content for testing.
    const SAMPLE_TODO: &str = r#"[
        {"content": "Task 1", "status": "completed", "activeForm": null},
        {"content": "Task 2", "status": "in_progress", "activeForm": "Working on task 2..."},
        {"content": "Task 3", "status": "pending", "activeForm": null}
    ]"#;

    /// Creates a temporary directory with a todo file.
    fn create_test_todo_file(dir: &TempDir, session_id: &str, content: &str) -> PathBuf {
        let filename = format!("{}-agent-{}.json", session_id, session_id);
        let todo_path = dir.path().join(&filename);

        let mut file = std::fs::File::create(&todo_path).expect("Failed to create todo file");
        file.write_all(content.as_bytes())
            .expect("Failed to write todo content");
        file.flush().expect("Failed to flush");

        todo_path
    }

    #[tokio::test]
    async fn test_tracker_creation_missing_directory() {
        let (tx, _rx) = mpsc::channel(100);
        let result = TodoTracker::with_path(PathBuf::from("/nonexistent/todos"), tx);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TodoTrackerError::ClaudeDirectoryNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_tracker_creation_with_valid_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let result = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx);

        assert!(result.is_ok(), "Should create tracker for valid directory");
        assert_eq!(result.unwrap().todos_dir(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_tracker_mark_session_ended() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Initially, session should not be ended
        assert!(!tracker.is_session_ended("sess-123").await);
        assert_eq!(tracker.ended_sessions_count().await, 0);

        // Mark session as ended
        tracker.mark_session_ended("sess-123").await;

        // Now it should be ended
        assert!(tracker.is_session_ended("sess-123").await);
        assert_eq!(tracker.ended_sessions_count().await, 1);

        // Other sessions should not be affected
        assert!(!tracker.is_session_ended("sess-456").await);
    }

    #[tokio::test]
    async fn test_tracker_clear_session_ended() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Mark session as ended
        tracker.mark_session_ended("sess-123").await;
        assert!(tracker.is_session_ended("sess-123").await);

        // Clear the ended status
        tracker.clear_session_ended("sess-123").await;
        assert!(!tracker.is_session_ended("sess-123").await);
    }

    #[tokio::test]
    async fn test_tracker_multiple_ended_sessions() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Mark multiple sessions as ended
        tracker.mark_session_ended("sess-1").await;
        tracker.mark_session_ended("sess-2").await;
        tracker.mark_session_ended("sess-3").await;

        assert_eq!(tracker.ended_sessions_count().await, 3);
        assert!(tracker.is_session_ended("sess-1").await);
        assert!(tracker.is_session_ended("sess-2").await);
        assert!(tracker.is_session_ended("sess-3").await);
        assert!(!tracker.is_session_ended("sess-4").await);
    }

    #[tokio::test]
    async fn test_tracker_detects_new_todo_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a new todo file
        let session_id = "6e45a55c-3124-4cc8-ad85-040a5c316009";
        create_test_todo_file(&temp_dir, session_id, SAMPLE_TODO);

        // Should receive a todo progress event
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event for new todo file");

        let event = result.unwrap().unwrap();
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.completed, 1);
        assert_eq!(event.in_progress, 1);
        assert_eq!(event.pending, 1);
        assert!(!event.abandoned);
    }

    #[tokio::test]
    async fn test_tracker_detects_modified_todo_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_id = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";

        // Create initial todo file
        let todo_path = create_test_todo_file(&temp_dir, session_id, SAMPLE_TODO);

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Modify the todo file (all tasks completed)
        let updated_todo = r#"[
            {"content": "Task 1", "status": "completed", "activeForm": null},
            {"content": "Task 2", "status": "completed", "activeForm": null},
            {"content": "Task 3", "status": "completed", "activeForm": null}
        ]"#;
        let mut file = std::fs::File::create(&todo_path).expect("Should open file");
        file.write_all(updated_todo.as_bytes())
            .expect("Should write");
        file.flush().expect("Should flush");

        // Should receive an updated event
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(
            result.is_ok(),
            "Should receive event for modified todo file"
        );

        let event = result.unwrap().unwrap();
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.completed, 3);
        assert_eq!(event.in_progress, 0);
        assert_eq!(event.pending, 0);
        assert!(!event.abandoned);
    }

    #[tokio::test]
    async fn test_tracker_abandonment_detection() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_id = "f1e2d3c4-b5a6-0987-fedc-ba9876543210";

        let (tx, mut rx) = mpsc::channel(100);
        let tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Mark the session as ended BEFORE creating the file
        tracker.mark_session_ended(session_id).await;

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a todo file with incomplete tasks
        let incomplete_todo = r#"[
            {"content": "Task 1", "status": "completed", "activeForm": null},
            {"content": "Task 2", "status": "pending", "activeForm": null}
        ]"#;
        create_test_todo_file(&temp_dir, session_id, incomplete_todo);

        // Should receive an event with abandoned=true
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event");

        let event = result.unwrap().unwrap();
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.completed, 1);
        assert_eq!(event.pending, 1);
        assert!(event.abandoned, "Should be marked as abandoned");
    }

    #[tokio::test]
    async fn test_tracker_no_abandonment_for_active_session() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_id = "11111111-2222-3333-4444-555555555555";

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a todo file with incomplete tasks (session NOT ended)
        let incomplete_todo = r#"[
            {"content": "Task 1", "status": "in_progress", "activeForm": "Working..."},
            {"content": "Task 2", "status": "pending", "activeForm": null}
        ]"#;
        create_test_todo_file(&temp_dir, session_id, incomplete_todo);

        // Should receive an event with abandoned=false (session not ended)
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event");

        let event = result.unwrap().unwrap();
        assert!(
            !event.abandoned,
            "Should NOT be marked as abandoned for active session"
        );
    }

    #[tokio::test]
    async fn test_tracker_ignores_non_json_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a non-json file
        let non_json_path = temp_dir.path().join("some-file.txt");
        std::fs::write(&non_json_path, "not a json file").expect("Should write");

        // Should NOT receive any event
        let result = timeout(TokioDuration::from_millis(200), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should not receive event for non-json file"
        );
    }

    #[tokio::test]
    async fn test_tracker_ignores_invalid_filename_format() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a json file with invalid filename format
        let invalid_path = temp_dir.path().join("not-a-valid-todo-filename.json");
        std::fs::write(&invalid_path, SAMPLE_TODO).expect("Should write");

        // Should NOT receive any event
        let result = timeout(TokioDuration::from_millis(200), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should not receive event for invalid filename format"
        );
    }

    #[tokio::test]
    async fn test_tracker_handles_empty_todo_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_id = "00000000-0000-0000-0000-000000000000";

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = TodoTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create an empty todo file
        create_test_todo_file(&temp_dir, session_id, "[]");

        // Should receive an event with zero counts
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event for empty todo file");

        let event = result.unwrap().unwrap();
        assert_eq!(event.completed, 0);
        assert_eq!(event.in_progress, 0);
        assert_eq!(event.pending, 0);
        assert!(!event.abandoned);
    }

    #[tokio::test]
    async fn test_tracker_config_with_custom_debounce() {
        // Test that custom debounce config is accepted
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let config = TodoTrackerConfig { debounce_ms: 50 };
        let tracker = TodoTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
            .expect("Should create tracker");

        // Tracker should be created successfully with custom debounce config
        assert_eq!(tracker.todos_dir(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_process_debounced_changes_integration() {
        // Integration test: manually invoke the processing logic
        // to verify events are emitted correctly
        use crate::utils::debounce::Debouncer;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_id = "00000000-1111-2222-3333-444444444444";

        // Create a todo file directly
        let todo_content = r#"[
            {"content": "Task 1", "status": "completed", "activeForm": null},
            {"content": "Task 2", "status": "pending", "activeForm": null}
        ]"#;
        let todo_path = create_test_todo_file(&temp_dir, session_id, todo_content);

        // Set up channels for the processing task
        let (event_tx, mut event_rx) = mpsc::channel(100);
        let ended_sessions = Arc::new(RwLock::new(HashSet::new()));

        // Set up debouncer that outputs to a channel we receive from
        let (debounce_output_tx, debounce_output_rx) = mpsc::channel(100);

        // Spawn the processing task
        let ended_for_task = Arc::clone(&ended_sessions);
        tokio::spawn(async move {
            process_debounced_changes(debounce_output_rx, event_tx, ended_for_task).await;
        });

        // Manually send the "debounced" path to simulate what would happen
        // after file watcher detects a change and debouncer coalesces it
        debounce_output_tx
            .send((todo_path.clone(), todo_path.clone()))
            .await
            .expect("Should send to debounce output");

        // Should receive the event
        let result = timeout(TokioDuration::from_millis(500), event_rx.recv()).await;
        assert!(result.is_ok(), "Should receive event");

        let event = result.unwrap().unwrap();
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.completed, 1);
        assert_eq!(event.pending, 1);
        assert!(!event.abandoned);
    }

    #[tokio::test]
    async fn test_process_debounced_changes_with_ended_session() {
        // Test that abandonment is detected when session is marked as ended
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_id = "55555555-6666-7777-8888-999999999999";

        // Create a todo file with incomplete tasks
        let todo_content = r#"[
            {"content": "Task 1", "status": "completed", "activeForm": null},
            {"content": "Task 2", "status": "in_progress", "activeForm": "Working..."}
        ]"#;
        let todo_path = create_test_todo_file(&temp_dir, session_id, todo_content);

        // Set up channels
        let (event_tx, mut event_rx) = mpsc::channel(100);
        let ended_sessions = Arc::new(RwLock::new(HashSet::new()));

        // Mark session as ended BEFORE processing
        {
            let mut sessions = ended_sessions.write().await;
            sessions.insert(session_id.to_string());
        }

        // Set up the output channel
        let (debounce_output_tx, debounce_output_rx) = mpsc::channel(100);

        // Spawn the processing task
        let ended_for_task = Arc::clone(&ended_sessions);
        tokio::spawn(async move {
            process_debounced_changes(debounce_output_rx, event_tx, ended_for_task).await;
        });

        // Send the path
        debounce_output_tx
            .send((todo_path.clone(), todo_path.clone()))
            .await
            .expect("Should send");

        // Should receive event with abandoned=true
        let result = timeout(TokioDuration::from_millis(500), event_rx.recv()).await;
        assert!(result.is_ok(), "Should receive event");

        let event = result.unwrap().unwrap();
        assert_eq!(event.session_id, session_id);
        assert!(event.abandoned, "Should be marked as abandoned");
    }

    #[tokio::test]
    async fn test_tracker_custom_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let config = TodoTrackerConfig { debounce_ms: 250 };
        let tracker = TodoTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
            .expect("Should create tracker");

        // Tracker should be created successfully with custom config
        assert_eq!(tracker.todos_dir(), temp_dir.path());
    }
}
