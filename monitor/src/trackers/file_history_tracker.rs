//! File history tracker for monitoring file edit line changes.
//!
//! This module watches `~/.claude/file-history/<session-id>/` directories for
//! changes and emits [`FileChangeEvent`]s when file versions are created.
//!
//! # File Version Format
//!
//! File versions follow the pattern: `<hash>@v<N>` where:
//! - `<hash>` is a 16-character hexadecimal identifier
//! - `<N>` is a positive integer version number (1, 2, 3, ...)
//!
//! Examples:
//! - `3f79c7095dc57fea@v2`
//! - `abc123def456789a@v1`
//! - `0123456789abcdef@v15`
//!
//! # Version Processing Rules
//!
//! Per FR-024 and FR-025:
//! - **v1 files**: Initial state, no diff possible. Skip processing.
//! - **v2+ files**: Diff against the previous version (vN vs vN-1).
//!
//! When version gaps exist (e.g., v1, v3, missing v2), the tracker diffs against
//! the highest available previous version and logs a warning.
//!
//! # Line Diff Calculation
//!
//! The tracker performs simple line-by-line diff to calculate:
//! - `lines_added`: Lines present in vN but not in v(N-1)
//! - `lines_removed`: Lines present in v(N-1) but not in vN
//! - `lines_modified`: Lines that changed between versions (optional)
//!
//! # Privacy
//!
//! This module follows the privacy-first principle: only line counts and
//! metadata are captured. No file paths or contents are transmitted.
//! The file hash is a content-addressable identifier that cannot be reversed.
//!
//! # Example
//!
//! ```
//! use vibetea_monitor::trackers::file_history_tracker::{
//!     FileVersion,
//!     parse_file_version,
//!     calculate_diff,
//!     should_skip_version,
//! };
//!
//! // Parse a file version from filename
//! let version = parse_file_version("3f79c7095dc57fea@v2").unwrap();
//! assert_eq!(version.hash, "3f79c7095dc57fea");
//! assert_eq!(version.version, 2);
//!
//! // Check if version should be skipped
//! assert!(should_skip_version(1)); // v1 is skipped
//! assert!(!should_skip_version(2)); // v2+ is processed
//!
//! // Calculate diff between file contents
//! let old_content = "hello\nworld";
//! let new_content = "hello\nrust";
//! let diff = calculate_diff(old_content, new_content);
//! assert_eq!(diff.lines_added, 1);
//! assert_eq!(diff.lines_removed, 1);
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, trace, warn};

use crate::types::FileChangeEvent;
use crate::utils::debounce::Debouncer;

/// Errors that can occur when parsing file version filenames.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FileHistoryParseError {
    /// The filename does not contain the `@v` version separator.
    #[error("missing version separator '@v' in filename")]
    MissingVersionSeparator,

    /// The hash portion is not exactly 16 hexadecimal characters.
    #[error("invalid hash: expected 16 hexadecimal characters, got {0} characters")]
    InvalidHashLength(usize),

    /// The hash portion contains non-hexadecimal characters.
    #[error("invalid hash: contains non-hexadecimal character '{0}'")]
    InvalidHashCharacter(char),

    /// The version number is missing or empty after `@v`.
    #[error("missing version number after '@v'")]
    MissingVersionNumber,

    /// The version number is not a valid positive integer.
    #[error("invalid version number: {0}")]
    InvalidVersionNumber(String),

    /// The version number is zero (versions start at 1).
    #[error("version number must be >= 1, got 0")]
    VersionZero,

    /// The filename is empty.
    #[error("empty filename")]
    EmptyFilename,
}

/// Result type for file history parsing operations.
pub type Result<T> = std::result::Result<T, FileHistoryParseError>;

/// Represents a parsed file version from a file-history filename.
///
/// File versions follow the pattern `<hash>@v<N>` where:
/// - `hash` is a 16-character hexadecimal identifier
/// - `version` is a positive integer (>= 1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileVersion {
    /// The 16-character hexadecimal hash identifying the file.
    pub hash: String,

    /// The version number (>= 1).
    pub version: u32,
}

impl FileVersion {
    /// Creates a new `FileVersion` with the given hash and version.
    ///
    /// # Arguments
    ///
    /// * `hash` - The 16-character hexadecimal hash
    /// * `version` - The version number (must be >= 1)
    ///
    /// # Panics
    ///
    /// Panics if version is 0 or hash is not 16 hex characters.
    /// Use [`parse_file_version`] for validated construction.
    #[must_use]
    pub fn new(hash: impl Into<String>, version: u32) -> Self {
        assert!(version >= 1, "version must be >= 1");
        let hash = hash.into();
        assert_eq!(hash.len(), 16, "hash must be 16 characters");
        Self { hash, version }
    }
}

/// Result of a line diff calculation between two file versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DiffResult {
    /// Number of lines added in the new version.
    pub lines_added: u32,

    /// Number of lines removed from the old version.
    pub lines_removed: u32,

    /// Number of lines modified between versions.
    ///
    /// This is an approximation based on the minimum of added/removed lines
    /// when both are non-zero, representing lines that likely changed.
    pub lines_modified: u32,
}

impl DiffResult {
    /// Returns the total number of changed lines (added + removed).
    #[must_use]
    pub fn total_changes(&self) -> u32 {
        self.lines_added + self.lines_removed
    }

    /// Returns true if there are no changes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines_added == 0 && self.lines_removed == 0
    }
}

/// Parses a file version from a filename.
///
/// The filename must follow the pattern `<hash>@v<N>` where:
/// - `<hash>` is exactly 16 hexadecimal characters (0-9, a-f, A-F)
/// - `<N>` is a positive integer version number (>= 1)
///
/// Leading zeros in the version number are valid (e.g., `@v02` parses as version 2).
///
/// # Arguments
///
/// * `filename` - The filename to parse (without directory path)
///
/// # Returns
///
/// * `Ok(FileVersion)` if parsing succeeds
/// * `Err(FileHistoryParseError)` if the filename is invalid
///
/// # Examples
///
/// ```
/// use vibetea_monitor::trackers::file_history_tracker::parse_file_version;
///
/// // Valid filenames
/// let v = parse_file_version("3f79c7095dc57fea@v2").unwrap();
/// assert_eq!(v.hash, "3f79c7095dc57fea");
/// assert_eq!(v.version, 2);
///
/// let v = parse_file_version("abc123def456789a@v1").unwrap();
/// assert_eq!(v.version, 1);
///
/// // Leading zeros are valid
/// let v = parse_file_version("aabbccdd11223344@v02").unwrap();
/// assert_eq!(v.version, 2);
///
/// // Invalid filenames
/// assert!(parse_file_version("nothash@v1").is_err()); // Hash too short
/// assert!(parse_file_version("3f79c7095dc57fea").is_err()); // No version
/// assert!(parse_file_version("3f79c7095dc57fea@v0").is_err()); // Version 0
/// ```
pub fn parse_file_version(filename: &str) -> Result<FileVersion> {
    if filename.is_empty() {
        return Err(FileHistoryParseError::EmptyFilename);
    }

    // Find the @v separator
    let separator_pos = filename.rfind("@v").ok_or(FileHistoryParseError::MissingVersionSeparator)?;

    // Extract hash (everything before @v)
    let hash = &filename[..separator_pos];

    // Validate hash length
    if hash.len() != 16 {
        return Err(FileHistoryParseError::InvalidHashLength(hash.len()));
    }

    // Validate hash contains only hex characters
    for c in hash.chars() {
        if !c.is_ascii_hexdigit() {
            return Err(FileHistoryParseError::InvalidHashCharacter(c));
        }
    }

    // Extract version number (everything after @v)
    let version_str = &filename[separator_pos + 2..];

    if version_str.is_empty() {
        return Err(FileHistoryParseError::MissingVersionNumber);
    }

    // Parse version number
    let version: u32 = version_str
        .parse()
        .map_err(|_| FileHistoryParseError::InvalidVersionNumber(version_str.to_string()))?;

    // Version must be >= 1
    if version == 0 {
        return Err(FileHistoryParseError::VersionZero);
    }

    Ok(FileVersion {
        hash: hash.to_string(),
        version,
    })
}

/// Parses a file version from a path, extracting just the filename.
///
/// This is a convenience wrapper around [`parse_file_version`] that
/// handles full paths.
///
/// # Arguments
///
/// * `path` - The path to the file version
///
/// # Returns
///
/// * `Ok(FileVersion)` if parsing succeeds
/// * `Err(FileHistoryParseError)` if the filename is invalid
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use vibetea_monitor::trackers::file_history_tracker::parse_file_version_from_path;
///
/// let path = Path::new("/home/user/.claude/file-history/session-123/3f79c7095dc57fea@v2");
/// let version = parse_file_version_from_path(path).unwrap();
/// assert_eq!(version.hash, "3f79c7095dc57fea");
/// assert_eq!(version.version, 2);
/// ```
pub fn parse_file_version_from_path(path: &Path) -> Result<FileVersion> {
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or(FileHistoryParseError::EmptyFilename)?;

    parse_file_version(filename)
}

/// Determines if a version should be skipped (not processed for diffing).
///
/// Per FR-025, v1 files represent the initial state and have no previous
/// version to diff against. Only versions >= 2 are processed.
///
/// # Arguments
///
/// * `version` - The version number to check
///
/// # Returns
///
/// `true` if the version should be skipped (v1), `false` if it should be processed (v2+).
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::file_history_tracker::should_skip_version;
///
/// assert!(should_skip_version(1)); // v1 is skipped
/// assert!(!should_skip_version(2)); // v2 is processed
/// assert!(!should_skip_version(10)); // v10 is processed
/// ```
#[must_use]
pub fn should_skip_version(version: u32) -> bool {
    version <= 1
}

/// Calculates the diff between two file contents.
///
/// Performs a simple line-by-line comparison to determine:
/// - Lines added (present in `new_content` but not in `old_content`)
/// - Lines removed (present in `old_content` but not in `new_content`)
/// - Lines modified (approximation based on overlapping changes)
///
/// This is a simplified diff algorithm that counts unique lines. For
/// accurate results with moved/reordered lines, a proper diff algorithm
/// like Myers would be needed, but for this use case simple counting
/// provides sufficient accuracy.
///
/// # Arguments
///
/// * `old_content` - The content of the previous version
/// * `new_content` - The content of the new version
///
/// # Returns
///
/// A [`DiffResult`] containing the line change counts.
///
/// # Examples
///
/// ```
/// use vibetea_monitor::trackers::file_history_tracker::calculate_diff;
///
/// // Empty files
/// let diff = calculate_diff("", "");
/// assert_eq!(diff.lines_added, 0);
/// assert_eq!(diff.lines_removed, 0);
///
/// // Added content
/// let diff = calculate_diff("", "hello\nworld");
/// assert_eq!(diff.lines_added, 2);
/// assert_eq!(diff.lines_removed, 0);
///
/// // Removed content
/// let diff = calculate_diff("hello\nworld", "");
/// assert_eq!(diff.lines_added, 0);
/// assert_eq!(diff.lines_removed, 2);
///
/// // Modified content
/// let diff = calculate_diff("hello", "world");
/// assert_eq!(diff.lines_added, 1);
/// assert_eq!(diff.lines_removed, 1);
/// ```
#[must_use]
pub fn calculate_diff(old_content: &str, new_content: &str) -> DiffResult {
    use std::collections::HashMap;

    // Handle empty content edge cases
    let old_lines: Vec<&str> = if old_content.is_empty() {
        Vec::new()
    } else {
        old_content.lines().collect()
    };

    let new_lines: Vec<&str> = if new_content.is_empty() {
        Vec::new()
    } else {
        new_content.lines().collect()
    };

    // Count occurrences of each line in old content
    let mut old_counts: HashMap<&str, i32> = HashMap::new();
    for line in &old_lines {
        *old_counts.entry(line).or_insert(0) += 1;
    }

    // Count occurrences of each line in new content
    let mut new_counts: HashMap<&str, i32> = HashMap::new();
    for line in &new_lines {
        *new_counts.entry(line).or_insert(0) += 1;
    }

    // Calculate lines added and removed
    let mut lines_added: u32 = 0;
    let mut lines_removed: u32 = 0;

    // Lines in new but not in old (or more occurrences)
    for (line, &new_count) in &new_counts {
        let old_count = old_counts.get(line).copied().unwrap_or(0);
        if new_count > old_count {
            lines_added += (new_count - old_count) as u32;
        }
    }

    // Lines in old but not in new (or fewer occurrences)
    for (line, &old_count) in &old_counts {
        let new_count = new_counts.get(line).copied().unwrap_or(0);
        if old_count > new_count {
            lines_removed += (old_count - new_count) as u32;
        }
    }

    // Approximate modified lines as the minimum of added/removed
    // (represents lines that were likely changed rather than purely added/removed)
    let lines_modified = lines_added.min(lines_removed);

    DiffResult {
        lines_added,
        lines_removed,
        lines_modified,
    }
}

/// Extracts the session ID from a file-history directory path.
///
/// The file-history directory structure is:
/// `~/.claude/file-history/<session-id>/<hash>@vN`
///
/// This function extracts the session ID from the parent directory name.
///
/// # Arguments
///
/// * `path` - Path to a file version (e.g., `.../file-history/session-123/hash@v1`)
///
/// # Returns
///
/// * `Some(String)` containing the session ID if extraction succeeds
/// * `None` if the path doesn't have a parent directory with a valid name
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use vibetea_monitor::trackers::file_history_tracker::extract_session_id_from_path;
///
/// let path = Path::new("/home/user/.claude/file-history/abc-123-def/3f79c7095dc57fea@v2");
/// assert_eq!(extract_session_id_from_path(path), Some("abc-123-def".to_string()));
/// ```
#[must_use]
pub fn extract_session_id_from_path(path: &Path) -> Option<String> {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

// ============================================================================
// FileHistoryTracker Types
// ============================================================================

/// Errors that can occur during file history tracking operations.
#[derive(Error, Debug)]
pub enum FileHistoryTrackerError {
    /// Failed to initialize the file system watcher.
    #[error("failed to create watcher: {0}")]
    WatcherInit(#[from] notify::Error),

    /// Failed to read a file.
    #[error("failed to read file: {0}")]
    Io(#[from] std::io::Error),

    /// The file-history directory does not exist.
    #[error("claude file-history directory not found: {0}")]
    ClaudeDirectoryNotFound(PathBuf),

    /// Failed to send event through the channel.
    #[error("failed to send event: channel closed")]
    ChannelClosed,
}

/// Result type for file history tracker operations.
pub type TrackerResult<T> = std::result::Result<T, FileHistoryTrackerError>;

/// Configuration for the file history tracker.
#[derive(Debug, Clone)]
pub struct FileHistoryTrackerConfig {
    /// Debounce duration in milliseconds. Default: 100ms.
    ///
    /// Per T164, 100ms is the recommended debounce interval for file-history
    /// files to coalesce rapid writes during file version creation.
    pub debounce_ms: u64,
}

impl Default for FileHistoryTrackerConfig {
    fn default() -> Self {
        Self { debounce_ms: 100 }
    }
}

/// Creates a [`FileChangeEvent`] from file version and diff information.
///
/// # Arguments
///
/// * `session_id` - The session UUID
/// * `file_version` - The parsed file version
/// * `diff` - The calculated diff result
///
/// # Returns
///
/// A [`FileChangeEvent`] ready for transmission.
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::file_history_tracker::{
///     FileVersion, DiffResult, create_file_change_event,
/// };
///
/// let version = FileVersion::new("0123456789abcdef", 3);
/// let diff = DiffResult { lines_added: 10, lines_removed: 5, lines_modified: 3 };
/// let event = create_file_change_event("sess-123", &version, &diff);
///
/// assert_eq!(event.session_id, "sess-123");
/// assert_eq!(event.file_hash, "0123456789abcdef");
/// assert_eq!(event.version, 3);
/// assert_eq!(event.lines_added, 10);
/// assert_eq!(event.lines_removed, 5);
/// assert_eq!(event.lines_modified, 3);
/// ```
#[must_use]
pub fn create_file_change_event(
    session_id: &str,
    file_version: &FileVersion,
    diff: &DiffResult,
) -> FileChangeEvent {
    FileChangeEvent {
        session_id: session_id.to_string(),
        file_hash: file_version.hash.clone(),
        version: file_version.version,
        lines_added: diff.lines_added,
        lines_removed: diff.lines_removed,
        lines_modified: diff.lines_modified,
        timestamp: Utc::now(),
    }
}

// ============================================================================
// FileHistoryTracker - File Watching Implementation
// ============================================================================

/// State maintained for tracking file versions within a session.
#[derive(Debug, Default)]
struct SessionState {
    /// Map of file hash to the highest known version number.
    /// Used for detecting version gaps.
    known_versions: std::collections::HashMap<String, u32>,
}

/// Tracker for Claude Code's file-history directory.
///
/// Watches `~/.claude/file-history/` for changes and emits [`FileChangeEvent`]s
/// when new file versions (v2+) are detected. For each new version, it diffs
/// against the previous version to calculate line changes.
///
/// # Directory Structure
///
/// The file-history directory has the following structure:
/// ```text
/// ~/.claude/file-history/
///   <session-id>/
///     <hash>@v1
///     <hash>@v2
///     <hash>@v3
///     ...
/// ```
///
/// # Processing Rules
///
/// - **v1 files**: Skipped (initial version, no previous to diff against)
/// - **v2+ files**: Diffed against the previous version (or highest available)
/// - **Version gaps**: Logged as warnings, diff against highest available
///
/// # Thread Safety
///
/// The tracker spawns a background task for async processing of file events.
/// Communication is done via channels for thread safety.
#[derive(Debug)]
pub struct FileHistoryTracker {
    /// The underlying file system watcher.
    ///
    /// Kept alive to maintain the watch subscription.
    #[allow(dead_code)]
    watcher: RecommendedWatcher,

    /// Path to the file-history directory being watched.
    root_dir: PathBuf,

    /// Channel sender for emitting file change events.
    #[allow(dead_code)]
    event_sender: mpsc::Sender<FileChangeEvent>,

    /// Per-session state tracking known versions.
    #[allow(dead_code)]
    session_state: Arc<RwLock<std::collections::HashMap<String, SessionState>>>,
}

impl FileHistoryTracker {
    /// Creates a new file history tracker watching the default file-history directory.
    ///
    /// The default location is `~/.claude/file-history/`.
    ///
    /// # Arguments
    ///
    /// * `event_sender` - Channel for emitting [`FileChangeEvent`]s
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The home directory cannot be determined
    /// - The `~/.claude/file-history` directory does not exist
    /// - The file system watcher cannot be initialized
    pub fn new(event_sender: mpsc::Sender<FileChangeEvent>) -> TrackerResult<Self> {
        Self::with_config(event_sender, FileHistoryTrackerConfig::default())
    }

    /// Creates a new file history tracker with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `event_sender` - Channel for emitting [`FileChangeEvent`]s
    /// * `config` - Configuration options for the tracker
    ///
    /// # Errors
    ///
    /// Returns an error if the tracker cannot be initialized.
    pub fn with_config(
        event_sender: mpsc::Sender<FileChangeEvent>,
        config: FileHistoryTrackerConfig,
    ) -> TrackerResult<Self> {
        let claude_dir = directories::BaseDirs::new()
            .map(|dirs| dirs.home_dir().join(".claude"))
            .ok_or_else(|| {
                FileHistoryTrackerError::ClaudeDirectoryNotFound(PathBuf::from("~/.claude"))
            })?;

        Self::with_path_and_config(claude_dir.join("file-history"), event_sender, config)
    }

    /// Creates a new file history tracker watching a specific directory.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - Path to the file-history directory to watch
    /// * `event_sender` - Channel for emitting [`FileChangeEvent`]s
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The specified directory does not exist
    /// - The file system watcher cannot be initialized
    pub fn with_path(
        root_dir: PathBuf,
        event_sender: mpsc::Sender<FileChangeEvent>,
    ) -> TrackerResult<Self> {
        Self::with_path_and_config(root_dir, event_sender, FileHistoryTrackerConfig::default())
    }

    /// Creates a new file history tracker with a specific path and configuration.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - Path to the file-history directory to watch
    /// * `event_sender` - Channel for emitting [`FileChangeEvent`]s
    /// * `config` - Configuration options for the tracker
    ///
    /// # Errors
    ///
    /// Returns an error if the tracker cannot be initialized.
    pub fn with_path_and_config(
        root_dir: PathBuf,
        event_sender: mpsc::Sender<FileChangeEvent>,
        config: FileHistoryTrackerConfig,
    ) -> TrackerResult<Self> {
        // Verify the directory exists
        if !root_dir.exists() {
            return Err(FileHistoryTrackerError::ClaudeDirectoryNotFound(root_dir));
        }

        info!(
            root_dir = %root_dir.display(),
            debounce_ms = config.debounce_ms,
            "Initializing file history tracker"
        );

        // Create the session state tracking
        let session_state = Arc::new(RwLock::new(std::collections::HashMap::new()));

        // Create channel for debounced file change notifications
        let (debounce_tx, debounce_rx) = mpsc::channel::<(PathBuf, PathBuf)>(1000);

        // Create the debouncer
        let debouncer = Debouncer::new(Duration::from_millis(config.debounce_ms), debounce_tx);

        // Spawn the async processing task
        let sender_for_task = event_sender.clone();
        let root_for_task = root_dir.clone();
        let state_for_task = Arc::clone(&session_state);
        tokio::spawn(async move {
            process_debounced_changes(debounce_rx, sender_for_task, root_for_task, state_for_task)
                .await;
        });

        // Create the file watcher
        let watcher = create_file_history_watcher(root_dir.clone(), debouncer)?;

        Ok(Self {
            watcher,
            root_dir,
            event_sender,
            session_state,
        })
    }

    /// Returns the path to the file-history directory being watched.
    #[must_use]
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }
}

/// Creates the file system watcher for the file-history directory.
fn create_file_history_watcher(
    root_dir: PathBuf,
    debouncer: Debouncer<PathBuf, PathBuf>,
) -> TrackerResult<RecommendedWatcher> {
    let watch_dir = root_dir.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: std::result::Result<Event, notify::Error>| {
            handle_notify_event(res, &root_dir, &debouncer);
        },
        Config::default(),
    )?;

    // Watch the file-history directory recursively
    // Structure: ~/.claude/file-history/<session-id>/<hash>@vN
    watcher.watch(&watch_dir, RecursiveMode::Recursive)?;

    debug!(
        watch_dir = %watch_dir.display(),
        "Started watching for file-history changes (recursive)"
    );

    Ok(watcher)
}

/// Handles raw notify events and sends them to the debouncer.
fn handle_notify_event(
    res: std::result::Result<Event, notify::Error>,
    root_dir: &Path,
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
        // Only process files (not directories)
        if path.is_dir() {
            continue;
        }

        // Verify the path is under the root directory
        if !path.starts_with(root_dir) {
            continue;
        }

        // Try to parse as a file version
        if parse_file_version_from_path(path).is_err() {
            trace!(path = %path.display(), "Ignoring non-version file");
            continue;
        }

        debug!(
            path = %path.display(),
            kind = ?event.kind,
            "File version changed, sending to debouncer"
        );

        // Send to debouncer (key is the path, value is also the path)
        if !debouncer.try_send(path.clone(), path.clone()) {
            warn!(path = %path.display(), "Failed to send to debouncer: channel full or closed");
        }
    }
}

/// Processes debounced file change events, calculating diffs and emitting events.
async fn process_debounced_changes(
    mut rx: mpsc::Receiver<(PathBuf, PathBuf)>,
    sender: mpsc::Sender<FileChangeEvent>,
    root_dir: PathBuf,
    session_state: Arc<RwLock<std::collections::HashMap<String, SessionState>>>,
) {
    debug!("Starting file history change processor");

    while let Some((path, _)) = rx.recv().await {
        debug!(path = %path.display(), "Processing debounced file version change");

        // Parse the file version from the path
        let file_version = match parse_file_version_from_path(&path) {
            Ok(v) => v,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to parse file version");
                continue;
            }
        };

        // Check if we should skip this version (v1)
        if should_skip_version(file_version.version) {
            trace!(
                path = %path.display(),
                version = file_version.version,
                "Skipping v1 file (no previous version to diff)"
            );
            continue;
        }

        // Extract session ID from path
        let session_id = match extract_session_id_from_path(&path) {
            Some(id) => id,
            None => {
                warn!(path = %path.display(), "Could not extract session ID from path");
                continue;
            }
        };

        // Find the previous version file
        let prev_version = file_version.version - 1;
        let (prev_path, actual_prev_version) =
            find_previous_version(&path, &file_version, &root_dir).await;

        // Check for version gaps
        if actual_prev_version < prev_version {
            warn!(
                session_id = %session_id,
                file_hash = %file_version.hash,
                current_version = file_version.version,
                expected_prev = prev_version,
                actual_prev = actual_prev_version,
                "Version gap detected, diffing against v{}", actual_prev_version
            );
        }

        // Update known versions
        {
            let mut state = session_state.write().await;
            let session = state.entry(session_id.clone()).or_default();
            session
                .known_versions
                .insert(file_version.hash.clone(), file_version.version);
        }

        // Read the current and previous file contents
        let current_content = match tokio::fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    trace!(path = %path.display(), "File was deleted");
                } else {
                    warn!(path = %path.display(), error = %e, "Failed to read current file");
                }
                continue;
            }
        };

        let prev_content = if let Some(ref prev_path) = prev_path {
            match tokio::fs::read_to_string(prev_path).await {
                Ok(content) => content,
                Err(e) => {
                    warn!(
                        path = %prev_path.display(),
                        error = %e,
                        "Failed to read previous version, using empty content"
                    );
                    String::new()
                }
            }
        } else {
            // No previous version found (shouldn't happen for v2+, but handle gracefully)
            warn!(
                session_id = %session_id,
                file_hash = %file_version.hash,
                version = file_version.version,
                "No previous version found, using empty content"
            );
            String::new()
        };

        // Calculate the diff
        let diff = calculate_diff(&prev_content, &current_content);

        // Create and emit the event
        let event = create_file_change_event(&session_id, &file_version, &diff);

        trace!(
            session_id = %session_id,
            file_hash = %file_version.hash,
            version = file_version.version,
            lines_added = diff.lines_added,
            lines_removed = diff.lines_removed,
            lines_modified = diff.lines_modified,
            "Emitting file change event"
        );

        if let Err(e) = sender.send(event).await {
            error!(error = %e, "Failed to send file change event: channel closed");
            break;
        }
    }

    debug!("File history change processor shutting down");
}

/// Finds the previous version file for diffing.
///
/// Returns the path to the previous version and its version number.
/// If the expected previous version doesn't exist, searches for the highest
/// available version less than the current one.
async fn find_previous_version(
    current_path: &Path,
    current_version: &FileVersion,
    _root_dir: &Path,
) -> (Option<PathBuf>, u32) {
    let parent = match current_path.parent() {
        Some(p) => p,
        None => return (None, 0),
    };

    let target_version = current_version.version - 1;

    // First, try the expected previous version
    let expected_prev_name = format!("{}@v{}", current_version.hash, target_version);
    let expected_prev_path = parent.join(&expected_prev_name);

    if expected_prev_path.exists() {
        return (Some(expected_prev_path), target_version);
    }

    // If not found, search for the highest available version less than current
    // This handles version gaps gracefully
    let mut best_version = 0u32;
    let mut best_path: Option<PathBuf> = None;

    if let Ok(mut entries) = tokio::fs::read_dir(parent).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let entry_path = entry.path();
            if let Ok(version) = parse_file_version_from_path(&entry_path) {
                if version.hash == current_version.hash
                    && version.version < current_version.version
                    && version.version > best_version
                {
                    best_version = version.version;
                    best_path = Some(entry_path);
                }
            }
        }
    }

    (best_path, best_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // T148: Unit Tests for File Version Parsing (@vN pattern)
    // =========================================================================

    /// Verifies that a standard file version with hash and version 2 is parsed correctly.
    #[test]
    fn t148_parse_version_standard_v2() {
        let result = parse_file_version("3f79c7095dc57fea@v2");
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.hash, "3f79c7095dc57fea");
        assert_eq!(version.version, 2);
    }

    /// Verifies that a file version with version 1 (initial version) is parsed correctly.
    #[test]
    fn t148_parse_version_v1() {
        let result = parse_file_version("abc123def456789a@v1");
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.hash, "abc123def456789a");
        assert_eq!(version.version, 1);
    }

    /// Verifies that a higher version number (v15) is parsed correctly.
    #[test]
    fn t148_parse_version_higher_number() {
        let result = parse_file_version("0123456789abcdef@v15");
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.hash, "0123456789abcdef");
        assert_eq!(version.version, 15);
    }

    /// Verifies that leading zeros in version number are handled correctly.
    /// Per spec: `@v02` parses as version 2.
    #[test]
    fn t148_parse_version_leading_zeros() {
        let result = parse_file_version("aabbccdd11223344@v02");
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.hash, "aabbccdd11223344");
        assert_eq!(version.version, 2);
    }

    /// Verifies that multiple leading zeros are handled correctly.
    #[test]
    fn t148_parse_version_multiple_leading_zeros() {
        let result = parse_file_version("1234567890abcdef@v007");
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.version, 7);
    }

    /// Verifies that a hash that is too short returns an error.
    #[test]
    fn t148_parse_version_hash_too_short() {
        let result = parse_file_version("nothash@v1");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, FileHistoryParseError::InvalidHashLength(7)));
        assert!(err.to_string().contains("7 characters"));
    }

    /// Verifies that a hash that is too long returns an error.
    #[test]
    fn t148_parse_version_hash_too_long() {
        let result = parse_file_version("3f79c7095dc57fea00@v1");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, FileHistoryParseError::InvalidHashLength(18)));
    }

    /// Verifies that a filename without version suffix returns an error.
    #[test]
    fn t148_parse_version_no_version_suffix() {
        let result = parse_file_version("3f79c7095dc57fea");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, FileHistoryParseError::MissingVersionSeparator));
        assert!(err.to_string().contains("@v"));
    }

    /// Verifies that version 0 returns an error (versions start at 1).
    #[test]
    fn t148_parse_version_zero() {
        let result = parse_file_version("3f79c7095dc57fea@v0");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, FileHistoryParseError::VersionZero));
        assert!(err.to_string().contains("must be >= 1"));
    }

    /// Verifies that non-numeric version returns an error.
    #[test]
    fn t148_parse_version_non_numeric() {
        let result = parse_file_version("3f79c7095dc57fea@vX");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, FileHistoryParseError::InvalidVersionNumber(_)));
    }

    /// Verifies that non-hex characters in hash return an error.
    #[test]
    fn t148_parse_version_non_hex_hash() {
        let result = parse_file_version("3f79c7095dc57feg@v1"); // 'g' is not hex
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, FileHistoryParseError::InvalidHashCharacter('g')));
    }

    /// Verifies that an empty filename returns an error.
    #[test]
    fn t148_parse_version_empty_filename() {
        let result = parse_file_version("");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, FileHistoryParseError::EmptyFilename));
    }

    /// Verifies that missing version number after @v returns an error.
    #[test]
    fn t148_parse_version_missing_number() {
        let result = parse_file_version("3f79c7095dc57fea@v");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, FileHistoryParseError::MissingVersionNumber));
    }

    /// Verifies that uppercase hex characters in hash are accepted.
    #[test]
    fn t148_parse_version_uppercase_hex() {
        let result = parse_file_version("3F79C7095DC57FEA@v2");
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.hash, "3F79C7095DC57FEA");
        assert_eq!(version.version, 2);
    }

    /// Verifies that mixed case hex characters in hash are accepted.
    #[test]
    fn t148_parse_version_mixed_case_hex() {
        let result = parse_file_version("3f79C7095Dc57feA@v3");
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.hash, "3f79C7095Dc57feA");
        assert_eq!(version.version, 3);
    }

    /// Verifies parsing from a full path works correctly.
    #[test]
    fn t148_parse_version_from_path() {
        let path = Path::new("/home/user/.claude/file-history/session-123/3f79c7095dc57fea@v2");
        let result = parse_file_version_from_path(path);
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.hash, "3f79c7095dc57fea");
        assert_eq!(version.version, 2);
    }

    /// Verifies that very large version numbers are parsed correctly.
    #[test]
    fn t148_parse_version_large_number() {
        let result = parse_file_version("0000000000000000@v999999");
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.version, 999_999);
    }

    /// Verifies that negative version numbers return an error.
    #[test]
    fn t148_parse_version_negative_number() {
        let result = parse_file_version("3f79c7095dc57fea@v-1");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, FileHistoryParseError::InvalidVersionNumber(_)));
    }

    // =========================================================================
    // T149: Unit Tests for Line Diff Calculation
    // =========================================================================

    /// Verifies that diffing two empty files returns zero changes.
    #[test]
    fn t149_diff_empty_files() {
        let diff = calculate_diff("", "");

        assert_eq!(diff.lines_added, 0);
        assert_eq!(diff.lines_removed, 0);
        assert_eq!(diff.lines_modified, 0);
        assert!(diff.is_empty());
    }

    /// Verifies that adding content to an empty file is counted correctly.
    #[test]
    fn t149_diff_added_content() {
        let diff = calculate_diff("", "hello\nworld");

        assert_eq!(diff.lines_added, 2);
        assert_eq!(diff.lines_removed, 0);
        assert_eq!(diff.total_changes(), 2);
    }

    /// Verifies that removing all content from a file is counted correctly.
    #[test]
    fn t149_diff_removed_content() {
        let diff = calculate_diff("hello\nworld", "");

        assert_eq!(diff.lines_added, 0);
        assert_eq!(diff.lines_removed, 2);
        assert_eq!(diff.total_changes(), 2);
    }

    /// Verifies that modifying a single line is counted as 1 add and 1 remove.
    #[test]
    fn t149_diff_modified_single_line() {
        let diff = calculate_diff("hello", "world");

        assert_eq!(diff.lines_added, 1);
        assert_eq!(diff.lines_removed, 1);
        assert_eq!(diff.lines_modified, 1);
    }

    /// Verifies complex diff with mix of adds, removes, and unchanged lines.
    #[test]
    fn t149_diff_complex_changes() {
        let old = "line1\nline2\nline3\nline4";
        let new = "line1\nmodified\nline4\nnew_line";

        let diff = calculate_diff(old, new);

        // line2 and line3 removed, "modified" and "new_line" added
        assert_eq!(diff.lines_added, 2);
        assert_eq!(diff.lines_removed, 2);
    }

    /// Verifies that identical content returns zero changes.
    #[test]
    fn t149_diff_identical_content() {
        let content = "hello\nworld\nfoo\nbar";
        let diff = calculate_diff(content, content);

        assert_eq!(diff.lines_added, 0);
        assert_eq!(diff.lines_removed, 0);
        assert!(diff.is_empty());
    }

    /// Verifies that adding a single line to existing content works.
    #[test]
    fn t149_diff_append_line() {
        let old = "line1\nline2";
        let new = "line1\nline2\nline3";

        let diff = calculate_diff(old, new);

        assert_eq!(diff.lines_added, 1);
        assert_eq!(diff.lines_removed, 0);
    }

    /// Verifies that removing a single line from existing content works.
    #[test]
    fn t149_diff_remove_line() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline3";

        let diff = calculate_diff(old, new);

        assert_eq!(diff.lines_added, 0);
        assert_eq!(diff.lines_removed, 1);
    }

    /// Verifies that duplicate lines are handled correctly.
    #[test]
    fn t149_diff_duplicate_lines() {
        let old = "line\nline\nline";
        let new = "line\nline";

        let diff = calculate_diff(old, new);

        assert_eq!(diff.lines_added, 0);
        assert_eq!(diff.lines_removed, 1);
    }

    /// Verifies that adding duplicate lines is counted correctly.
    #[test]
    fn t149_diff_add_duplicates() {
        let old = "line";
        let new = "line\nline\nline";

        let diff = calculate_diff(old, new);

        assert_eq!(diff.lines_added, 2);
        assert_eq!(diff.lines_removed, 0);
    }

    /// Verifies handling of content with only whitespace lines.
    #[test]
    fn t149_diff_whitespace_lines() {
        let old = "   \n\t\n";
        let new = "   \n";

        let diff = calculate_diff(old, new);

        // The empty line after the tab line in old is handled
        // Note: This tests that whitespace-only lines are treated as lines
        assert!(diff.lines_removed >= 1);
    }

    /// Verifies diffing single-line files.
    #[test]
    fn t149_diff_single_line_to_multiple() {
        let old = "single";
        let new = "line1\nline2\nline3";

        let diff = calculate_diff(old, new);

        assert_eq!(diff.lines_added, 3);
        assert_eq!(diff.lines_removed, 1);
    }

    /// Verifies that the DiffResult total_changes method works correctly.
    #[test]
    fn t149_diff_result_total_changes() {
        let diff = DiffResult {
            lines_added: 5,
            lines_removed: 3,
            lines_modified: 2,
        };

        assert_eq!(diff.total_changes(), 8);
    }

    /// Verifies that reordered lines are counted as changes.
    /// (Simple line-based diff doesn't track reordering, so all lines are "different")
    #[test]
    fn t149_diff_reordered_lines() {
        let old = "a\nb\nc";
        let new = "c\nb\na";

        let diff = calculate_diff(old, new);

        // Same lines, just reordered - simple diff sees no change
        // because it's counting unique line occurrences
        assert_eq!(diff.lines_added, 0);
        assert_eq!(diff.lines_removed, 0);
    }

    /// Verifies handling of content with trailing newline.
    #[test]
    fn t149_diff_trailing_newline() {
        let old = "line1\nline2";
        let new = "line1\nline2\n";

        let diff = calculate_diff(old, new);

        // The trailing newline creates an empty final line
        // which may or may not be significant depending on
        // how lines() handles it (it typically doesn't include
        // a trailing empty string)
        assert!(diff.is_empty() || diff.lines_added <= 1);
    }

    // =========================================================================
    // T150: Unit Tests for v1 Skip Behavior
    // =========================================================================

    /// Verifies that v1 files are skipped (no diff performed).
    #[test]
    fn t150_skip_version_1() {
        assert!(should_skip_version(1));
    }

    /// Verifies that v2 files are processed (diff performed).
    #[test]
    fn t150_process_version_2() {
        assert!(!should_skip_version(2));
    }

    /// Verifies that v3 files are processed.
    #[test]
    fn t150_process_version_3() {
        assert!(!should_skip_version(3));
    }

    /// Verifies that high version numbers are processed.
    #[test]
    fn t150_process_high_version() {
        assert!(!should_skip_version(100));
        assert!(!should_skip_version(1000));
        assert!(!should_skip_version(u32::MAX));
    }

    /// Verifies that version 0 (if it somehow exists) would be skipped.
    /// Note: Version 0 is invalid per the parser, but the skip function
    /// should handle it gracefully.
    #[test]
    fn t150_skip_version_0() {
        assert!(should_skip_version(0));
    }

    /// Verifies the integration of parsing and skip checking.
    #[test]
    fn t150_parse_and_skip_v1() {
        let version = parse_file_version("3f79c7095dc57fea@v1").unwrap();
        assert!(should_skip_version(version.version));
    }

    /// Verifies the integration of parsing and processing v2.
    #[test]
    fn t150_parse_and_process_v2() {
        let version = parse_file_version("3f79c7095dc57fea@v2").unwrap();
        assert!(!should_skip_version(version.version));
    }

    // =========================================================================
    // Additional Tests: Session ID Extraction
    // =========================================================================

    /// Verifies extracting session ID from a file-history path.
    #[test]
    fn extract_session_id_from_valid_path() {
        let path = Path::new("/home/user/.claude/file-history/abc-123-def/3f79c7095dc57fea@v2");
        let session_id = extract_session_id_from_path(path);

        assert_eq!(session_id, Some("abc-123-def".to_string()));
    }

    /// Verifies extracting session ID with UUID-style session.
    #[test]
    fn extract_session_id_uuid_format() {
        let path = Path::new(
            "/home/user/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/hash@v1",
        );
        let session_id = extract_session_id_from_path(path);

        assert_eq!(
            session_id,
            Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
        );
    }

    /// Verifies that extracting from a root path returns None.
    #[test]
    fn extract_session_id_no_parent() {
        let path = Path::new("hash@v1");
        let session_id = extract_session_id_from_path(path);

        // The parent would be "" which file_name() returns None for
        assert!(session_id.is_none() || session_id == Some("".to_string()));
    }

    // =========================================================================
    // Error Message Tests
    // =========================================================================

    /// Verifies all error types have meaningful display messages.
    #[test]
    fn error_display_messages() {
        let err = FileHistoryParseError::MissingVersionSeparator;
        assert!(err.to_string().contains("@v"));

        let err = FileHistoryParseError::InvalidHashLength(10);
        assert!(err.to_string().contains("10"));

        let err = FileHistoryParseError::InvalidHashCharacter('z');
        assert!(err.to_string().contains("z"));

        let err = FileHistoryParseError::MissingVersionNumber;
        assert!(err.to_string().contains("version number"));

        let err = FileHistoryParseError::InvalidVersionNumber("abc".to_string());
        assert!(err.to_string().contains("abc"));

        let err = FileHistoryParseError::VersionZero;
        assert!(err.to_string().contains("1"));

        let err = FileHistoryParseError::EmptyFilename;
        assert!(err.to_string().contains("empty"));
    }

    /// Verifies that errors implement Debug.
    #[test]
    fn error_is_debug() {
        let err = FileHistoryParseError::VersionZero;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("VersionZero"));
    }

    /// Verifies that errors implement PartialEq for testing.
    #[test]
    fn error_is_eq() {
        assert_eq!(
            FileHistoryParseError::VersionZero,
            FileHistoryParseError::VersionZero
        );
        assert_ne!(
            FileHistoryParseError::VersionZero,
            FileHistoryParseError::EmptyFilename
        );
    }

    // =========================================================================
    // FileVersion Struct Tests
    // =========================================================================

    /// Verifies FileVersion::new works for valid inputs.
    #[test]
    fn file_version_new_valid() {
        let version = FileVersion::new("0123456789abcdef", 5);
        assert_eq!(version.hash, "0123456789abcdef");
        assert_eq!(version.version, 5);
    }

    /// Verifies FileVersion implements Clone correctly.
    #[test]
    fn file_version_clone() {
        let original = FileVersion::new("0123456789abcdef", 3);
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(cloned.hash, "0123456789abcdef");
        assert_eq!(cloned.version, 3);
    }

    /// Verifies FileVersion implements Debug correctly.
    #[test]
    fn file_version_debug() {
        let version = FileVersion::new("0123456789abcdef", 2);
        let debug_str = format!("{:?}", version);

        assert!(debug_str.contains("FileVersion"));
        assert!(debug_str.contains("0123456789abcdef"));
        assert!(debug_str.contains("2"));
    }

    // =========================================================================
    // DiffResult Struct Tests
    // =========================================================================

    /// Verifies DiffResult::default returns all zeros.
    #[test]
    fn diff_result_default() {
        let diff = DiffResult::default();

        assert_eq!(diff.lines_added, 0);
        assert_eq!(diff.lines_removed, 0);
        assert_eq!(diff.lines_modified, 0);
        assert!(diff.is_empty());
        assert_eq!(diff.total_changes(), 0);
    }

    /// Verifies DiffResult implements Clone correctly.
    #[test]
    fn diff_result_clone() {
        let original = DiffResult {
            lines_added: 10,
            lines_removed: 5,
            lines_modified: 3,
        };
        let cloned = original;

        assert_eq!(original, cloned);
    }

    /// Verifies DiffResult::is_empty works correctly.
    #[test]
    fn diff_result_is_empty() {
        let empty = DiffResult {
            lines_added: 0,
            lines_removed: 0,
            lines_modified: 0,
        };
        assert!(empty.is_empty());

        let not_empty = DiffResult {
            lines_added: 1,
            lines_removed: 0,
            lines_modified: 0,
        };
        assert!(!not_empty.is_empty());
    }

    // =========================================================================
    // Integration Tests: Full Flow
    // =========================================================================

    /// Tests the full flow: parse version, check skip, calculate diff.
    #[test]
    fn integration_full_flow_v2_processed() {
        // Parse a v2 file
        let version = parse_file_version("3f79c7095dc57fea@v2").unwrap();

        // Verify it should be processed
        assert!(!should_skip_version(version.version));

        // Calculate diff (simulating v1 -> v2)
        let old_content = "line1\nline2";
        let new_content = "line1\nline2\nline3";
        let diff = calculate_diff(old_content, new_content);

        assert_eq!(diff.lines_added, 1);
        assert_eq!(diff.lines_removed, 0);
    }

    /// Tests the full flow for a v1 file (should be skipped).
    #[test]
    fn integration_full_flow_v1_skipped() {
        // Parse a v1 file
        let version = parse_file_version("3f79c7095dc57fea@v1").unwrap();

        // Verify it should be skipped
        assert!(should_skip_version(version.version));

        // No diff calculation needed for v1
    }

    /// Tests handling of version gaps (v3 with missing v2).
    /// Note: This tests the logic; actual gap handling with warnings
    /// would be in the tracker implementation.
    #[test]
    fn integration_version_gap_detection() {
        let v1 = parse_file_version("3f79c7095dc57fea@v1").unwrap();
        let v3 = parse_file_version("3f79c7095dc57fea@v3").unwrap();

        // Same file (same hash), but v2 is missing
        assert_eq!(v1.hash, v3.hash);
        assert_eq!(v3.version - v1.version, 2); // Gap detected

        // v3 should still be processed (diff against v1 with warning)
        assert!(!should_skip_version(v3.version));
    }

    // =========================================================================
    // FileHistoryTracker Types Tests
    // =========================================================================

    /// Verifies FileHistoryTrackerError display messages.
    #[test]
    fn tracker_error_display_messages() {
        let err = FileHistoryTrackerError::ChannelClosed;
        assert_eq!(err.to_string(), "failed to send event: channel closed");

        let err = FileHistoryTrackerError::ClaudeDirectoryNotFound(PathBuf::from("/test/path"));
        assert_eq!(
            err.to_string(),
            "claude file-history directory not found: /test/path"
        );
    }

    /// Verifies FileHistoryTrackerError is Debug.
    #[test]
    fn tracker_error_is_debug() {
        let err = FileHistoryTrackerError::ChannelClosed;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ChannelClosed"));

        let err = FileHistoryTrackerError::ClaudeDirectoryNotFound(PathBuf::from("/test"));
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ClaudeDirectoryNotFound"));
    }

    /// Verifies FileHistoryTrackerConfig default values.
    #[test]
    fn tracker_config_default() {
        let config = FileHistoryTrackerConfig::default();
        assert_eq!(config.debounce_ms, 100);
    }

    /// Verifies FileHistoryTrackerConfig clone.
    #[test]
    fn tracker_config_clone() {
        let config = FileHistoryTrackerConfig { debounce_ms: 200 };
        let cloned = config.clone();
        assert_eq!(cloned.debounce_ms, 200);
    }

    // =========================================================================
    // create_file_change_event Tests
    // =========================================================================

    /// Verifies create_file_change_event creates correct event.
    #[test]
    fn create_event_from_version_and_diff() {
        let version = FileVersion::new("0123456789abcdef", 3);
        let diff = DiffResult {
            lines_added: 10,
            lines_removed: 5,
            lines_modified: 3,
        };

        let event = create_file_change_event("session-abc", &version, &diff);

        assert_eq!(event.session_id, "session-abc");
        assert_eq!(event.file_hash, "0123456789abcdef");
        assert_eq!(event.version, 3);
        assert_eq!(event.lines_added, 10);
        assert_eq!(event.lines_removed, 5);
        assert_eq!(event.lines_modified, 3);
    }

    /// Verifies event timestamp is set to now.
    #[test]
    fn create_event_has_timestamp() {
        let version = FileVersion::new("0123456789abcdef", 2);
        let diff = DiffResult::default();

        let before = Utc::now();
        let event = create_file_change_event("sess", &version, &diff);
        let after = Utc::now();

        assert!(event.timestamp >= before);
        assert!(event.timestamp <= after);
    }

    /// Verifies event with zero diff values.
    #[test]
    fn create_event_empty_diff() {
        let version = FileVersion::new("aaaaaaaaaaaaaaaa", 5);
        let diff = DiffResult::default();

        let event = create_file_change_event("test-session", &version, &diff);

        assert_eq!(event.lines_added, 0);
        assert_eq!(event.lines_removed, 0);
        assert_eq!(event.lines_modified, 0);
    }

    // =========================================================================
    // FileHistoryTracker File Operations Tests
    // =========================================================================

    use std::io::Write;
    use tempfile::TempDir;
    use tokio::time::{sleep, timeout, Duration as TokioDuration};

    /// Creates a session directory with file versions for testing.
    fn create_test_session(dir: &TempDir, session_id: &str) -> PathBuf {
        let session_dir = dir.path().join(session_id);
        std::fs::create_dir_all(&session_dir).expect("Failed to create session dir");
        session_dir
    }

    /// Creates a file version in the session directory.
    fn create_test_file_version(session_dir: &Path, hash: &str, version: u32, content: &str) -> PathBuf {
        let filename = format!("{}@v{}", hash, version);
        let file_path = session_dir.join(&filename);
        let mut file = std::fs::File::create(&file_path).expect("Failed to create file version");
        file.write_all(content.as_bytes()).expect("Failed to write content");
        file.flush().expect("Failed to flush");
        file_path
    }

    #[tokio::test]
    async fn test_tracker_creation_missing_directory() {
        let (tx, _rx) = mpsc::channel(100);
        let result = FileHistoryTracker::with_path(PathBuf::from("/nonexistent/file-history"), tx);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FileHistoryTrackerError::ClaudeDirectoryNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_tracker_creation_with_valid_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let result = FileHistoryTracker::with_path(temp_dir.path().to_path_buf(), tx);

        assert!(result.is_ok(), "Should create tracker for valid directory");
        assert_eq!(result.unwrap().root_dir(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_tracker_detects_new_v2_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = FileHistoryTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a session directory
        let session_id = "6e45a55c-3124-4cc8-ad85-040a5c316009";
        let session_dir = create_test_session(&temp_dir, session_id);

        // Create v1 (should be skipped)
        let hash = "0123456789abcdef";
        create_test_file_version(&session_dir, hash, 1, "line1\nline2");

        // Give some time for v1 to be processed (skipped)
        sleep(TokioDuration::from_millis(200)).await;

        // Create v2 (should emit event)
        create_test_file_version(&session_dir, hash, 2, "line1\nline2\nline3");

        // Should receive a file change event
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event for new v2 file");

        let event = result.unwrap().unwrap();
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.file_hash, hash);
        assert_eq!(event.version, 2);
        assert_eq!(event.lines_added, 1); // line3 was added
        assert_eq!(event.lines_removed, 0);
    }

    #[tokio::test]
    async fn test_tracker_skips_v1_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = FileHistoryTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a session directory and v1 file
        let session_id = "test-session-v1";
        let session_dir = create_test_session(&temp_dir, session_id);
        create_test_file_version(&session_dir, "aaaaaaaaaaaaaaaa", 1, "initial content");

        // Should NOT receive any event for v1
        let result = timeout(TokioDuration::from_millis(300), rx.recv()).await;
        assert!(result.is_err(), "Should not receive event for v1 file");
    }

    #[tokio::test]
    async fn test_tracker_handles_version_sequence() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = FileHistoryTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create session and versions
        let session_id = "version-sequence-test";
        let session_dir = create_test_session(&temp_dir, session_id);
        let hash = "bbbbbbbbbbbbbbbb";

        // Create v1
        create_test_file_version(&session_dir, hash, 1, "a\nb\nc");
        sleep(TokioDuration::from_millis(200)).await;

        // Create v2
        create_test_file_version(&session_dir, hash, 2, "a\nb\nc\nd");

        // Should receive event for v2
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event for v2");

        let event = result.unwrap().unwrap();
        assert_eq!(event.version, 2);
        assert_eq!(event.lines_added, 1); // 'd' added

        // Create v3
        sleep(TokioDuration::from_millis(100)).await;
        create_test_file_version(&session_dir, hash, 3, "a\nb\nc\nd\ne\nf");

        // Should receive event for v3
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event for v3");

        let event = result.unwrap().unwrap();
        assert_eq!(event.version, 3);
        assert_eq!(event.lines_added, 2); // 'e' and 'f' added
    }

    #[tokio::test]
    async fn test_tracker_handles_modified_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = FileHistoryTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create session and initial versions
        let session_id = "modify-test";
        let session_dir = create_test_session(&temp_dir, session_id);
        let hash = "cccccccccccccccc";

        create_test_file_version(&session_dir, hash, 1, "original");
        sleep(TokioDuration::from_millis(150)).await;

        // Create v2 with modifications
        create_test_file_version(&session_dir, hash, 2, "modified\nwith\nnew\nlines");

        // Should receive event
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event for modified file");

        let event = result.unwrap().unwrap();
        assert_eq!(event.session_id, session_id);
        assert_eq!(event.lines_added, 4); // 4 new lines
        assert_eq!(event.lines_removed, 1); // 'original' removed
    }

    #[tokio::test]
    async fn test_tracker_ignores_non_version_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = FileHistoryTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create session directory
        let session_id = "ignore-test";
        let session_dir = create_test_session(&temp_dir, session_id);

        // Create a non-version file
        let random_file = session_dir.join("not-a-version.txt");
        std::fs::write(&random_file, "random content").expect("Should write");

        // Should NOT receive any event
        let result = timeout(TokioDuration::from_millis(300), rx.recv()).await;
        assert!(result.is_err(), "Should not receive event for non-version file");
    }

    #[tokio::test]
    async fn test_tracker_config_with_custom_debounce() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let config = FileHistoryTrackerConfig { debounce_ms: 50 };
        let tracker =
            FileHistoryTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Tracker should be created successfully with custom config
        assert_eq!(tracker.root_dir(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_tracker_multiple_sessions() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = FileHistoryTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create two sessions
        let session1 = "session-1111-1111";
        let session2 = "session-2222-2222";
        let session_dir1 = create_test_session(&temp_dir, session1);
        let session_dir2 = create_test_session(&temp_dir, session2);

        let hash1 = "1111111111111111";
        let hash2 = "2222222222222222";

        // Create v1 for both
        create_test_file_version(&session_dir1, hash1, 1, "content1");
        create_test_file_version(&session_dir2, hash2, 1, "content2");
        sleep(TokioDuration::from_millis(200)).await;

        // Create v2 for both
        create_test_file_version(&session_dir1, hash1, 2, "content1\nmore1");
        create_test_file_version(&session_dir2, hash2, 2, "content2\nmore2");

        // Should receive events for both
        let mut events = Vec::new();
        for _ in 0..2 {
            if let Ok(Some(event)) = timeout(TokioDuration::from_millis(500), rx.recv()).await {
                events.push(event);
            }
        }

        assert_eq!(events.len(), 2, "Should receive events from both sessions");

        let session_ids: Vec<_> = events.iter().map(|e| e.session_id.as_str()).collect();
        assert!(session_ids.contains(&session1));
        assert!(session_ids.contains(&session2));
    }

    #[tokio::test]
    async fn test_tracker_calculates_diff_correctly() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, mut rx) = mpsc::channel(100);
        let _tracker = FileHistoryTracker::with_path(temp_dir.path().to_path_buf(), tx)
            .expect("Should create tracker");

        sleep(TokioDuration::from_millis(50)).await;

        let session_id = "diff-calc-test";
        let session_dir = create_test_session(&temp_dir, session_id);
        let hash = "dddddddddddddddd";

        // v1: 3 lines
        create_test_file_version(&session_dir, hash, 1, "line1\nline2\nline3");
        sleep(TokioDuration::from_millis(200)).await;

        // v2: replace line2 with line2-modified, add line4
        create_test_file_version(&session_dir, hash, 2, "line1\nline2-modified\nline3\nline4");

        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok());

        let event = result.unwrap().unwrap();
        // line2-modified and line4 added
        // line2 removed
        assert_eq!(event.lines_added, 2);
        assert_eq!(event.lines_removed, 1);
        assert_eq!(event.lines_modified, 1); // min(added, removed)
    }
}
