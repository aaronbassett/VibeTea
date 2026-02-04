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
/// let path = parse_project_slug("-home-user-code-my-project");
/// assert_eq!(path, "/home/user/code/my-project");
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
}
