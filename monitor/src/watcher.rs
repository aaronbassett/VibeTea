//! File watcher for monitoring Claude Code session files.
//!
//! This module provides functionality to watch `~/.claude/projects/**/*.jsonl` files
//! for changes and emit events when files are created, modified, or removed.
//!
//! # Architecture
//!
//! The watcher uses the [`notify`] crate to monitor file system events and maintains
//! a position map to track the last-read byte offset for each JSONL file. This allows
//! efficient tailing of files without re-reading content that has already been processed.
//!
//! The notify callback is kept lightweight by sending raw events through an internal
//! channel to a dedicated async task, which handles all file I/O and lock acquisition.
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use tokio::sync::mpsc;
//! use directories::BaseDirs;
//! use vibetea_monitor::watcher::{FileWatcher, WatchEvent};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let base_dirs = BaseDirs::new().expect("home directory");
//!     let watch_dir = base_dirs.home_dir().join(".claude/projects");
//!
//!     let (tx, mut rx) = mpsc::channel(100);
//!     let watcher = FileWatcher::new(watch_dir, tx)?;
//!
//!     while let Some(event) = rx.recv().await {
//!         match event {
//!             WatchEvent::FileCreated(path) => println!("New file: {:?}", path),
//!             WatchEvent::LinesAdded { path, lines } => {
//!                 println!("New lines in {:?}: {}", path, lines.len());
//!             }
//!             WatchEvent::FileRemoved(path) => println!("Removed: {:?}", path),
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use notify::{
    event::{CreateKind, ModifyKind, RemoveKind},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, trace, warn};

/// Events emitted by the file watcher.
///
/// These events represent significant file system changes relevant to monitoring
/// JSONL session files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    /// A new JSONL file was created.
    ///
    /// The watcher automatically begins tracking this file for future modifications.
    FileCreated(PathBuf),

    /// New lines were appended to a JSONL file.
    ///
    /// Contains the path to the file and the newly added lines (excluding any
    /// previously read content).
    LinesAdded {
        /// Path to the modified file.
        path: PathBuf,
        /// New lines that were appended since the last read.
        lines: Vec<String>,
    },

    /// A JSONL file was removed.
    ///
    /// The watcher automatically stops tracking this file and cleans up its
    /// position state.
    FileRemoved(PathBuf),
}

/// Internal events from the notify callback, processed by the async task.
#[derive(Debug)]
enum InternalEvent {
    FileCreated(PathBuf),
    FileModified(PathBuf),
    FileRemoved(PathBuf),
}

/// Errors that can occur during file watching operations.
#[derive(Error, Debug)]
pub enum WatcherError {
    /// Failed to initialize the file system watcher.
    #[error("failed to create watcher: {0}")]
    WatcherInit(#[from] notify::Error),

    /// Failed to read or process a file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The watch directory does not exist or is inaccessible.
    #[error("watch directory does not exist: {0}")]
    DirectoryNotFound(PathBuf),

    /// Failed to send event through the channel.
    #[error("failed to send event: channel closed")]
    ChannelClosed,
}

/// Result type for watcher operations.
pub type Result<T> = std::result::Result<T, WatcherError>;

/// File watcher for monitoring JSONL session files.
///
/// Watches a directory tree for `.jsonl` files and emits events when files are
/// created, modified, or removed. Maintains position tracking to enable efficient
/// tailing without re-reading already-processed content.
///
/// # Thread Safety
///
/// The watcher is designed to be used with async Tokio code. The position map is
/// protected by an `RwLock` to allow concurrent reads while ensuring exclusive
/// access during updates.
#[derive(Debug)]
pub struct FileWatcher {
    /// The underlying file system watcher.
    ///
    /// Kept alive to maintain the watch subscription. Dropping this will stop
    /// watching for events.
    #[allow(dead_code)]
    watcher: RecommendedWatcher,

    /// Byte positions for each tracked file.
    ///
    /// Used to track where we last read from, enabling efficient tailing.
    positions: Arc<RwLock<HashMap<PathBuf, u64>>>,

    /// The root directory being watched.
    watch_dir: PathBuf,

    /// Channel sender for emitting watch events.
    #[allow(dead_code)]
    event_sender: mpsc::Sender<WatchEvent>,
}

impl FileWatcher {
    /// Creates a new file watcher for the specified directory.
    ///
    /// On creation, the watcher:
    /// 1. Scans the watch directory for existing `.jsonl` files
    /// 2. Seeks to the end of each file (does not replay old events)
    /// 3. Begins monitoring for new file system events
    ///
    /// # Arguments
    ///
    /// * `watch_dir` - The root directory to watch (e.g., `~/.claude/projects`)
    /// * `event_sender` - Channel for emitting [`WatchEvent`]s
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The watch directory does not exist
    /// - The file system watcher cannot be initialized
    /// - Initial file scanning fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use tokio::sync::mpsc;
    /// use vibetea_monitor::watcher::FileWatcher;
    ///
    /// let (tx, rx) = mpsc::channel(100);
    /// let watcher = FileWatcher::new(PathBuf::from("/home/user/.claude/projects"), tx)?;
    /// # Ok::<(), vibetea_monitor::watcher::WatcherError>(())
    /// ```
    pub fn new(watch_dir: PathBuf, event_sender: mpsc::Sender<WatchEvent>) -> Result<Self> {
        // Verify the watch directory exists
        if !watch_dir.exists() {
            return Err(WatcherError::DirectoryNotFound(watch_dir));
        }

        let positions = Arc::new(RwLock::new(HashMap::new()));

        // Scan existing files and seek to end
        let initial_positions = scan_existing_files(&watch_dir)?;
        {
            // Initialize positions synchronously before async context
            let positions_ref = Arc::clone(&positions);
            // Use std blocking for initial setup
            let mut guard = futures::executor::block_on(positions_ref.write());
            *guard = initial_positions;
        }

        info!(
            watch_dir = %watch_dir.display(),
            "Initialized file watcher"
        );

        // Create internal channel for notify events
        // This channel bridges the sync notify callback to our async processing task
        let (internal_tx, internal_rx) = mpsc::channel::<InternalEvent>(1000);

        // Spawn the async processing task
        let positions_for_task = Arc::clone(&positions);
        let sender_for_task = event_sender.clone();
        tokio::spawn(async move {
            process_internal_events(internal_rx, positions_for_task, sender_for_task).await;
        });

        // Create the notify watcher with lightweight callback
        let watcher = create_watcher(internal_tx, watch_dir.clone())?;

        Ok(Self {
            watcher,
            positions,
            watch_dir,
            event_sender,
        })
    }

    /// Returns the directory being watched.
    #[must_use]
    pub fn watch_dir(&self) -> &Path {
        &self.watch_dir
    }

    /// Returns the current number of tracked files.
    ///
    /// This is an async operation that acquires a read lock on the position map.
    pub async fn tracked_file_count(&self) -> usize {
        self.positions.read().await.len()
    }

    /// Returns the current byte position for a specific file.
    ///
    /// Returns `None` if the file is not being tracked.
    pub async fn file_position(&self, path: &Path) -> Option<u64> {
        self.positions.read().await.get(path).copied()
    }

    /// Manually triggers a check for new content in a specific file.
    ///
    /// This is useful when you want to poll for changes rather than relying
    /// solely on file system events (which can sometimes be missed).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the event channel is closed.
    pub async fn check_file(&self, path: &Path) -> Result<()> {
        if path.extension().is_none_or(|ext| ext != "jsonl") {
            return Ok(());
        }

        let mut positions = self.positions.write().await;
        let lines = read_new_lines(path, &mut positions)?;

        if !lines.is_empty() {
            self.event_sender
                .send(WatchEvent::LinesAdded {
                    path: path.to_path_buf(),
                    lines,
                })
                .await
                .map_err(|_| WatcherError::ChannelClosed)?;
        }

        Ok(())
    }
}

/// Creates the underlying notify watcher with a lightweight callback.
///
/// The callback only sends events through the internal channel; all heavy
/// processing is done by the async task.
fn create_watcher(
    internal_tx: mpsc::Sender<InternalEvent>,
    watch_dir: PathBuf,
) -> Result<RecommendedWatcher> {
    let mut watcher = RecommendedWatcher::new(
        move |res: std::result::Result<Event, notify::Error>| {
            handle_notify_event(res, &internal_tx);
        },
        Config::default(),
    )?;

    // Start watching recursively
    watcher.watch(&watch_dir, RecursiveMode::Recursive)?;

    debug!(
        watch_dir = %watch_dir.display(),
        "Started recursive file watch"
    );

    Ok(watcher)
}

/// Handles events from the notify crate.
///
/// This callback is kept extremely lightweight - it only filters events and
/// sends them through a channel. All file I/O and locking is done by the
/// async processing task.
fn handle_notify_event(
    res: std::result::Result<Event, notify::Error>,
    internal_tx: &mpsc::Sender<InternalEvent>,
) {
    let event = match res {
        Ok(event) => event,
        Err(e) => {
            error!(error = %e, "File watcher error");
            return;
        }
    };

    trace!(kind = ?event.kind, paths = ?event.paths, "Received notify event");

    // Process each path in the event
    for path in &event.paths {
        // Only process .jsonl files
        if path.extension().is_none_or(|ext| ext != "jsonl") {
            continue;
        }

        let internal_event = match event.kind {
            EventKind::Create(CreateKind::File) | EventKind::Create(CreateKind::Any) => {
                Some(InternalEvent::FileCreated(path.clone()))
            }
            EventKind::Modify(ModifyKind::Data(_)) | EventKind::Modify(ModifyKind::Any) => {
                Some(InternalEvent::FileModified(path.clone()))
            }
            EventKind::Remove(RemoveKind::File) | EventKind::Remove(RemoveKind::Any) => {
                Some(InternalEvent::FileRemoved(path.clone()))
            }
            _ => {
                trace!(kind = ?event.kind, path = %path.display(), "Ignoring event kind");
                None
            }
        };

        if let Some(evt) = internal_event {
            // Use try_send to avoid blocking the notify thread
            // If the channel is full, we'll miss some events, but that's
            // preferable to blocking the file system watcher
            if let Err(e) = internal_tx.try_send(evt) {
                warn!(error = %e, "Failed to queue internal event, channel may be full");
            }
        }
    }
}

/// Async task that processes internal events.
///
/// This centralizes all async operations (file I/O, lock acquisition) into
/// a single managed task, avoiding the need to spawn threads or block on
/// the runtime from the notify callback.
async fn process_internal_events(
    mut rx: mpsc::Receiver<InternalEvent>,
    positions: Arc<RwLock<HashMap<PathBuf, u64>>>,
    sender: mpsc::Sender<WatchEvent>,
) {
    while let Some(event) = rx.recv().await {
        match event {
            InternalEvent::FileCreated(path) => {
                handle_file_created_async(&path, &positions, &sender).await;
            }
            InternalEvent::FileModified(path) => {
                handle_file_modified_async(&path, &positions, &sender).await;
            }
            InternalEvent::FileRemoved(path) => {
                handle_file_removed_async(&path, &positions, &sender).await;
            }
        }
    }

    debug!("Internal event processor shutting down");
}

/// Handles a file creation event asynchronously.
async fn handle_file_created_async(
    path: &Path,
    positions: &Arc<RwLock<HashMap<PathBuf, u64>>>,
    sender: &mpsc::Sender<WatchEvent>,
) {
    info!(path = %path.display(), "New JSONL file detected");

    // Get file size and initialize position at end
    let size = match fs::metadata(path) {
        Ok(meta) => meta.len(),
        Err(e) => {
            warn!(path = %path.display(), error = %e, "Failed to get file metadata");
            0
        }
    };

    // Update position
    {
        let mut guard = positions.write().await;
        guard.insert(path.to_path_buf(), size);
    }

    // Send event
    if let Err(e) = sender.send(WatchEvent::FileCreated(path.to_path_buf())).await {
        error!(error = %e, "Failed to send FileCreated event");
    }
}

/// Handles a file modification event asynchronously.
async fn handle_file_modified_async(
    path: &Path,
    positions: &Arc<RwLock<HashMap<PathBuf, u64>>>,
    sender: &mpsc::Sender<WatchEvent>,
) {
    debug!(path = %path.display(), "File modification detected");

    let lines = {
        let mut guard = positions.write().await;
        match read_new_lines(path, &mut guard) {
            Ok(lines) => lines,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to read new lines");
                return;
            }
        }
    };

    if !lines.is_empty() {
        debug!(
            path = %path.display(),
            line_count = lines.len(),
            "Read new lines from file"
        );

        if let Err(e) = sender
            .send(WatchEvent::LinesAdded {
                path: path.to_path_buf(),
                lines,
            })
            .await
        {
            error!(error = %e, "Failed to send LinesAdded event");
        }
    } else {
        trace!(path = %path.display(), "No new lines to read");
    }
}

/// Handles a file removal event asynchronously.
async fn handle_file_removed_async(
    path: &Path,
    positions: &Arc<RwLock<HashMap<PathBuf, u64>>>,
    sender: &mpsc::Sender<WatchEvent>,
) {
    info!(path = %path.display(), "JSONL file removed");

    // Clean up position tracking
    {
        let mut guard = positions.write().await;
        guard.remove(path);
    }

    // Send event
    if let Err(e) = sender.send(WatchEvent::FileRemoved(path.to_path_buf())).await {
        error!(error = %e, "Failed to send FileRemoved event");
    }
}

/// Scans a directory tree for existing `.jsonl` files and returns their sizes.
///
/// Files are seeked to end on startup, meaning existing content is not replayed.
fn scan_existing_files(dir: &Path) -> Result<HashMap<PathBuf, u64>> {
    let mut positions = HashMap::new();

    if !dir.exists() {
        return Ok(positions);
    }

    scan_directory_recursive(dir, &mut positions)?;

    info!(file_count = positions.len(), "Scanned existing JSONL files");

    Ok(positions)
}

/// Recursively scans a directory for `.jsonl` files.
fn scan_directory_recursive(dir: &Path, positions: &mut HashMap<PathBuf, u64>) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            warn!(dir = %dir.display(), "Permission denied, skipping directory");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_directory_recursive(&path, positions)?;
        } else if path.extension().is_some_and(|ext| ext == "jsonl") {
            match fs::metadata(&path) {
                Ok(meta) => {
                    let size = meta.len();
                    debug!(
                        path = %path.display(),
                        size = size,
                        "Found existing JSONL file"
                    );
                    positions.insert(path, size);
                }
                Err(e) => {
                    warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to get metadata for existing file"
                    );
                }
            }
        }
    }

    Ok(())
}

/// Reads new lines from a file starting at the tracked position.
///
/// Handles file truncation by resetting to position 0 if the file is smaller
/// than the last known position.
fn read_new_lines(path: &Path, positions: &mut HashMap<PathBuf, u64>) -> Result<Vec<String>> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();

    // Get last position, defaulting to 0 for new files
    let last_position = positions.get(path).copied().unwrap_or(0);

    // Handle truncation: if file is smaller than last position, reset
    let read_position = if file_size < last_position {
        info!(
            path = %path.display(),
            old_pos = last_position,
            new_size = file_size,
            "File truncated, resetting position to 0"
        );
        0
    } else {
        last_position
    };

    // Nothing to read if we're at EOF, but still update position for new files
    if read_position >= file_size {
        positions.insert(path.to_path_buf(), file_size);
        return Ok(Vec::new());
    }

    // Seek to read position
    file.seek(SeekFrom::Start(read_position))?;

    let mut lines = Vec::new();
    let mut reader = BufReader::new(&file);

    // Read lines, handling partial lines at the end
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                // Only include complete lines (ending with newline)
                if line.ends_with('\n') {
                    let trimmed = line.trim_end_matches(&['\n', '\r'][..]).to_string();
                    if !trimmed.is_empty() {
                        lines.push(trimmed);
                    }
                }
            }
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Error reading line");
                break;
            }
        }
    }

    // Update position to current file size
    positions.insert(path.to_path_buf(), file_size);

    Ok(lines)
}

/// Reads new lines from a file synchronously, used for testing and non-async contexts.
#[cfg(test)]
fn read_new_lines_sync(path: &Path, positions: &mut HashMap<PathBuf, u64>) -> Result<Vec<String>> {
    read_new_lines(path, positions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Creates a temporary directory with a test structure.
    fn create_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    /// Creates a JSONL file with initial content.
    fn create_jsonl_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }
        let mut file = File::create(&path).expect("Failed to create file");
        file.write_all(content.as_bytes())
            .expect("Failed to write content");
        path
    }

    #[test]
    fn test_position_tracking_initial_state() {
        let temp_dir = create_test_dir();
        let mut positions = HashMap::new();

        // Create a file with content
        let content = r#"{"line":1}
{"line":2}
"#;
        let path = create_jsonl_file(temp_dir.path(), "test.jsonl", content);

        // First read should read nothing (we simulate starting at end)
        positions.insert(path.clone(), content.len() as u64);
        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert!(
            lines.is_empty(),
            "Should not read existing content on startup"
        );
    }

    #[test]
    fn test_position_tracking_new_content() {
        let temp_dir = create_test_dir();
        let mut positions = HashMap::new();

        // Create initial file
        let initial_content = r#"{"line":1}
"#;
        let path = create_jsonl_file(temp_dir.path(), "test.jsonl", initial_content);
        positions.insert(path.clone(), initial_content.len() as u64);

        // Append new content
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("Failed to open file");
        writeln!(file, r#"{{"line":2}}"#).expect("Failed to append");
        writeln!(file, r#"{{"line":3}}"#).expect("Failed to append");

        // Read should return only new lines
        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], r#"{"line":2}"#);
        assert_eq!(lines[1], r#"{"line":3}"#);
    }

    #[test]
    fn test_position_tracking_empty_file() {
        let temp_dir = create_test_dir();
        let mut positions = HashMap::new();

        let path = create_jsonl_file(temp_dir.path(), "empty.jsonl", "");

        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert!(lines.is_empty());
        assert_eq!(positions.get(&path), Some(&0));
    }

    #[test]
    fn test_truncation_handling() {
        let temp_dir = create_test_dir();
        let mut positions = HashMap::new();

        // Create file with content
        let initial_content = r#"{"line":1}
{"line":2}
{"line":3}
"#;
        let path = create_jsonl_file(temp_dir.path(), "test.jsonl", initial_content);
        positions.insert(path.clone(), initial_content.len() as u64);

        // Truncate the file (simulates log rotation or similar)
        let truncated_content = r#"{"new":1}
"#;
        fs::write(&path, truncated_content).expect("Failed to truncate");

        // Read should detect truncation and read from beginning
        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], r#"{"new":1}"#);
    }

    #[test]
    fn test_partial_line_handling() {
        let temp_dir = create_test_dir();
        let mut positions = HashMap::new();

        // Create file without trailing newline
        let content = r#"{"line":1}
{"line":2}"#; // Note: no newline at end
        let path = create_jsonl_file(temp_dir.path(), "test.jsonl", content);

        // Read should only return complete lines
        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], r#"{"line":1}"#);

        // Position should be at file end
        assert_eq!(positions.get(&path), Some(&(content.len() as u64)));

        // Complete the partial line
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("Failed to open");
        writeln!(file).expect("Failed to write newline");

        // Now the second line should be readable
        // Reset position to just after first line
        positions.insert(path.clone(), 11); // After "{"line":1}\n"
        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], r#"{"line":2}"#);
    }

    #[test]
    fn test_scan_existing_files() {
        let temp_dir = create_test_dir();

        // Create nested structure
        create_jsonl_file(
            temp_dir.path(),
            "project1/session.jsonl",
            r#"{"a":1}
"#,
        );
        create_jsonl_file(
            temp_dir.path(),
            "project2/subdir/events.jsonl",
            r#"{"b":1}
{"b":2}
"#,
        );
        create_jsonl_file(temp_dir.path(), "root.jsonl", "");

        // Create non-jsonl files that should be ignored
        create_jsonl_file(temp_dir.path(), "readme.md", "# Test");
        create_jsonl_file(temp_dir.path(), "config.json", "{}");

        let positions = scan_existing_files(temp_dir.path()).unwrap();

        assert_eq!(positions.len(), 3, "Should find exactly 3 JSONL files");

        // Verify positions are at end of files
        for (path, pos) in &positions {
            let meta = fs::metadata(path).unwrap();
            assert_eq!(
                *pos,
                meta.len(),
                "Position should be at end of file for {:?}",
                path
            );
        }
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let positions = scan_existing_files(Path::new("/nonexistent/path")).unwrap();
        assert!(positions.is_empty());
    }

    #[test]
    fn test_file_removal_cleanup() {
        let temp_dir = create_test_dir();
        let mut positions = HashMap::new();

        // Create and track a file
        let path = create_jsonl_file(
            temp_dir.path(),
            "test.jsonl",
            r#"{"line":1}
"#,
        );
        positions.insert(path.clone(), 12);

        // Verify it's tracked
        assert!(positions.contains_key(&path));

        // Simulate removal cleanup
        positions.remove(&path);

        // Verify it's no longer tracked
        assert!(!positions.contains_key(&path));
    }

    #[test]
    fn test_watch_event_equality() {
        let path1 = PathBuf::from("/test/file1.jsonl");
        let path2 = PathBuf::from("/test/file2.jsonl");

        // FileCreated equality
        assert_eq!(
            WatchEvent::FileCreated(path1.clone()),
            WatchEvent::FileCreated(path1.clone())
        );
        assert_ne!(
            WatchEvent::FileCreated(path1.clone()),
            WatchEvent::FileCreated(path2.clone())
        );

        // LinesAdded equality
        assert_eq!(
            WatchEvent::LinesAdded {
                path: path1.clone(),
                lines: vec!["test".to_string()]
            },
            WatchEvent::LinesAdded {
                path: path1.clone(),
                lines: vec!["test".to_string()]
            }
        );

        // FileRemoved equality
        assert_eq!(
            WatchEvent::FileRemoved(path1.clone()),
            WatchEvent::FileRemoved(path1.clone())
        );
    }

    #[test]
    fn test_multiple_appends() {
        let temp_dir = create_test_dir();
        let mut positions = HashMap::new();

        // Start with empty file
        let path = create_jsonl_file(temp_dir.path(), "test.jsonl", "");

        // First append
        {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .expect("Failed to open");
            writeln!(file, r#"{{"batch":1}}"#).expect("Failed to append");
        }

        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], r#"{"batch":1}"#);

        // Second append
        {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .expect("Failed to open");
            writeln!(file, r#"{{"batch":2}}"#).expect("Failed to append");
            writeln!(file, r#"{{"batch":3}}"#).expect("Failed to append");
        }

        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], r#"{"batch":2}"#);
        assert_eq!(lines[1], r#"{"batch":3}"#);
    }

    #[test]
    fn test_crlf_line_endings() {
        let temp_dir = create_test_dir();
        let mut positions = HashMap::new();

        // Create file with Windows line endings
        let content = "{\"line\":1}\r\n{\"line\":2}\r\n";
        let path = create_jsonl_file(temp_dir.path(), "windows.jsonl", content);

        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], r#"{"line":1}"#);
        assert_eq!(lines[1], r#"{"line":2}"#);
    }

    #[test]
    fn test_empty_lines_filtered() {
        let temp_dir = create_test_dir();
        let mut positions = HashMap::new();

        // Create file with empty lines
        let content = r#"{"line":1}

{"line":2}

"#;
        let path = create_jsonl_file(temp_dir.path(), "test.jsonl", content);

        let lines = read_new_lines_sync(&path, &mut positions).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], r#"{"line":1}"#);
        assert_eq!(lines[1], r#"{"line":2}"#);
    }

    #[tokio::test]
    async fn test_file_watcher_directory_not_found() {
        let (tx, _rx) = mpsc::channel(10);
        let result = FileWatcher::new(PathBuf::from("/nonexistent/path"), tx);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WatcherError::DirectoryNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let temp_dir = create_test_dir();
        let (tx, _rx) = mpsc::channel(10);

        // Create some initial files
        create_jsonl_file(
            temp_dir.path(),
            "existing.jsonl",
            r#"{"existing":true}
"#,
        );

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), tx).expect("Should create watcher");

        // Verify initial state
        assert_eq!(watcher.watch_dir(), temp_dir.path());
        assert_eq!(watcher.tracked_file_count().await, 1);
    }

    #[tokio::test]
    async fn test_watcher_error_display() {
        let err = WatcherError::DirectoryNotFound(PathBuf::from("/test/path"));
        assert_eq!(
            err.to_string(),
            "watch directory does not exist: /test/path"
        );

        let err = WatcherError::ChannelClosed;
        assert_eq!(err.to_string(), "failed to send event: channel closed");
    }
}
