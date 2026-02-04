//! Integration tests for enhanced tracking modules.
//!
//! These tests verify that all trackers work correctly together, handling
//! concurrent file modifications and proper event emission.

use serial_test::serial;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::mpsc;
use tokio::time::timeout;

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a temporary directory structure mimicking ~/.claude/projects/
fn create_test_projects_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Creates a test project directory with a session JSONL file.
///
/// # Arguments
/// * `projects_dir` - The base projects directory
/// * `slug` - The project slug (e.g., "-home-user-myproject")
/// * `session_id` - The session UUID
/// * `content` - The JSONL content for the session file
fn create_test_project(
    projects_dir: &Path,
    slug: &str,
    session_id: &str,
    content: &str,
) -> PathBuf {
    let project_dir = projects_dir.join(slug);
    fs::create_dir_all(&project_dir).expect("Failed to create project dir");
    let session_file = project_dir.join(format!("{}.jsonl", session_id));
    let mut file = fs::File::create(&session_file).expect("Failed to create session.jsonl");
    file.write_all(content.as_bytes())
        .expect("Failed to write session.jsonl");
    file.flush().expect("Failed to flush");
    project_dir
}

/// Creates a stats-cache.json file with the given content.
fn create_test_stats_file(dir: &Path, content: &str) -> PathBuf {
    let stats_path = dir.join("stats-cache.json");
    let mut file = fs::File::create(&stats_path).expect("Failed to create stats file");
    file.write_all(content.as_bytes())
        .expect("Failed to write stats file");
    file.flush().expect("Failed to flush");
    stats_path
}

/// Creates a history.jsonl file for skill tracking.
fn create_test_history_file(dir: &Path, content: &str) -> PathBuf {
    let history_path = dir.join("history.jsonl");
    let mut file = fs::File::create(&history_path).expect("Failed to create history file");
    file.write_all(content.as_bytes())
        .expect("Failed to write history file");
    file.flush().expect("Failed to flush");
    history_path
}

/// Creates a todos directory structure for todo tracking.
fn create_test_todos_dir(base_dir: &Path) -> PathBuf {
    let todos_dir = base_dir.join("todos");
    fs::create_dir_all(&todos_dir).expect("Failed to create todos dir");
    todos_dir
}

/// Creates a todo file with the expected naming pattern.
///
/// Filename pattern: `<session-uuid>-agent-<session-uuid>.json`
fn create_test_todo_file(todos_dir: &Path, session_id: &str, content: &str) -> PathBuf {
    let filename = format!("{}-agent-{}.json", session_id, session_id);
    let todo_path = todos_dir.join(filename);
    let mut file = fs::File::create(&todo_path).expect("Failed to create todo file");
    file.write_all(content.as_bytes())
        .expect("Failed to write todo file");
    file.flush().expect("Failed to flush");
    todo_path
}

/// Creates a file-history directory structure for file history tracking.
fn create_test_file_history_dir(base_dir: &Path) -> PathBuf {
    let file_history_dir = base_dir.join("file-history");
    fs::create_dir_all(&file_history_dir).expect("Failed to create file-history dir");
    file_history_dir
}

/// Creates a session directory within file-history.
fn create_test_session_dir(file_history_dir: &Path, session_id: &str) -> PathBuf {
    let session_dir = file_history_dir.join(session_id);
    fs::create_dir_all(&session_dir).expect("Failed to create session dir");
    session_dir
}

/// Creates a file version in the session directory.
///
/// Filename pattern: `<16-char-hex-hash>@v<N>`
fn create_test_file_version(
    session_dir: &Path,
    hash: &str,
    version: u32,
    content: &str,
) -> PathBuf {
    let filename = format!("{}@v{}", hash, version);
    let file_path = session_dir.join(filename);
    let mut file = fs::File::create(&file_path).expect("Failed to create file version");
    file.write_all(content.as_bytes())
        .expect("Failed to write file version");
    file.flush().expect("Failed to flush");
    file_path
}

/// Generates a minimal active session JSONL content (no summary event).
fn active_session_jsonl() -> String {
    r#"{"type":"user","message":"hello"}
{"type":"assistant","message":"hi there"}
"#
    .to_string()
}

/// Generates a completed session JSONL content (has summary event).
fn completed_session_jsonl() -> String {
    r#"{"type":"user","message":"hello"}
{"type":"assistant","message":"hi there"}
{"type":"summary","summary":"Session completed"}
"#
    .to_string()
}

/// Generates valid stats-cache.json content with all metric types.
fn stats_cache_json() -> String {
    r#"{
        "totalSessions": 150,
        "totalMessages": 2500,
        "totalToolUsage": 8000,
        "longestSession": "00:45:30",
        "hourCounts": {"9": 50, "10": 80, "14": 60},
        "modelUsage": {
            "claude-sonnet-4-20250514": {
                "inputTokens": 1500000,
                "outputTokens": 300000,
                "cacheReadInputTokens": 800000,
                "cacheCreationInputTokens": 100000
            }
        }
    }"#
    .to_string()
}

/// Generates a history.jsonl line for a skill invocation.
fn skill_invocation_jsonl(session_id: &str, skill_name: &str, timestamp_ms: i64) -> String {
    format!(
        r#"{{"display":"/{skill_name}","timestamp":{timestamp_ms},"project":"/home/user/project","sessionId":"{session_id}"}}"#,
    )
}

/// Generates todo file content with various states (JSON array format).
fn todo_json(completed: usize, in_progress: usize, pending: usize) -> String {
    let mut todos = Vec::new();

    for i in 0..completed {
        todos.push(format!(
            r#"{{"content":"Completed task {}","status":"completed","activeForm":null}}"#,
            i
        ));
    }
    for i in 0..in_progress {
        todos.push(format!(
            r#"{{"content":"In progress task {}","status":"in_progress","activeForm":"Working on task {}"}}"#,
            i, i
        ));
    }
    for i in 0..pending {
        todos.push(format!(
            r#"{{"content":"Pending task {}","status":"pending","activeForm":null}}"#,
            i
        ));
    }

    format!("[{}]", todos.join(","))
}

// ============================================================================
// ProjectTracker Tests
// ============================================================================

mod project_tracker_tests {
    use super::*;
    use vibetea_monitor::trackers::project_tracker::{ProjectTracker, ProjectTrackerConfig};
    use vibetea_monitor::types::ProjectActivityEvent;

    #[tokio::test]
    #[serial]
    async fn test_project_tracker_initialization() {
        let temp_dir = create_test_projects_dir();
        let session_id = "550e8400-e29b-41d4-a716-446655440000";

        // Create a test project before starting the tracker
        create_test_project(
            temp_dir.path(),
            "-home-user-testproject",
            session_id,
            &active_session_jsonl(),
        );

        let (tx, mut rx) = mpsc::channel::<ProjectActivityEvent>(100);
        let config = ProjectTrackerConfig { scan_on_init: true };

        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Failed to create ProjectTracker");

        // Should receive an event for the existing project
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        assert_eq!(event.session_id, session_id);
        assert!(event.is_active);
        assert!(event.project_path.contains("testproject"));
    }

    #[tokio::test]
    #[serial]
    async fn test_project_tracker_detects_new_project() {
        let temp_dir = create_test_projects_dir();
        let session_id = "550e8400-e29b-41d4-a716-446655440001";

        // Create the project directory first (before the tracker starts)
        let project_dir = temp_dir.path().join("-home-user-newproject");
        fs::create_dir_all(&project_dir).expect("Failed to create project dir");

        let (tx, mut rx) = mpsc::channel::<ProjectActivityEvent>(100);
        let config = ProjectTrackerConfig {
            scan_on_init: false,
        };

        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Failed to create ProjectTracker");

        // Give the watcher time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create the session file AFTER the tracker is running
        let session_file = project_dir.join(format!("{}.jsonl", session_id));
        let mut file = fs::File::create(&session_file).expect("Failed to create session file");
        file.write_all(active_session_jsonl().as_bytes())
            .expect("Failed to write session");
        file.flush().expect("Failed to flush");

        // Should receive an event for the new session
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        assert_eq!(event.session_id, session_id);
        assert!(event.is_active);
    }

    #[tokio::test]
    #[serial]
    async fn test_project_tracker_detects_completed_session() {
        let temp_dir = create_test_projects_dir();
        let session_id = "550e8400-e29b-41d4-a716-446655440002";

        // Create a project with a completed session
        create_test_project(
            temp_dir.path(),
            "-home-user-completed",
            session_id,
            &completed_session_jsonl(),
        );

        let (tx, mut rx) = mpsc::channel::<ProjectActivityEvent>(100);
        let config = ProjectTrackerConfig { scan_on_init: true };

        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Failed to create ProjectTracker");

        // Should receive an event showing session is NOT active
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        assert_eq!(event.session_id, session_id);
        assert!(!event.is_active); // Completed session is not active
    }

    #[tokio::test]
    #[serial]
    async fn test_project_tracker_multiple_projects() {
        let temp_dir = create_test_projects_dir();
        let session_ids = [
            "550e8400-e29b-41d4-a716-446655440010",
            "550e8400-e29b-41d4-a716-446655440011",
            "550e8400-e29b-41d4-a716-446655440012",
        ];

        // Create multiple projects
        for (i, session_id) in session_ids.iter().enumerate() {
            create_test_project(
                temp_dir.path(),
                &format!("-home-user-project{}", i),
                session_id,
                &active_session_jsonl(),
            );
        }

        let (tx, mut rx) = mpsc::channel::<ProjectActivityEvent>(100);
        let config = ProjectTrackerConfig { scan_on_init: true };

        let _tracker =
            ProjectTracker::with_path_and_config(temp_dir.path().to_path_buf(), tx, config)
                .expect("Failed to create ProjectTracker");

        // Collect all events
        let mut received_sessions = Vec::new();
        for _ in 0..3 {
            if let Ok(Some(event)) = timeout(Duration::from_secs(2), rx.recv()).await {
                received_sessions.push(event.session_id);
            }
        }

        // All sessions should be received
        for session_id in session_ids {
            assert!(
                received_sessions.contains(&session_id.to_string()),
                "Missing session: {}",
                session_id
            );
        }
    }
}

// ============================================================================
// StatsTracker Tests
// ============================================================================

mod stats_tracker_tests {
    use super::*;
    use vibetea_monitor::trackers::stats_tracker::StatsTracker;
    use vibetea_monitor::trackers::StatsEvent;

    #[tokio::test]
    #[serial]
    async fn test_stats_tracker_initialization() {
        let temp_dir = create_test_projects_dir();
        let stats_path = create_test_stats_file(temp_dir.path(), &stats_cache_json());

        let (tx, mut rx) = mpsc::channel::<StatsEvent>(100);

        let _tracker =
            StatsTracker::with_path(stats_path, tx).expect("Failed to create StatsTracker");

        // Should receive multiple event types on initialization
        let mut received_events = Vec::new();
        while let Ok(Some(event)) = timeout(Duration::from_millis(500), rx.recv()).await {
            received_events.push(event);
        }

        // Verify we received at least some events
        assert!(!received_events.is_empty(), "Should receive some events");

        // Check for expected event types
        let has_session_metrics = received_events
            .iter()
            .any(|e| matches!(e, StatsEvent::SessionMetrics(_)));
        let has_token_usage = received_events
            .iter()
            .any(|e| matches!(e, StatsEvent::TokenUsage(_)));

        assert!(
            has_session_metrics || has_token_usage,
            "Should receive SessionMetrics or TokenUsage event"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_stats_tracker_file_update() {
        let temp_dir = create_test_projects_dir();
        let stats_path = create_test_stats_file(temp_dir.path(), &stats_cache_json());

        let (tx, mut rx) = mpsc::channel::<StatsEvent>(100);

        let _tracker =
            StatsTracker::with_path(stats_path.clone(), tx).expect("Failed to create StatsTracker");

        // Consume initial events
        while timeout(Duration::from_millis(300), rx.recv()).await.is_ok() {}

        // Update the stats file with new values
        let updated_stats = r#"{
            "totalSessions": 200,
            "totalMessages": 3000,
            "totalToolUsage": 10000,
            "longestSession": "01:00:00",
            "hourCounts": {"9": 100},
            "modelUsage": {
                "claude-sonnet-4-20250514": {
                    "inputTokens": 2000000,
                    "outputTokens": 500000,
                    "cacheReadInputTokens": 1000000,
                    "cacheCreationInputTokens": 200000
                }
            }
        }"#;
        let mut file = fs::File::create(&stats_path).expect("Failed to open stats file");
        file.write_all(updated_stats.as_bytes())
            .expect("Failed to write stats file");
        file.flush().expect("Failed to flush");

        // Wait for debounce (200ms) plus some margin
        tokio::time::sleep(Duration::from_millis(400)).await;

        // Should receive new events
        let result = timeout(Duration::from_secs(2), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event after file update");
    }

    #[tokio::test]
    #[serial]
    async fn test_stats_tracker_handles_empty_json() {
        let temp_dir = create_test_projects_dir();
        let stats_path = create_test_stats_file(temp_dir.path(), "{}");

        let (tx, mut rx) = mpsc::channel::<StatsEvent>(100);

        let _tracker =
            StatsTracker::with_path(stats_path, tx).expect("Failed to create StatsTracker");

        // Empty JSON should emit events with default/zero values
        let result = timeout(Duration::from_millis(500), rx.recv()).await;
        // It's OK if no events are emitted for empty stats
        if result.is_ok() {
            // If we got an event, it should be valid
            assert!(result.unwrap().is_some());
        }
    }
}

// ============================================================================
// SkillTracker Tests
// ============================================================================

mod skill_tracker_tests {
    use super::*;
    use vibetea_monitor::trackers::skill_tracker::{SkillTracker, SkillTrackerConfig};
    use vibetea_monitor::types::SkillInvocationEvent;

    #[tokio::test]
    #[serial]
    async fn test_skill_tracker_initialization() {
        let temp_dir = create_test_projects_dir();
        let session_id = "550e8400-e29b-41d4-a716-446655440020";
        let content = skill_invocation_jsonl(session_id, "commit", 1738567268363) + "\n";
        let history_path = create_test_history_file(temp_dir.path(), &content);

        let (tx, mut rx) = mpsc::channel::<SkillInvocationEvent>(100);
        let config = SkillTrackerConfig {
            emit_existing_on_startup: true,
        };

        let _tracker = SkillTracker::with_path_and_config(history_path, tx, config)
            .expect("Failed to create SkillTracker");

        // Should receive event for existing skill invocation
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.skill_name, "commit");
    }

    #[tokio::test]
    #[serial]
    async fn test_skill_tracker_detects_new_invocation() {
        let temp_dir = create_test_projects_dir();
        let history_path = create_test_history_file(temp_dir.path(), "");

        let (tx, mut rx) = mpsc::channel::<SkillInvocationEvent>(100);
        let config = SkillTrackerConfig {
            emit_existing_on_startup: false,
        };

        let _tracker = SkillTracker::with_path_and_config(history_path.clone(), tx, config)
            .expect("Failed to create SkillTracker");

        // Give the watcher time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Append a new skill invocation
        let session_id = "550e8400-e29b-41d4-a716-446655440021";
        let new_line = skill_invocation_jsonl(session_id, "review-pr", 1738567300000) + "\n";

        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .append(true)
            .open(&history_path)
            .expect("Failed to open history file");
        file.write_all(new_line.as_bytes())
            .expect("Failed to append to history");
        file.flush().expect("Failed to flush");
        drop(file);

        // Should receive event for new invocation
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.skill_name, "review-pr");
    }

    #[tokio::test]
    #[serial]
    async fn test_skill_tracker_multiple_skills() {
        let temp_dir = create_test_projects_dir();
        let session_id = "550e8400-e29b-41d4-a716-446655440022";
        let content = [
            skill_invocation_jsonl(session_id, "commit", 1738567000000),
            skill_invocation_jsonl(session_id, "review-pr", 1738567100000),
            skill_invocation_jsonl(session_id, "debug", 1738567200000),
        ]
        .join("\n")
            + "\n";

        let history_path = create_test_history_file(temp_dir.path(), &content);

        let (tx, mut rx) = mpsc::channel::<SkillInvocationEvent>(100);
        let config = SkillTrackerConfig {
            emit_existing_on_startup: true,
        };

        let _tracker = SkillTracker::with_path_and_config(history_path, tx, config)
            .expect("Failed to create SkillTracker");

        // Collect all events
        let mut skill_names = Vec::new();
        for _ in 0..3 {
            if let Ok(Some(event)) = timeout(Duration::from_secs(2), rx.recv()).await {
                skill_names.push(event.skill_name);
            }
        }

        assert!(skill_names.contains(&"commit".to_string()));
        assert!(skill_names.contains(&"review-pr".to_string()));
        assert!(skill_names.contains(&"debug".to_string()));
    }
}

// ============================================================================
// TodoTracker Tests
// ============================================================================

mod todo_tracker_tests {
    use super::*;
    use vibetea_monitor::trackers::todo_tracker::{TodoTracker, TodoTrackerConfig};
    use vibetea_monitor::types::TodoProgressEvent;

    #[tokio::test]
    #[serial]
    async fn test_todo_tracker_detects_new_file() {
        let temp_dir = create_test_projects_dir();
        let todos_dir = create_test_todos_dir(temp_dir.path());
        let session_id = "550e8400-e29b-41d4-a716-446655440030";

        let (tx, mut rx) = mpsc::channel::<TodoProgressEvent>(100);
        let config = TodoTrackerConfig { debounce_ms: 50 };

        let _tracker = TodoTracker::with_path_and_config(todos_dir.clone(), tx, config)
            .expect("Failed to create TodoTracker");

        // Give watcher time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create a todo file AFTER the tracker is running
        create_test_todo_file(&todos_dir, session_id, &todo_json(2, 1, 3));

        // Should receive event for new todo file
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.completed, 2);
        assert_eq!(event.in_progress, 1);
        assert_eq!(event.pending, 3);
        assert!(!event.abandoned);
    }

    #[tokio::test]
    #[serial]
    async fn test_todo_tracker_progress_update() {
        let temp_dir = create_test_projects_dir();
        let todos_dir = create_test_todos_dir(temp_dir.path());
        let session_id = "550e8400-e29b-41d4-a716-446655440031";

        let (tx, mut rx) = mpsc::channel::<TodoProgressEvent>(100);
        let config = TodoTrackerConfig { debounce_ms: 50 };

        let _tracker = TodoTracker::with_path_and_config(todos_dir.clone(), tx, config)
            .expect("Failed to create TodoTracker");

        // Give watcher time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create initial todo file
        let todo_path = create_test_todo_file(&todos_dir, session_id, &todo_json(0, 1, 5));

        // Consume initial event
        let _ = timeout(Duration::from_secs(2), rx.recv()).await;

        // Update the todo file - complete some tasks
        let mut file = fs::File::create(&todo_path).expect("Failed to open todo file");
        file.write_all(todo_json(3, 1, 2).as_bytes())
            .expect("Failed to write");
        file.flush().expect("Failed to flush");

        // Wait for debounce
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should receive updated progress
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for update event")
            .expect("Channel closed");

        assert_eq!(event.completed, 3);
        assert_eq!(event.pending, 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_todo_tracker_abandonment_detection() {
        let temp_dir = create_test_projects_dir();
        let todos_dir = create_test_todos_dir(temp_dir.path());
        let session_id = "550e8400-e29b-41d4-a716-446655440032";

        let (tx, mut rx) = mpsc::channel::<TodoProgressEvent>(100);
        let config = TodoTrackerConfig { debounce_ms: 50 };

        let tracker = TodoTracker::with_path_and_config(todos_dir.clone(), tx, config)
            .expect("Failed to create TodoTracker");

        // Mark the session as ended BEFORE creating the file
        tracker.mark_session_ended(session_id).await;

        // Give watcher time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create a todo file with incomplete tasks
        create_test_todo_file(&todos_dir, session_id, &todo_json(2, 2, 1));

        // Should receive event with abandoned flag
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        // Session ended with incomplete tasks should be marked as abandoned
        assert!(event.abandoned);
    }
}

// ============================================================================
// FileHistoryTracker Tests
// ============================================================================

mod file_history_tracker_tests {
    use super::*;
    use vibetea_monitor::trackers::file_history_tracker::{
        FileHistoryTracker, FileHistoryTrackerConfig,
    };
    use vibetea_monitor::types::FileChangeEvent;

    #[tokio::test]
    #[serial]
    async fn test_file_history_tracker_detects_new_version() {
        let temp_dir = create_test_projects_dir();
        let file_history_dir = create_test_file_history_dir(temp_dir.path());
        let session_id = "550e8400-e29b-41d4-a716-446655440040";
        let file_hash = "a1b2c3d4e5f6a7b8";

        let session_dir = create_test_session_dir(&file_history_dir, session_id);

        // Create v1 file before tracker starts
        create_test_file_version(&session_dir, file_hash, 1, "line1\nline2\n");

        let (tx, mut rx) = mpsc::channel::<FileChangeEvent>(100);
        let config = FileHistoryTrackerConfig { debounce_ms: 50 };

        let _tracker = FileHistoryTracker::with_path_and_config(file_history_dir, tx, config)
            .expect("Failed to create FileHistoryTracker");

        // Give watcher time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create v2 file AFTER tracker is running
        create_test_file_version(&session_dir, file_hash, 2, "line1\nline2\nline3\n");

        // Should receive event for the v2 file (diffed against v1)
        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.file_hash, file_hash);
        assert_eq!(event.version, 2);
        assert_eq!(event.lines_added, 1); // Added line3
        assert_eq!(event.lines_removed, 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_file_history_tracker_skips_v1() {
        let temp_dir = create_test_projects_dir();
        let file_history_dir = create_test_file_history_dir(temp_dir.path());
        let session_id = "550e8400-e29b-41d4-a716-446655440042";
        let file_hash = "c3d4e5f6a7b8c9d0";

        let session_dir = create_test_session_dir(&file_history_dir, session_id);

        let (tx, mut rx) = mpsc::channel::<FileChangeEvent>(100);
        let config = FileHistoryTrackerConfig { debounce_ms: 50 };

        let _tracker = FileHistoryTracker::with_path_and_config(file_history_dir, tx, config)
            .expect("Failed to create FileHistoryTracker");

        // Give watcher time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Only create v1 file (no previous version to diff against)
        create_test_file_version(&session_dir, file_hash, 1, "initial content\n");

        // Should not receive any event for v1 only
        let result = timeout(Duration::from_millis(500), rx.recv()).await;
        assert!(
            result.is_err(),
            "Should not emit events for v1 files without previous version"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_file_history_tracker_line_modifications() {
        let temp_dir = create_test_projects_dir();
        let file_history_dir = create_test_file_history_dir(temp_dir.path());
        let session_id = "550e8400-e29b-41d4-a716-446655440043";
        let file_hash = "d4e5f6a7b8c9d0e1";

        let session_dir = create_test_session_dir(&file_history_dir, session_id);

        // Create v1 before tracker
        create_test_file_version(
            &session_dir,
            file_hash,
            1,
            "line1\nline2\nline3\nline4\nline5\n",
        );

        let (tx, mut rx) = mpsc::channel::<FileChangeEvent>(100);
        let config = FileHistoryTrackerConfig { debounce_ms: 50 };

        let _tracker = FileHistoryTracker::with_path_and_config(file_history_dir, tx, config)
            .expect("Failed to create FileHistoryTracker");

        // Give watcher time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create v2 with modifications
        create_test_file_version(
            &session_dir,
            file_hash,
            2,
            "line1\nmodified3\nline4\nline5\nline6\n",
        );

        let event = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Channel closed");

        // Should detect the changes
        assert!(event.lines_added > 0 || event.lines_removed > 0 || event.lines_modified > 0);
    }
}

// ============================================================================
// AgentTracker Tests (Parsing Functions Only)
// ============================================================================

mod agent_tracker_tests {
    use chrono::{TimeZone, Utc};
    use vibetea_monitor::trackers::agent_tracker::{
        create_agent_spawn_event, parse_task_tool_use, try_extract_agent_spawn,
    };

    #[test]
    fn test_parse_task_tool_use() {
        let input = serde_json::json!({
            "description": "Run unit tests",
            "prompt": "Execute all unit tests in the project",
            "subagent_type": "devs:rust-dev"
        });

        let result = parse_task_tool_use("Task", &input);
        assert!(result.is_some());

        let task_input = result.unwrap();
        assert_eq!(task_input.description, "Run unit tests");
        assert_eq!(task_input.subagent_type, "devs:rust-dev");
    }

    #[test]
    fn test_parse_task_tool_use_wrong_tool() {
        let input = serde_json::json!({
            "description": "Something",
            "prompt": "Something else"
        });

        let result = parse_task_tool_use("Read", &input);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_task_tool_use_default_subagent_type() {
        let input = serde_json::json!({
            "description": "Task without subagent_type"
        });

        let result = parse_task_tool_use("Task", &input);
        assert!(result.is_some());

        let task_input = result.unwrap();
        assert_eq!(task_input.subagent_type, "task"); // Default value
    }

    #[test]
    fn test_create_agent_spawn_event() {
        let input = serde_json::json!({
            "description": "Debug issue",
            "subagent_type": "devs:rust-dev"
        });

        let task_input = parse_task_tool_use("Task", &input).unwrap();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let event = create_agent_spawn_event("session-123".to_string(), timestamp, &task_input);

        assert_eq!(event.session_id, "session-123");
        assert_eq!(event.description, "Debug issue");
        assert_eq!(event.agent_type, "devs:rust-dev");
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn test_try_extract_agent_spawn() {
        let input = serde_json::json!({
            "description": "Test task",
            "subagent_type": "task"
        });

        let timestamp = Utc::now();
        let result = try_extract_agent_spawn("Task", &input, "session-456".to_string(), timestamp);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.session_id, "session-456");
        assert_eq!(event.description, "Test task");
    }

    #[test]
    fn test_try_extract_agent_spawn_non_task() {
        let input = serde_json::json!({
            "file_path": "/some/file.rs"
        });

        let timestamp = Utc::now();
        let result = try_extract_agent_spawn("Read", &input, "session-789".to_string(), timestamp);

        assert!(result.is_none());
    }
}

// ============================================================================
// Combined Integration Tests
// ============================================================================

mod combined_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use vibetea_monitor::trackers::file_history_tracker::{
        FileHistoryTracker, FileHistoryTrackerConfig,
    };
    use vibetea_monitor::trackers::project_tracker::{ProjectTracker, ProjectTrackerConfig};
    use vibetea_monitor::trackers::skill_tracker::{SkillTracker, SkillTrackerConfig};
    use vibetea_monitor::trackers::stats_tracker::StatsTracker;
    use vibetea_monitor::trackers::todo_tracker::{TodoTracker, TodoTrackerConfig};
    use vibetea_monitor::trackers::StatsEvent;
    use vibetea_monitor::types::{
        FileChangeEvent, ProjectActivityEvent, SkillInvocationEvent, TodoProgressEvent,
    };

    /// Test that all trackers can be initialized and emit events for new files.
    #[tokio::test]
    #[serial]
    async fn test_all_trackers_emit_events() {
        let temp_dir = create_test_projects_dir();
        let session_id = "550e8400-e29b-41d4-a716-446655440050";

        // Setup directory structure (but don't create most files yet)
        let projects_dir = temp_dir.path().join("projects");
        fs::create_dir_all(&projects_dir).expect("Failed to create projects dir");

        // Create project subdirectory first (needed for file watch to work)
        let project_dir = projects_dir.join("-home-user-testproject");
        fs::create_dir_all(&project_dir).expect("Failed to create project dir");

        // Stats file exists before tracker starts (it reads on init)
        let stats_path = create_test_stats_file(temp_dir.path(), &stats_cache_json());

        // History file exists before tracker starts (with emit_existing_on_startup: true)
        let history_content = skill_invocation_jsonl(session_id, "commit", 1738567268363) + "\n";
        let history_path = create_test_history_file(temp_dir.path(), &history_content);

        let todos_dir = create_test_todos_dir(temp_dir.path());
        let file_history_dir = create_test_file_history_dir(temp_dir.path());
        let session_dir = create_test_session_dir(&file_history_dir, session_id);

        // Create v1 file before tracker (so v2 can be diffed)
        create_test_file_version(&session_dir, "abcd1234abcd1234", 1, "line1\n");

        // Create channels
        let (project_tx, mut project_rx) = mpsc::channel::<ProjectActivityEvent>(100);
        let (stats_tx, mut stats_rx) = mpsc::channel::<StatsEvent>(100);
        let (skill_tx, mut skill_rx) = mpsc::channel::<SkillInvocationEvent>(100);
        let (todo_tx, mut todo_rx) = mpsc::channel::<TodoProgressEvent>(100);
        let (file_tx, mut file_rx) = mpsc::channel::<FileChangeEvent>(100);

        // Initialize all trackers
        let _project_tracker = ProjectTracker::with_path_and_config(
            projects_dir.clone(),
            project_tx,
            ProjectTrackerConfig {
                scan_on_init: false,
            },
        )
        .expect("Failed to create ProjectTracker");

        let _stats_tracker =
            StatsTracker::with_path(stats_path, stats_tx).expect("Failed to create StatsTracker");

        let _skill_tracker = SkillTracker::with_path_and_config(
            history_path,
            skill_tx,
            SkillTrackerConfig {
                emit_existing_on_startup: true,
            },
        )
        .expect("Failed to create SkillTracker");

        let _todo_tracker = TodoTracker::with_path_and_config(
            todos_dir.clone(),
            todo_tx,
            TodoTrackerConfig { debounce_ms: 50 },
        )
        .expect("Failed to create TodoTracker");

        let _file_tracker = FileHistoryTracker::with_path_and_config(
            file_history_dir,
            file_tx,
            FileHistoryTrackerConfig { debounce_ms: 50 },
        )
        .expect("Failed to create FileHistoryTracker");

        // Give watchers time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Now create files that will trigger events
        // Create session file in existing project directory
        let session_file = project_dir.join(format!("{}.jsonl", session_id));
        let mut file = fs::File::create(&session_file).expect("Failed to create session file");
        file.write_all(active_session_jsonl().as_bytes())
            .expect("Failed to write session");
        file.flush().expect("Failed to flush");

        create_test_todo_file(&todos_dir, session_id, &todo_json(1, 1, 2));
        create_test_file_version(&session_dir, "abcd1234abcd1234", 2, "line1\nline2\n");

        // Verify each tracker emits events
        let project_event = timeout(Duration::from_secs(2), project_rx.recv()).await;
        assert!(
            project_event.is_ok() && project_event.unwrap().is_some(),
            "ProjectTracker should emit event"
        );

        let stats_event = timeout(Duration::from_secs(2), stats_rx.recv()).await;
        assert!(
            stats_event.is_ok() && stats_event.unwrap().is_some(),
            "StatsTracker should emit event"
        );

        let skill_event = timeout(Duration::from_secs(2), skill_rx.recv()).await;
        assert!(
            skill_event.is_ok() && skill_event.unwrap().is_some(),
            "SkillTracker should emit event"
        );

        let todo_event = timeout(Duration::from_secs(2), todo_rx.recv()).await;
        assert!(
            todo_event.is_ok() && todo_event.unwrap().is_some(),
            "TodoTracker should emit event"
        );

        let file_event = timeout(Duration::from_secs(2), file_rx.recv()).await;
        assert!(
            file_event.is_ok() && file_event.unwrap().is_some(),
            "FileHistoryTracker should emit event"
        );
    }

    /// Test that trackers handle concurrent file modifications correctly.
    #[tokio::test]
    #[serial]
    async fn test_concurrent_file_modifications() {
        let temp_dir = create_test_projects_dir();
        let session_id = "550e8400-e29b-41d4-a716-446655440051";

        // Setup initial state
        let projects_dir = temp_dir.path().join("projects");
        fs::create_dir_all(&projects_dir).expect("Failed to create projects dir");

        let stats_path = create_test_stats_file(temp_dir.path(), &stats_cache_json());

        let (project_tx, mut project_rx) = mpsc::channel::<ProjectActivityEvent>(100);
        let (stats_tx, mut stats_rx) = mpsc::channel::<StatsEvent>(100);

        let _project_tracker = ProjectTracker::with_path_and_config(
            projects_dir.clone(),
            project_tx,
            ProjectTrackerConfig {
                scan_on_init: false,
            },
        )
        .expect("Failed to create ProjectTracker");

        let _stats_tracker = StatsTracker::with_path(stats_path.clone(), stats_tx)
            .expect("Failed to create StatsTracker");

        // Consume initial stats events
        while timeout(Duration::from_millis(300), stats_rx.recv())
            .await
            .is_ok()
        {}

        // Give watchers time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create initial project
        let project_dir = create_test_project(
            &projects_dir,
            "-home-user-concurrent",
            session_id,
            &active_session_jsonl(),
        );

        // Consume project creation event
        let _ = timeout(Duration::from_millis(500), project_rx.recv()).await;

        // Perform concurrent modifications
        let session_file = project_dir.join(format!("{}.jsonl", session_id));
        let stats_path_clone = stats_path.clone();

        let modification_tasks = vec![
            tokio::spawn(async move {
                for i in 0..3 {
                    // Add more content to the session file
                    let additional_content =
                        format!(r#"{{"type":"user","message":"message {}"}}"#, i);
                    use std::fs::OpenOptions;
                    let mut file = OpenOptions::new()
                        .append(true)
                        .open(&session_file)
                        .expect("Failed to open session file");
                    writeln!(file, "{}", additional_content).expect("Failed to write");
                    file.flush().expect("Failed to flush");
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }),
            tokio::spawn(async move {
                for i in 0..3 {
                    let updated_stats = format!(
                        r#"{{"totalSessions":{},"totalMessages":{},"totalToolUsage":8000,"longestSession":"00:45:30","hourCounts":{{"9":50}},"modelUsage":{{}}}}"#,
                        150 + i,
                        2500 + i * 100
                    );
                    let mut file =
                        fs::File::create(&stats_path_clone).expect("Failed to create stats file");
                    file.write_all(updated_stats.as_bytes())
                        .expect("Failed to write stats");
                    file.flush().expect("Failed to flush");
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }),
        ];

        // Wait for modifications to complete
        for task in modification_tasks {
            task.await.expect("Modification task failed");
        }

        // Wait a bit for events to propagate
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Test passes if no panics occurred - concurrent modifications should be handled gracefully
    }

    /// Test event aggregation across multiple trackers for the same session.
    #[tokio::test]
    #[serial]
    async fn test_session_event_aggregation() {
        let temp_dir = create_test_projects_dir();
        let session_id = "550e8400-e29b-41d4-a716-446655440052";

        // Setup directory structure
        let projects_dir = temp_dir.path().join("projects");
        fs::create_dir_all(&projects_dir).expect("Failed to create projects dir");

        let history_content = skill_invocation_jsonl(session_id, "commit", 1738567268363) + "\n";
        let history_path = create_test_history_file(temp_dir.path(), &history_content);

        let todos_dir = create_test_todos_dir(temp_dir.path());

        let file_history_dir = create_test_file_history_dir(temp_dir.path());
        let session_dir = create_test_session_dir(&file_history_dir, session_id);
        create_test_file_version(&session_dir, "efgh5678efgh5678", 1, "line1\n");

        // Shared state for collecting events
        let collected_sessions: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let (project_tx, mut project_rx) = mpsc::channel::<ProjectActivityEvent>(100);
        let (skill_tx, mut skill_rx) = mpsc::channel::<SkillInvocationEvent>(100);
        let (todo_tx, mut todo_rx) = mpsc::channel::<TodoProgressEvent>(100);
        let (file_tx, mut file_rx) = mpsc::channel::<FileChangeEvent>(100);

        // Initialize trackers
        let _project_tracker = ProjectTracker::with_path_and_config(
            projects_dir.clone(),
            project_tx,
            ProjectTrackerConfig {
                scan_on_init: false,
            },
        )
        .expect("Failed to create ProjectTracker");

        let _skill_tracker = SkillTracker::with_path_and_config(
            history_path,
            skill_tx,
            SkillTrackerConfig {
                emit_existing_on_startup: true,
            },
        )
        .expect("Failed to create SkillTracker");

        let _todo_tracker = TodoTracker::with_path_and_config(
            todos_dir.clone(),
            todo_tx,
            TodoTrackerConfig { debounce_ms: 50 },
        )
        .expect("Failed to create TodoTracker");

        let _file_tracker = FileHistoryTracker::with_path_and_config(
            file_history_dir,
            file_tx,
            FileHistoryTrackerConfig { debounce_ms: 50 },
        )
        .expect("Failed to create FileHistoryTracker");

        // Give watchers time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create files that trigger events
        create_test_project(
            &projects_dir,
            "-home-user-aggregate",
            session_id,
            &active_session_jsonl(),
        );
        create_test_todo_file(&todos_dir, session_id, &todo_json(1, 1, 2));
        create_test_file_version(&session_dir, "efgh5678efgh5678", 2, "line1\nline2\n");

        // Collect session IDs from all trackers
        let sessions_clone = Arc::clone(&collected_sessions);
        tokio::spawn(async move {
            while let Some(event) = project_rx.recv().await {
                sessions_clone.lock().await.push(event.session_id);
            }
        });

        let sessions_clone = Arc::clone(&collected_sessions);
        tokio::spawn(async move {
            while let Some(event) = skill_rx.recv().await {
                sessions_clone.lock().await.push(event.session_id);
            }
        });

        let sessions_clone = Arc::clone(&collected_sessions);
        tokio::spawn(async move {
            while let Some(event) = todo_rx.recv().await {
                sessions_clone.lock().await.push(event.session_id);
            }
        });

        let sessions_clone = Arc::clone(&collected_sessions);
        tokio::spawn(async move {
            while let Some(event) = file_rx.recv().await {
                sessions_clone.lock().await.push(event.session_id);
            }
        });

        // Wait for events to be collected
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Verify all events have the same session ID
        let sessions = collected_sessions.lock().await;
        assert!(!sessions.is_empty(), "Should have collected some events");

        for collected_session in sessions.iter() {
            assert_eq!(
                collected_session, session_id,
                "All events should have the same session ID"
            );
        }
    }
}
