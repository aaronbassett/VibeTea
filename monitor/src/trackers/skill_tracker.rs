//! Skill tracker for detecting skill/slash command invocations.
//!
//! This module parses `history.jsonl` entries to extract [`SkillInvocationEvent`]
//! data from Claude Code skill invocations.
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
//! This module provides parsing functions for history.jsonl entries. The actual
//! file watching implementation will be added in a later task (T098).
//!
//! # Example
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

use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use thiserror::Error;

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
pub fn parse_history_entry(line: &str) -> Result<HistoryEntry, HistoryParseError> {
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
}
