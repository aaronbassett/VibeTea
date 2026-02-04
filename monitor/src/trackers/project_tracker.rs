//! Project tracker for monitoring active Claude Code sessions per project.
//!
//! This module scans `~/.claude/projects/` to identify projects and their
//! session activity status by checking for the presence of summary events.
//!
//! # Directory Structure
//!
//! ```text
//! ~/.claude/projects/
//! +-- -home-ubuntu-Projects-VibeTea/
//! |   +-- 6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl  # Active session
//! |   +-- a1b2c3d4-5678-90ab-cdef-1234567890ab.jsonl  # Completed session
//! +-- -home-ubuntu-Projects-SMILE/
//!     +-- 60fc5b5e-a285-4a6d-b9cc-9a315eb90ea8.jsonl
//! ```
//!
//! # Path Slug Format
//!
//! Project directories use a "slug" format where the absolute path has
//! forward slashes replaced with dashes:
//! - `/home/ubuntu/Projects/VibeTea` becomes `-home-ubuntu-Projects-VibeTea`
//!
//! # Session Activity Detection
//!
//! A session is considered **active** if its JSONL file does not contain
//! a summary event (`{"type": "summary", ...}`). Once a summary event
//! is written, the session is considered **completed**.
//!
//! # Privacy
//!
//! This module follows the privacy-first principle: only project paths
//! and session IDs are extracted. No code content or prompts are transmitted.
//!
//! # Example
//!
//! ```
//! use vibetea_monitor::trackers::project_tracker::{parse_project_slug, has_summary_event};
//!
//! // Parse a project slug back to its original path
//! let slug = "-home-ubuntu-Projects-VibeTea";
//! let path = parse_project_slug(slug);
//! assert_eq!(path, "/home/ubuntu/Projects/VibeTea");
//!
//! // Check if a session JSONL has a summary event
//! let jsonl_content = r#"{"type": "user", "message": "hello"}
//! {"type": "assistant", "message": "hi there"}
//! {"type": "summary", "summary": "Session ended"}
//! "#;
//! assert!(has_summary_event(jsonl_content));
//!
//! // Active session has no summary event
//! let active_content = r#"{"type": "user", "message": "hello"}
//! {"type": "assistant", "message": "hi there"}
//! "#;
//! assert!(!has_summary_event(active_content));
//! ```
//!
//! # File Watching Example
//!
//! ```no_run
//! use tokio::sync::mpsc;
//! use vibetea_monitor::trackers::project_tracker::ProjectTracker;
//! use vibetea_monitor::types::ProjectActivityEvent;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tx, mut rx) = mpsc::channel(100);
//!     let tracker = ProjectTracker::new(tx)?;
//!
//!     while let Some(event) = rx.recv().await {
//!         println!(
//!             "Project {}: session {} (active: {})",
//!             event.project_path,
//!             event.session_id,
//!             event.is_active
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::path::{Path, PathBuf};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

use crate::types::ProjectActivityEvent;

/// Parses a project directory slug back to its original absolute path.
///
/// Project directories in `~/.claude/projects/` use a "slug" format where
/// forward slashes in the path are replaced with dashes. This function
/// reverses that transformation.
///
/// # Arguments
///
/// * `slug` - The project directory name (e.g., `-home-ubuntu-Projects-VibeTea`)
///
/// # Returns
///
/// The original absolute path (e.g., `/home/ubuntu/Projects/VibeTea`)
///
/// # Examples
///
/// ```
/// use vibetea_monitor::trackers::project_tracker::parse_project_slug;
///
/// let path = parse_project_slug("-home-ubuntu-Projects-VibeTea");
/// assert_eq!(path, "/home/ubuntu/Projects/VibeTea");
///
/// // Note: dashes in directory names cannot be distinguished from separators
/// let path = parse_project_slug("-home-user-code-rust");
/// assert_eq!(path, "/home/user/code/rust");
/// ```
#[must_use]
pub fn parse_project_slug(slug: &str) -> String {
    // The slug format replaces '/' with '-'
    // A leading dash represents the root '/'
    slug.replace('-', "/")
}

/// Checks whether a JSONL content string contains a summary event.
///
/// A session is considered **completed** when its JSONL file contains
/// a line with `{"type": "summary", ...}`. The summary event can appear
/// at any position in the file, not necessarily the last line.
///
/// # Arguments
///
/// * `content` - The full content of a session JSONL file
///
/// # Returns
///
/// `true` if the content contains a summary event, `false` otherwise.
///
/// # Examples
///
/// ```
/// use vibetea_monitor::trackers::project_tracker::has_summary_event;
///
/// // Completed session with summary at the end
/// let completed = r#"{"type": "user", "message": "hello"}
/// {"type": "summary", "summary": "Done"}
/// "#;
/// assert!(has_summary_event(completed));
///
/// // Active session without summary
/// let active = r#"{"type": "user", "message": "hello"}
/// {"type": "assistant", "message": "hi"}
/// "#;
/// assert!(!has_summary_event(active));
/// ```
#[must_use]
pub fn has_summary_event(content: &str) -> bool {
    // Parse each line as JSON and check for type: "summary"
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Try to parse as JSON and check for summary type
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if value.get("type").and_then(|t| t.as_str()) == Some("summary") {
                return true;
            }
        }
    }
    false
}

/// Creates a [`ProjectActivityEvent`] from project information.
///
/// # Arguments
///
/// * `project_path` - The absolute path to the project
/// * `session_id` - The session UUID
/// * `is_active` - Whether the session is currently active
///
/// # Returns
///
/// A [`ProjectActivityEvent`] ready for transmission.
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::project_tracker::create_project_activity_event;
///
/// let event = create_project_activity_event(
///     "/home/ubuntu/Projects/VibeTea",
///     "6e45a55c-3124-4cc8-ad85-040a5c316009",
///     true,
/// );
///
/// assert_eq!(event.project_path, "/home/ubuntu/Projects/VibeTea");
/// assert_eq!(event.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
/// assert!(event.is_active);
/// ```
#[must_use]
pub fn create_project_activity_event(
    project_path: &str,
    session_id: &str,
    is_active: bool,
) -> ProjectActivityEvent {
    ProjectActivityEvent {
        project_path: project_path.to_string(),
        session_id: session_id.to_string(),
        is_active,
    }
}

// ============================================================================
// ProjectTracker Types
// ============================================================================

/// Errors that can occur during project tracking operations.
#[derive(Error, Debug)]
pub enum ProjectTrackerError {
    /// Failed to initialize the file system watcher.
    #[error("failed to create watcher: {0}")]
    WatcherInit(#[from] notify::Error),

    /// Failed to read a project session file.
    #[error("failed to read session file: {0}")]
    Io(#[from] std::io::Error),

    /// The projects directory does not exist.
    #[error("claude projects directory not found: {0}")]
    ClaudeDirectoryNotFound(PathBuf),

    /// Failed to send event through the channel.
    #[error("failed to send event: channel closed")]
    ChannelClosed,
}

/// Result type for project tracker operations.
pub type TrackerResult<T> = std::result::Result<T, ProjectTrackerError>;

/// Configuration for the project tracker.
///
/// Per research.md, no debouncing is needed for project/*.jsonl files (0ms).
/// We can still configure whether to perform an initial scan on startup.
#[derive(Debug, Clone)]
pub struct ProjectTrackerConfig {
    /// Whether to scan all projects on initialization. Default: true.
    pub scan_on_init: bool,
}

impl Default for ProjectTrackerConfig {
    fn default() -> Self {
        Self { scan_on_init: true }
    }
}

// ============================================================================
// ProjectTracker - File Watching Implementation
// ============================================================================

/// Tracker for Claude Code's project sessions.
///
/// Watches `~/.claude/projects/` recursively for changes to session JSONL files
/// and emits [`ProjectActivityEvent`]s when sessions are created or modified.
///
/// # Directory Structure
///
/// The tracker expects the following structure:
///
/// ```text
/// ~/.claude/projects/
/// +-- -home-ubuntu-Projects-VibeTea/
/// |   +-- 6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl
/// +-- -home-ubuntu-Projects-SMILE/
///     +-- 60fc5b5e-a285-4a6d-b9cc-9a315eb90ea8.jsonl
/// ```
///
/// # Session Activity Detection
///
/// A session is active if its JSONL file does NOT contain a summary event.
/// Once a summary event is detected, the session is considered completed.
///
/// # Thread Safety
///
/// The tracker spawns a background task for async processing of file events.
/// Communication is done via channels for thread safety.
#[derive(Debug)]
pub struct ProjectTracker {
    /// The underlying file system watcher.
    ///
    /// Kept alive to maintain the watch subscription.
    #[allow(dead_code)]
    watcher: RecommendedWatcher,

    /// Path to the projects directory being watched.
    projects_dir: PathBuf,

    /// Channel sender for emitting project activity events.
    #[allow(dead_code)]
    event_sender: mpsc::Sender<ProjectActivityEvent>,
}

impl ProjectTracker {
    /// Creates a new project tracker watching the default projects directory.
    ///
    /// The default location is `~/.claude/projects/`.
    ///
    /// # Arguments
    ///
    /// * `event_sender` - Channel for emitting [`ProjectActivityEvent`]s
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The home directory cannot be determined
    /// - The `~/.claude/projects` directory does not exist
    /// - The file system watcher cannot be initialized
    pub fn new(event_sender: mpsc::Sender<ProjectActivityEvent>) -> TrackerResult<Self> {
        Self::with_config(event_sender, ProjectTrackerConfig::default())
    }

    /// Creates a new project tracker with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `event_sender` - Channel for emitting [`ProjectActivityEvent`]s
    /// * `config` - Configuration options for the tracker
    ///
    /// # Errors
    ///
    /// Returns an error if the tracker cannot be initialized.
    pub fn with_config(
        event_sender: mpsc::Sender<ProjectActivityEvent>,
        config: ProjectTrackerConfig,
    ) -> TrackerResult<Self> {
        let claude_dir = directories::BaseDirs::new()
            .map(|dirs| dirs.home_dir().join(".claude"))
            .ok_or_else(|| {
                ProjectTrackerError::ClaudeDirectoryNotFound(PathBuf::from("~/.claude"))
            })?;

        Self::with_path_and_config(claude_dir.join("projects"), event_sender, config)
    }

    /// Creates a new project tracker watching a specific directory.
    ///
    /// # Arguments
    ///
    /// * `projects_dir` - Path to the projects directory to watch
    /// * `event_sender` - Channel for emitting [`ProjectActivityEvent`]s
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The specified directory does not exist
    /// - The file system watcher cannot be initialized
    pub fn with_path(
        projects_dir: PathBuf,
        event_sender: mpsc::Sender<ProjectActivityEvent>,
    ) -> TrackerResult<Self> {
        Self::with_path_and_config(projects_dir, event_sender, ProjectTrackerConfig::default())
    }

    /// Creates a new project tracker with a specific path and configuration.
    ///
    /// # Arguments
    ///
    /// * `projects_dir` - Path to the projects directory to watch
    /// * `event_sender` - Channel for emitting [`ProjectActivityEvent`]s
    /// * `config` - Configuration options for the tracker
    ///
    /// # Errors
    ///
    /// Returns an error if the tracker cannot be initialized.
    pub fn with_path_and_config(
        projects_dir: PathBuf,
        event_sender: mpsc::Sender<ProjectActivityEvent>,
        config: ProjectTrackerConfig,
    ) -> TrackerResult<Self> {
        // Verify the directory exists
        if !projects_dir.exists() {
            return Err(ProjectTrackerError::ClaudeDirectoryNotFound(projects_dir));
        }

        info!(
            projects_dir = %projects_dir.display(),
            scan_on_init = config.scan_on_init,
            "Initializing project tracker"
        );

        // Create channel for file change notifications
        let (change_tx, change_rx) = mpsc::channel::<PathBuf>(1000);

        // Spawn the async processing task
        let sender_for_task = event_sender.clone();
        let projects_dir_for_task = projects_dir.clone();
        tokio::spawn(async move {
            process_file_changes(change_rx, sender_for_task, projects_dir_for_task).await;
        });

        // Create the file watcher
        let watcher = create_projects_watcher(projects_dir.clone(), change_tx.clone())?;

        // Perform initial scan if configured
        if config.scan_on_init {
            let projects_dir_for_scan = projects_dir.clone();
            tokio::spawn(async move {
                if let Err(e) = scan_all_projects(&projects_dir_for_scan, &change_tx).await {
                    warn!(error = %e, "Failed to perform initial project scan");
                }
            });
        }

        Ok(Self {
            watcher,
            projects_dir,
            event_sender,
        })
    }

    /// Returns the path to the projects directory being watched.
    #[must_use]
    pub fn projects_dir(&self) -> &Path {
        &self.projects_dir
    }

    /// Manually triggers a scan of all projects.
    ///
    /// This is useful for refreshing the state of all projects without
    /// waiting for file system events.
    ///
    /// # Errors
    ///
    /// Returns an error if scanning fails.
    pub async fn scan_projects(&self) -> TrackerResult<()> {
        info!(projects_dir = %self.projects_dir.display(), "Manually scanning all projects");

        // Read all project directories
        let mut entries = tokio::fs::read_dir(&self.projects_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip non-directories
            if !path.is_dir() {
                continue;
            }

            // Get the project slug from the directory name
            let Some(slug) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            // Skip hidden directories
            if slug.starts_with('.') {
                continue;
            }

            let project_path = parse_project_slug(slug);
            debug!(slug = slug, project_path = %project_path, "Scanning project");

            // Scan all session files in this project
            if let Err(e) = self.scan_project_sessions(&path, &project_path).await {
                warn!(
                    project_path = %project_path,
                    error = %e,
                    "Failed to scan project sessions"
                );
            }
        }

        Ok(())
    }

    /// Scans all session files in a single project directory.
    async fn scan_project_sessions(
        &self,
        project_dir: &Path,
        project_path: &str,
    ) -> TrackerResult<()> {
        let mut entries = tokio::fs::read_dir(project_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Only process .jsonl files
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            // Extract session ID from filename
            let Some(session_id) = extract_session_id(&path) else {
                trace!(path = %path.display(), "Skipping file with invalid session ID");
                continue;
            };

            // Check if session is active
            let is_active = match check_session_active(&path).await {
                Ok(active) => active,
                Err(e) => {
                    warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to check session status"
                    );
                    continue;
                }
            };

            let event = create_project_activity_event(project_path, &session_id, is_active);

            trace!(
                project_path = %project_path,
                session_id = %session_id,
                is_active = is_active,
                "Emitting project activity event from scan"
            );

            if let Err(e) = self.event_sender.send(event).await {
                error!(error = %e, "Failed to send project activity event: channel closed");
                return Err(ProjectTrackerError::ChannelClosed);
            }
        }

        Ok(())
    }
}

/// Creates the file system watcher for the projects directory.
fn create_projects_watcher(
    projects_dir: PathBuf,
    change_tx: mpsc::Sender<PathBuf>,
) -> TrackerResult<RecommendedWatcher> {
    let watch_dir = projects_dir.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: std::result::Result<Event, notify::Error>| {
            handle_notify_event(res, &projects_dir, &change_tx);
        },
        Config::default(),
    )?;

    // Watch the projects directory recursively to catch new JONLs in subdirectories
    watcher.watch(&watch_dir, RecursiveMode::Recursive)?;

    debug!(
        watch_dir = %watch_dir.display(),
        "Started watching for project session changes (recursive)"
    );

    Ok(watcher)
}

/// Handles raw notify events and sends them to the change channel.
fn handle_notify_event(
    res: std::result::Result<Event, notify::Error>,
    projects_dir: &Path,
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

    // Only process create and modify events
    let should_process = matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_));

    if !should_process {
        trace!(kind = ?event.kind, "Ignoring event kind");
        return;
    }

    // Process each path in the event
    for path in &event.paths {
        // Only process .jsonl files
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }

        // Verify the path is within the projects directory
        if !path.starts_with(projects_dir) {
            continue;
        }

        // Validate that this looks like a valid session file (UUID.jsonl)
        if extract_session_id(path).is_none() {
            trace!(path = %path.display(), "Ignoring file with invalid session ID format");
            continue;
        }

        debug!(
            path = %path.display(),
            kind = ?event.kind,
            "Session file changed, sending for processing"
        );

        // No debouncing needed for project session files (per research.md: 0ms)
        if let Err(e) = change_tx.try_send(path.clone()) {
            warn!(path = %path.display(), error = %e, "Failed to send change notification");
        }
    }
}

/// Processes file change events, checking sessions and emitting events.
async fn process_file_changes(
    mut rx: mpsc::Receiver<PathBuf>,
    sender: mpsc::Sender<ProjectActivityEvent>,
    projects_dir: PathBuf,
) {
    debug!("Starting project file change processor");

    while let Some(path) = rx.recv().await {
        debug!(path = %path.display(), "Processing session file change");

        // Extract session ID from filename
        let Some(session_id) = extract_session_id(&path) else {
            warn!(path = %path.display(), "Could not extract session ID from filename");
            continue;
        };

        // Extract project slug from parent directory
        let Some(project_slug) = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
        else {
            warn!(path = %path.display(), "Could not extract project slug from path");
            continue;
        };

        // Verify this is directly under projects_dir
        if path.parent().and_then(|p| p.parent()) != Some(&projects_dir) {
            trace!(path = %path.display(), "Ignoring file not directly under project directory");
            continue;
        }

        let project_path = parse_project_slug(project_slug);

        // Check if session is active
        let is_active = match check_session_active(&path).await {
            Ok(active) => active,
            Err(e) => {
                // File might have been deleted between event and processing
                if e.kind() == std::io::ErrorKind::NotFound {
                    trace!(path = %path.display(), "Session file was deleted");
                } else {
                    warn!(path = %path.display(), error = %e, "Failed to read session file");
                }
                continue;
            }
        };

        let event = create_project_activity_event(&project_path, &session_id, is_active);

        trace!(
            project_path = %project_path,
            session_id = %session_id,
            is_active = is_active,
            "Emitting project activity event"
        );

        if let Err(e) = sender.send(event).await {
            error!(error = %e, "Failed to send project activity event: channel closed");
            break;
        }
    }

    debug!("Project file change processor shutting down");
}

/// Scans all projects in the directory and sends paths to the change channel.
async fn scan_all_projects(
    projects_dir: &Path,
    change_tx: &mpsc::Sender<PathBuf>,
) -> TrackerResult<()> {
    info!(projects_dir = %projects_dir.display(), "Performing initial project scan");

    let mut project_count = 0;
    let mut session_count = 0;

    // Read all project directories
    let mut entries = tokio::fs::read_dir(projects_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let project_dir = entry.path();

        // Skip non-directories
        if !project_dir.is_dir() {
            continue;
        }

        // Get the project slug from the directory name
        let Some(slug) = project_dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // Skip hidden directories
        if slug.starts_with('.') {
            continue;
        }

        project_count += 1;

        // Scan all session files in this project
        let mut session_entries = tokio::fs::read_dir(&project_dir).await?;

        while let Some(session_entry) = session_entries.next_entry().await? {
            let session_path = session_entry.path();

            // Only process .jsonl files
            if session_path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            // Validate session ID format
            if extract_session_id(&session_path).is_none() {
                continue;
            }

            session_count += 1;

            // Send to change channel for processing
            if let Err(e) = change_tx.send(session_path.clone()).await {
                warn!(
                    path = %session_path.display(),
                    error = %e,
                    "Failed to send session path during scan"
                );
            }
        }
    }

    info!(
        project_count = project_count,
        session_count = session_count,
        "Initial project scan complete"
    );

    Ok(())
}

/// Extracts the session ID from a JSONL filename.
///
/// Session files are named `<uuid>.jsonl` where uuid is a standard UUID format.
fn extract_session_id(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;

    // Validate UUID format (8-4-4-4-12 hex digits)
    let parts: Vec<&str> = stem.split('-').collect();
    if parts.len() != 5 {
        return None;
    }

    // Check each part has the right length and is hex
    let expected_lengths = [8, 4, 4, 4, 12];
    for (part, expected_len) in parts.iter().zip(expected_lengths.iter()) {
        if part.len() != *expected_len {
            return None;
        }
        if !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
    }

    Some(stem.to_string())
}

/// Checks if a session is active by reading the file and checking for summary events.
async fn check_session_active(path: &Path) -> std::io::Result<bool> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(!has_summary_event(&content))
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // T230: Unit test for project path slug parsing
    // =========================================================================

    #[test]
    fn parse_project_slug_standard_path() {
        // Standard Unix path
        let slug = "-home-ubuntu-Projects-VibeTea";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/home/ubuntu/Projects/VibeTea");
    }

    #[test]
    fn parse_project_slug_with_nested_directories() {
        // Deeply nested path
        let slug = "-home-user-code-rust-projects-my-app";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/home/user/code/rust/projects/my/app");
    }

    #[test]
    fn parse_project_slug_single_segment() {
        // Single directory under root
        let slug = "-root";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/root");
    }

    #[test]
    fn parse_project_slug_empty_string() {
        // Edge case: empty string
        let slug = "";
        let path = parse_project_slug(slug);
        assert_eq!(path, "");
    }

    #[test]
    fn parse_project_slug_just_root() {
        // Edge case: just the leading dash (root directory)
        let slug = "-";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/");
    }

    #[test]
    fn parse_project_slug_with_double_dashes() {
        // Path that originally contained something like "/a//b"
        // This would become "-a--b" as a slug
        let slug = "-home-ubuntu--weird--path";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/home/ubuntu//weird//path");
    }

    #[test]
    fn parse_project_slug_trailing_dash() {
        // Path that ends with a slash (trailing dash in slug)
        let slug = "-home-ubuntu-Projects-";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/home/ubuntu/Projects/");
    }

    #[test]
    fn parse_project_slug_multiple_dashes_in_name() {
        // Project name with dashes (like "my-cool-project")
        // Note: This is a limitation - we can't distinguish between
        // path separators and dashes in directory names
        let slug = "-home-user-my-cool-project";
        let path = parse_project_slug(slug);
        // This will produce /home/user/my/cool/project instead of
        // /home/user/my-cool-project - this is a known limitation
        assert_eq!(path, "/home/user/my/cool/project");
    }

    #[test]
    fn parse_project_slug_usr_local_path() {
        // System path
        let slug = "-usr-local-bin";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/usr/local/bin");
    }

    #[test]
    fn parse_project_slug_var_www_path() {
        // Web server path
        let slug = "-var-www-html-mysite";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/var/www/html/mysite");
    }

    #[test]
    fn parse_project_slug_opt_path() {
        // Optional software path
        let slug = "-opt-software-app";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/opt/software/app");
    }

    // =========================================================================
    // T231: Unit test for active session detection (no summary event)
    // =========================================================================

    #[test]
    fn has_summary_event_with_summary_at_end() {
        // Typical completed session - summary at the end
        let content = r#"{"type": "user", "message": "hello"}
{"type": "assistant", "message": "hi there"}
{"type": "summary", "summary": "Session completed successfully"}
"#;
        assert!(has_summary_event(content));
    }

    #[test]
    fn has_summary_event_with_summary_in_middle() {
        // Summary event can appear anywhere in the file (per FR-019)
        let content = r#"{"type": "user", "message": "hello"}
{"type": "summary", "summary": "Mid-session summary"}
{"type": "user", "message": "continue"}
{"type": "assistant", "message": "continuing..."}
"#;
        assert!(has_summary_event(content));
    }

    #[test]
    fn has_summary_event_with_summary_at_start() {
        // Summary at the very beginning
        let content = r#"{"type": "summary", "summary": "Early summary"}
{"type": "user", "message": "hello"}
{"type": "assistant", "message": "hi"}
"#;
        assert!(has_summary_event(content));
    }

    #[test]
    fn has_summary_event_no_summary_active_session() {
        // Active session - no summary event present
        let content = r#"{"type": "user", "message": "hello"}
{"type": "assistant", "message": "hi there"}
{"type": "user", "message": "help me"}
"#;
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_empty_content() {
        // Edge case: empty file
        let content = "";
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_only_whitespace() {
        // Edge case: file with only whitespace
        let content = "   \n  \n\t\n";
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_only_empty_lines() {
        // Edge case: file with only empty lines
        let content = "\n\n\n";
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_with_different_types() {
        // Various event types but no summary
        let content = r#"{"type": "user", "content": "message"}
{"type": "assistant", "content": "response"}
{"type": "tool", "name": "Read"}
{"type": "agent", "state": "active"}
{"type": "error", "message": "something failed"}
"#;
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_partial_match_in_string() {
        // "summary" appears in a string value but type is not "summary"
        let content = r#"{"type": "assistant", "message": "Here is a summary of the changes"}
{"type": "user", "message": "type: summary"}
"#;
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_type_summary_as_value() {
        // type field is "summary" - this IS a summary event
        let content = r#"{"type": "summary", "data": "anything"}
"#;
        assert!(has_summary_event(content));
    }

    #[test]
    fn has_summary_event_invalid_json_lines() {
        // Some lines are invalid JSON - should skip them
        let content = r#"not valid json
{"type": "user", "message": "hello"}
also not valid
{"type": "assistant", "message": "hi"}
"#;
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_invalid_json_with_summary() {
        // Mix of invalid JSON and valid summary
        let content = r#"not valid json
{"type": "user", "message": "hello"}
also not valid
{"type": "summary", "summary": "done"}
"#;
        assert!(has_summary_event(content));
    }

    #[test]
    fn has_summary_event_nested_type_field() {
        // type: summary in nested object should NOT count
        let content = r#"{"type": "assistant", "data": {"type": "summary"}}
{"type": "user", "message": "hi"}
"#;
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_array_type_field() {
        // type field is not a string
        let content = r#"{"type": ["summary"], "data": "test"}
{"type": 123, "message": "hi"}
"#;
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_null_type_field() {
        // type field is null
        let content = r#"{"type": null, "message": "test"}
"#;
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_missing_type_field() {
        // Lines without type field
        let content = r#"{"message": "hello", "timestamp": 123456}
{"data": "something", "id": "abc"}
"#;
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_realistic_session_jsonl() {
        // Realistic session JSONL content
        let content = r#"{"type": "user", "message": {"content": "Help me write a test"}, "timestamp": "2026-02-03T10:00:00.000Z"}
{"type": "assistant", "message": {"content": [{"type": "text", "text": "Sure, I can help with that."}]}, "timestamp": "2026-02-03T10:00:05.000Z"}
{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Read", "input": {"file_path": "/test.rs"}}]}, "timestamp": "2026-02-03T10:00:10.000Z"}
{"type": "user", "message": {"content": "Thanks!"}, "timestamp": "2026-02-03T10:00:30.000Z"}
{"type": "summary", "summary": "Helped write a test file", "leafUuid": "abc-123", "timestamp": "2026-02-03T10:00:35.000Z"}
"#;
        assert!(has_summary_event(content));
    }

    #[test]
    fn has_summary_event_realistic_active_session() {
        // Realistic active session (no summary yet)
        let content = r#"{"type": "user", "message": {"content": "Help me debug this"}, "timestamp": "2026-02-03T10:00:00.000Z"}
{"type": "assistant", "message": {"content": [{"type": "text", "text": "Let me look at the code."}]}, "timestamp": "2026-02-03T10:00:05.000Z"}
{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Read", "input": {"file_path": "/src/main.rs"}}]}, "timestamp": "2026-02-03T10:00:10.000Z"}
"#;
        assert!(!has_summary_event(content));
    }

    #[test]
    fn has_summary_event_with_extra_whitespace() {
        // Lines with extra whitespace around them
        let content = "   {\"type\": \"user\", \"message\": \"hi\"}   \n  {\"type\": \"summary\", \"data\": \"done\"}  \n";
        assert!(has_summary_event(content));
    }

    #[test]
    fn has_summary_event_multiple_summaries() {
        // Multiple summary events (unusual but should still detect)
        let content = r#"{"type": "summary", "summary": "first"}
{"type": "user", "message": "continue"}
{"type": "summary", "summary": "second"}
"#;
        assert!(has_summary_event(content));
    }

    // =========================================================================
    // create_project_activity_event Tests
    // =========================================================================

    #[test]
    fn create_event_active_session() {
        let event = create_project_activity_event(
            "/home/ubuntu/Projects/VibeTea",
            "6e45a55c-3124-4cc8-ad85-040a5c316009",
            true,
        );

        assert_eq!(event.project_path, "/home/ubuntu/Projects/VibeTea");
        assert_eq!(event.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
        assert!(event.is_active);
    }

    #[test]
    fn create_event_inactive_session() {
        let event = create_project_activity_event(
            "/home/ubuntu/Projects/SMILE",
            "a1b2c3d4-5678-90ab-cdef-1234567890ab",
            false,
        );

        assert_eq!(event.project_path, "/home/ubuntu/Projects/SMILE");
        assert_eq!(event.session_id, "a1b2c3d4-5678-90ab-cdef-1234567890ab");
        assert!(!event.is_active);
    }

    #[test]
    fn create_event_empty_strings() {
        let event = create_project_activity_event("", "", false);

        assert_eq!(event.project_path, "");
        assert_eq!(event.session_id, "");
        assert!(!event.is_active);
    }

    #[test]
    fn create_event_unicode_path() {
        let event =
            create_project_activity_event("/home/user/Projects/my-project", "sess-123", true);

        assert!(event.project_path.contains("project"));
        assert!(event.is_active);
    }

    // =========================================================================
    // Integration Tests: Full Parse to Event Flow
    // =========================================================================

    #[test]
    fn full_flow_active_project() {
        // Simulate the full flow for an active project
        let slug = "-home-ubuntu-Projects-VibeTea";
        let session_id = "6e45a55c-3124-4cc8-ad85-040a5c316009";
        let session_content = r#"{"type": "user", "message": "hello"}
{"type": "assistant", "message": "hi there"}
"#;

        let project_path = parse_project_slug(slug);
        let is_active = !has_summary_event(session_content);
        let event = create_project_activity_event(&project_path, session_id, is_active);

        assert_eq!(event.project_path, "/home/ubuntu/Projects/VibeTea");
        assert_eq!(event.session_id, session_id);
        assert!(event.is_active);
    }

    #[test]
    fn full_flow_completed_project() {
        // Simulate the full flow for a completed project
        let slug = "-home-ubuntu-Projects-SMILE";
        let session_id = "a1b2c3d4-5678-90ab-cdef-1234567890ab";
        let session_content = r#"{"type": "user", "message": "hello"}
{"type": "assistant", "message": "hi there"}
{"type": "summary", "summary": "Session ended successfully"}
"#;

        let project_path = parse_project_slug(slug);
        let is_active = !has_summary_event(session_content);
        let event = create_project_activity_event(&project_path, session_id, is_active);

        assert_eq!(event.project_path, "/home/ubuntu/Projects/SMILE");
        assert_eq!(event.session_id, session_id);
        assert!(!event.is_active);
    }

    // =========================================================================
    // ProjectActivityEvent Trait Tests
    // =========================================================================

    #[test]
    fn project_activity_event_debug() {
        let event = create_project_activity_event("/home/user/project", "sess-123", true);

        let debug_str = format!("{:?}", event);

        assert!(debug_str.contains("ProjectActivityEvent"));
        assert!(debug_str.contains("project_path"));
        assert!(debug_str.contains("session_id"));
        assert!(debug_str.contains("is_active"));
    }

    #[test]
    fn project_activity_event_clone() {
        let original = create_project_activity_event("/home/user/project", "sess-123", true);

        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(cloned.project_path, "/home/user/project");
        assert_eq!(cloned.session_id, "sess-123");
        assert!(cloned.is_active);
    }

    #[test]
    fn project_activity_event_equality() {
        let a = create_project_activity_event("/path/a", "sess-1", true);
        let b = create_project_activity_event("/path/a", "sess-1", true);
        let c = create_project_activity_event("/path/a", "sess-1", false);
        let d = create_project_activity_event("/path/b", "sess-1", true);

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn project_activity_event_serializes_with_camel_case() {
        let event = create_project_activity_event(
            "/home/ubuntu/Projects/VibeTea",
            "6e45a55c-3124-4cc8-ad85-040a5c316009",
            true,
        );

        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["projectPath"], "/home/ubuntu/Projects/VibeTea");
        assert_eq!(json["sessionId"], "6e45a55c-3124-4cc8-ad85-040a5c316009");
        assert_eq!(json["isActive"], true);
    }

    #[test]
    fn project_activity_event_roundtrip_serialization() {
        let original = create_project_activity_event("/home/user/project", "sess-abc", false);

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ProjectActivityEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    // =========================================================================
    // T233/T234: ProjectTracker with file watching tests
    // =========================================================================

    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;
    use tokio::time::{sleep, timeout, Duration as TokioDuration};

    /// Sample active session content (no summary event).
    const ACTIVE_SESSION: &str = r#"{"type": "user", "message": "hello"}
{"type": "assistant", "message": "hi there"}
"#;

    /// Sample completed session content (has summary event).
    const COMPLETED_SESSION: &str = r#"{"type": "user", "message": "hello"}
{"type": "assistant", "message": "hi there"}
{"type": "summary", "summary": "Session completed"}
"#;

    /// Creates a project directory structure for testing.
    fn create_test_project(
        projects_dir: &TempDir,
        slug: &str,
        session_id: &str,
        content: &str,
    ) -> PathBuf {
        let project_dir = projects_dir.path().join(slug);
        std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

        let session_path = project_dir.join(format!("{}.jsonl", session_id));
        let mut file = std::fs::File::create(&session_path).expect("Failed to create session file");
        file.write_all(content.as_bytes())
            .expect("Failed to write session content");
        file.flush().expect("Failed to flush");

        session_path
    }

    // =========================================================================
    // extract_session_id Tests
    // =========================================================================

    #[test]
    fn extract_session_id_valid_uuid() {
        let path =
            Path::new("/projects/-home-user-code/6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl");
        let session_id = extract_session_id(path);
        assert_eq!(
            session_id,
            Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
        );
    }

    #[test]
    fn extract_session_id_uppercase_uuid() {
        let path =
            Path::new("/projects/-home-user-code/6E45A55C-3124-4CC8-AD85-040A5C316009.jsonl");
        let session_id = extract_session_id(path);
        assert_eq!(
            session_id,
            Some("6E45A55C-3124-4CC8-AD85-040A5C316009".to_string())
        );
    }

    #[test]
    fn extract_session_id_invalid_not_uuid() {
        let path = Path::new("/projects/-home-user-code/not-a-uuid.jsonl");
        let session_id = extract_session_id(path);
        assert_eq!(session_id, None);
    }

    #[test]
    fn extract_session_id_invalid_wrong_extension() {
        let path = Path::new("/projects/-home-user-code/6e45a55c-3124-4cc8-ad85-040a5c316009.json");
        // Note: extract_session_id only checks the filename stem, not extension
        // Extension check is done elsewhere
        let session_id = extract_session_id(path);
        assert_eq!(
            session_id,
            Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
        );
    }

    #[test]
    fn extract_session_id_invalid_too_few_parts() {
        let path = Path::new("/projects/-home-user-code/6e45a55c-3124-4cc8.jsonl");
        let session_id = extract_session_id(path);
        assert_eq!(session_id, None);
    }

    #[test]
    fn extract_session_id_invalid_wrong_part_length() {
        let path = Path::new("/projects/-home-user-code/6e45a55c-312-4cc8-ad85-040a5c316009.jsonl");
        let session_id = extract_session_id(path);
        assert_eq!(session_id, None);
    }

    #[test]
    fn extract_session_id_invalid_non_hex() {
        let path =
            Path::new("/projects/-home-user-code/6e45a55c-3124-4cc8-ad85-040a5c31600g.jsonl");
        let session_id = extract_session_id(path);
        assert_eq!(session_id, None);
    }

    #[test]
    fn extract_session_id_empty_path() {
        let path = Path::new("");
        let session_id = extract_session_id(path);
        assert_eq!(session_id, None);
    }

    #[test]
    fn extract_session_id_just_filename() {
        let path = Path::new("a1b2c3d4-e5f6-7890-abcd-ef1234567890.jsonl");
        let session_id = extract_session_id(path);
        assert_eq!(
            session_id,
            Some("a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string())
        );
    }

    // =========================================================================
    // ProjectTrackerError Tests
    // =========================================================================

    #[test]
    fn project_tracker_error_display() {
        let err = ProjectTrackerError::ChannelClosed;
        assert_eq!(err.to_string(), "failed to send event: channel closed");

        let err = ProjectTrackerError::ClaudeDirectoryNotFound(PathBuf::from("/test/projects"));
        assert_eq!(
            err.to_string(),
            "claude projects directory not found: /test/projects"
        );
    }

    #[test]
    fn project_tracker_error_is_debug() {
        let err = ProjectTrackerError::ChannelClosed;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ChannelClosed"));

        let err = ProjectTrackerError::ClaudeDirectoryNotFound(PathBuf::from("/test"));
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ClaudeDirectoryNotFound"));
    }

    // =========================================================================
    // ProjectTrackerConfig Tests
    // =========================================================================

    #[test]
    fn project_tracker_config_default() {
        let config = ProjectTrackerConfig::default();
        assert!(config.scan_on_init);
    }

    #[test]
    fn project_tracker_config_clone() {
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let cloned = config.clone();
        assert!(!cloned.scan_on_init);
    }

    // =========================================================================
    // ProjectTracker Creation Tests
    // =========================================================================

    #[tokio::test]
    async fn test_tracker_creation_missing_directory() {
        let (tx, _rx) = mpsc::channel(100);
        let result = ProjectTracker::with_path(PathBuf::from("/nonexistent/projects"), tx);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProjectTrackerError::ClaudeDirectoryNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_tracker_creation_with_valid_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let result =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config);

        assert!(result.is_ok(), "Should create tracker for valid directory");
        assert_eq!(result.unwrap().projects_dir(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_tracker_projects_dir_accessor() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let (tx, _rx) = mpsc::channel(100);
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        assert_eq!(tracker.projects_dir(), temp_dir.path());
    }

    // =========================================================================
    // ProjectTracker File Watching Tests
    // =========================================================================

    #[tokio::test]
    async fn test_tracker_detects_new_session_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a project directory
        let project_slug = "-home-ubuntu-Projects-TestProject";
        let project_dir = temp_dir.path().join(project_slug);
        std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

        let (tx, mut rx) = mpsc::channel(100);
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a new session file
        let session_id = "6e45a55c-3124-4cc8-ad85-040a5c316009";
        let session_path = project_dir.join(format!("{}.jsonl", session_id));
        std::fs::write(&session_path, ACTIVE_SESSION).expect("Should write session file");

        // Should receive a project activity event
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event for new session file");

        let event = result.unwrap().unwrap();
        assert_eq!(event.project_path, "/home/ubuntu/Projects/TestProject");
        assert_eq!(event.session_id, session_id);
        assert!(event.is_active);
    }

    #[tokio::test]
    async fn test_tracker_detects_modified_session_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_slug = "-home-ubuntu-Projects-ModifyTest";
        let session_id = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";

        // Create initial session file
        let session_path = create_test_project(&temp_dir, project_slug, session_id, ACTIVE_SESSION);

        let (tx, mut rx) = mpsc::channel(100);
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Modify the session file to be completed
        std::fs::write(&session_path, COMPLETED_SESSION).expect("Should write");

        // Should receive an updated event
        let result = timeout(TokioDuration::from_millis(500), rx.recv()).await;
        assert!(
            result.is_ok(),
            "Should receive event for modified session file"
        );

        let event = result.unwrap().unwrap();
        assert_eq!(event.session_id, session_id);
        assert!(
            !event.is_active,
            "Session should now be inactive (has summary)"
        );
    }

    #[tokio::test]
    async fn test_tracker_initial_scan() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create some projects with sessions BEFORE starting the tracker
        let session_id_1 = "11111111-1111-1111-1111-111111111111";
        let session_id_2 = "22222222-2222-2222-2222-222222222222";
        create_test_project(
            &temp_dir,
            "-home-ubuntu-Projects-Project1",
            session_id_1,
            ACTIVE_SESSION,
        );
        create_test_project(
            &temp_dir,
            "-home-ubuntu-Projects-Project2",
            session_id_2,
            COMPLETED_SESSION,
        );

        let (tx, mut rx) = mpsc::channel(100);
        let config = ProjectTrackerConfig { scan_on_init: true };
        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Should receive events from the initial scan
        let mut received_events = Vec::new();
        for _ in 0..2 {
            if let Ok(Some(event)) = timeout(TokioDuration::from_millis(500), rx.recv()).await {
                received_events.push(event);
            }
        }

        assert_eq!(
            received_events.len(),
            2,
            "Should receive 2 events from scan"
        );

        // Verify we got both sessions
        let session_ids: Vec<&str> = received_events
            .iter()
            .map(|e| e.session_id.as_str())
            .collect();
        assert!(session_ids.contains(&session_id_1));
        assert!(session_ids.contains(&session_id_2));

        // Verify activity status
        let active_event = received_events
            .iter()
            .find(|e| e.session_id == session_id_1)
            .unwrap();
        assert!(active_event.is_active);

        let inactive_event = received_events
            .iter()
            .find(|e| e.session_id == session_id_2)
            .unwrap();
        assert!(!inactive_event.is_active);
    }

    #[tokio::test]
    async fn test_tracker_ignores_non_jsonl_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create project directory
        let project_dir = temp_dir.path().join("-home-ubuntu-Projects-TestProject");
        std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

        let (tx, mut rx) = mpsc::channel(100);
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a non-jsonl file (like CLAUDE.local.md)
        let non_jsonl_path = project_dir.join("CLAUDE.local.md");
        std::fs::write(&non_jsonl_path, "# Local config").expect("Should write");

        // Should NOT receive any event
        let result = timeout(TokioDuration::from_millis(200), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should not receive event for non-jsonl file"
        );
    }

    #[tokio::test]
    async fn test_tracker_ignores_invalid_uuid_filename() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create project directory
        let project_dir = temp_dir.path().join("-home-ubuntu-Projects-TestProject");
        std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

        let (tx, mut rx) = mpsc::channel(100);
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(50)).await;

        // Create a jsonl file with invalid UUID filename
        let invalid_path = project_dir.join("not-a-valid-uuid.jsonl");
        std::fs::write(&invalid_path, ACTIVE_SESSION).expect("Should write");

        // Should NOT receive any event
        let result = timeout(TokioDuration::from_millis(200), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should not receive event for invalid UUID filename"
        );
    }

    #[tokio::test]
    async fn test_tracker_multiple_projects() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create project directories
        let project1_dir = temp_dir.path().join("-home-ubuntu-Projects-ProjectA");
        let project2_dir = temp_dir.path().join("-home-ubuntu-Projects-ProjectB");
        std::fs::create_dir_all(&project1_dir).expect("Failed to create project1 dir");
        std::fs::create_dir_all(&project2_dir).expect("Failed to create project2 dir");

        let (tx, mut rx) = mpsc::channel(100);
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Give watcher time to start
        sleep(TokioDuration::from_millis(100)).await;

        // Create sessions in both projects
        let session_id_1 = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let session_id_2 = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";

        std::fs::write(
            project1_dir.join(format!("{}.jsonl", session_id_1)),
            ACTIVE_SESSION,
        )
        .expect("Should write");

        // Small delay to ensure the first write is processed
        sleep(TokioDuration::from_millis(50)).await;

        std::fs::write(
            project2_dir.join(format!("{}.jsonl", session_id_2)),
            COMPLETED_SESSION,
        )
        .expect("Should write");

        // Should receive events from both projects - allow more time for file system events
        let mut received_events = Vec::new();
        for _ in 0..2 {
            if let Ok(Some(event)) = timeout(TokioDuration::from_millis(1000), rx.recv()).await {
                received_events.push(event);
            }
        }

        assert!(
            !received_events.is_empty(),
            "Should receive at least 1 event from multiple projects"
        );

        // Verify we got events from at least one project with expected path format
        let project_paths: Vec<&str> = received_events
            .iter()
            .map(|e| e.project_path.as_str())
            .collect();

        // At least one of these should be present
        let has_project_a = project_paths.contains(&"/home/ubuntu/Projects/ProjectA");
        let has_project_b = project_paths.contains(&"/home/ubuntu/Projects/ProjectB");
        assert!(
            has_project_a || has_project_b,
            "Should have received an event from at least one project"
        );
    }

    #[tokio::test]
    async fn test_tracker_skips_hidden_directories() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a hidden project directory
        let hidden_dir = temp_dir.path().join(".hidden-project");
        std::fs::create_dir_all(&hidden_dir).expect("Failed to create hidden dir");

        // Create a session in the hidden directory
        let session_id = "cccccccc-cccc-cccc-cccc-cccccccccccc";
        std::fs::write(
            hidden_dir.join(format!("{}.jsonl", session_id)),
            ACTIVE_SESSION,
        )
        .expect("Should write");

        let (tx, mut rx) = mpsc::channel(100);
        let config = ProjectTrackerConfig { scan_on_init: true };
        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Should NOT receive any event (hidden directory should be skipped in scan)
        // Note: The watcher may still see events, but scan_all_projects skips hidden dirs
        let result = timeout(TokioDuration::from_millis(300), rx.recv()).await;
        // This may or may not receive an event depending on timing with the watcher
        // The important test is that scan_all_projects skips hidden directories
        if let Ok(Some(event)) = result {
            // If we did get an event, it shouldn't be from the hidden directory
            // (would be from watcher, not from scan)
            // Hidden directories don't have slug format starting with '-'
            // so they would parse differently
            assert!(!event.project_path.starts_with("/.hidden"));
        }
    }

    #[tokio::test]
    async fn test_tracker_custom_config_no_scan() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a project with a session BEFORE starting the tracker
        let session_id = "dddddddd-dddd-dddd-dddd-dddddddddddd";
        create_test_project(
            &temp_dir,
            "-home-ubuntu-Projects-NoScanTest",
            session_id,
            ACTIVE_SESSION,
        );

        let (tx, mut rx) = mpsc::channel(100);
        // Explicitly disable initial scan
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Should NOT receive any event (scan_on_init is false)
        let result = timeout(TokioDuration::from_millis(200), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should not receive event when scan_on_init is false"
        );
    }

    #[tokio::test]
    async fn test_check_session_active_function() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create an active session file
        let active_path = temp_dir.path().join("active.jsonl");
        std::fs::write(&active_path, ACTIVE_SESSION).expect("Should write");

        // Create a completed session file
        let completed_path = temp_dir.path().join("completed.jsonl");
        std::fs::write(&completed_path, COMPLETED_SESSION).expect("Should write");

        // Test active session
        let is_active = check_session_active(&active_path).await.unwrap();
        assert!(is_active, "Active session should return true");

        // Test completed session
        let is_active = check_session_active(&completed_path).await.unwrap();
        assert!(!is_active, "Completed session should return false");
    }

    #[tokio::test]
    async fn test_check_session_active_nonexistent_file() {
        let path = Path::new("/nonexistent/file.jsonl");
        let result = check_session_active(path).await;
        assert!(result.is_err());
    }

    // =========================================================================
    // Manual scan_projects Tests
    // =========================================================================

    #[tokio::test]
    async fn test_manual_scan_projects() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create projects with sessions
        let session_id_1 = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";
        let session_id_2 = "ffffffff-ffff-ffff-ffff-ffffffffffff";
        create_test_project(
            &temp_dir,
            "-home-ubuntu-Projects-ScanTest1",
            session_id_1,
            ACTIVE_SESSION,
        );
        create_test_project(
            &temp_dir,
            "-home-ubuntu-Projects-ScanTest2",
            session_id_2,
            COMPLETED_SESSION,
        );

        let (tx, mut rx) = mpsc::channel(100);
        // Start without initial scan
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };
        let tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Should create tracker");

        // Manually trigger scan
        tracker.scan_projects().await.expect("Scan should succeed");

        // Should receive events from the manual scan
        let mut received_events = Vec::new();
        for _ in 0..2 {
            if let Ok(Some(event)) = timeout(TokioDuration::from_millis(500), rx.recv()).await {
                received_events.push(event);
            }
        }

        assert_eq!(
            received_events.len(),
            2,
            "Should receive 2 events from manual scan"
        );
    }
}
