//! Claude Code JSONL parser for VibeTea Monitor.
//!
//! This module provides parsing functionality for Claude Code session files,
//! extracting structured events from the JSONL format. The parser implements
//! a privacy-first approach, extracting only metadata (tool names, timestamps,
//! file basenames) and never processing actual code content or prompts.
//!
//! # Claude Code Event Format
//!
//! Claude Code writes session data as JSONL (JSON Lines) files at:
//! `~/.claude/projects/<slugified-path>/<uuid>.jsonl`
//!
//! Each line contains a JSON object with a `type` field indicating the event type.
//!
//! # Event Mapping
//!
//! | Claude Code Type | VibeTea Event | Fields Extracted |
//! |------------------|---------------|------------------|
//! | `assistant` with `tool_use` | Tool started | tool name, context |
//! | `progress` with `PostToolUse` | Tool completed | tool name, success |
//! | `user` | Activity | timestamp only |
//! | `summary` | Summary | marks session end |
//! | First event in file | Session started | project from path |
//!
//! # Example Usage
//!
//! ```ignore
//! use vibetea_monitor::parser::{SessionParser, parse_line};
//!
//! let path = "~/.claude/projects/-home-user-my-project/abc123.jsonl";
//! let mut parser = SessionParser::from_path(path).unwrap();
//!
//! // Parse a single line
//! let line = r#"{"type":"user","timestamp":"2026-01-15T10:00:00Z"}"#;
//! if let Some(event) = parser.parse_line(line) {
//!     println!("Parsed event: {:?}", event.kind);
//! }
//! ```

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use thiserror::Error;
use tracing::warn;
use uuid::Uuid;

/// Errors that can occur during parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    /// Failed to parse JSON.
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// Invalid file path format.
    #[error("invalid path format: {0}")]
    InvalidPath(String),

    /// Failed to parse UUID from filename.
    #[error("invalid session ID in filename: {0}")]
    InvalidSessionId(String),
}

/// A parsed event from Claude Code JSONL.
///
/// Represents a normalized event extracted from the raw Claude Code format,
/// containing only the metadata needed for VibeTea processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEvent {
    /// The type of event that was parsed.
    pub kind: ParsedEventKind,

    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
}

/// The kind of parsed event.
///
/// Each variant represents a different type of activity detected in the
/// Claude Code session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedEventKind {
    /// A tool invocation has started.
    ToolStarted {
        /// The name of the tool (e.g., "Read", "Bash", "Edit").
        name: String,
        /// Optional context, typically a file basename.
        context: Option<String>,
    },

    /// A tool invocation has completed.
    ToolCompleted {
        /// The name of the tool.
        name: String,
        /// Whether the tool completed successfully.
        success: bool,
        /// Optional context, typically a file basename.
        context: Option<String>,
    },

    /// User activity was detected (indicates an active session).
    Activity,

    /// A session summary, marking the end of the session.
    Summary,

    /// A new session has started.
    SessionStarted {
        /// The project name extracted from the file path.
        project: String,
    },
}

/// Raw Claude Code event structure for deserialization.
///
/// This struct captures the top-level fields from Claude Code JSONL events.
/// Only the fields needed for event extraction are included.
#[derive(Debug, Deserialize)]
pub struct RawClaudeEvent {
    /// The event type (e.g., "assistant", "user", "progress", "summary").
    #[serde(rename = "type")]
    pub event_type: String,

    /// The message content for assistant/user events.
    #[serde(default)]
    pub message: Option<RawMessage>,

    /// Progress data for progress events.
    #[serde(default)]
    pub progress: Option<ProgressData>,

    /// Event timestamp (RFC 3339 format).
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
}

/// Raw message structure containing content blocks.
#[derive(Debug, Deserialize)]
pub struct RawMessage {
    /// The content blocks in the message.
    #[serde(default)]
    pub content: Vec<ContentBlock>,
}

/// A content block within a message.
///
/// Content blocks can be text, tool use, or tool results.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content (ignored for privacy).
    Text {
        /// We don't extract text content for privacy.
        #[serde(skip)]
        _text: (),
    },

    /// A tool use block indicating a tool invocation.
    ToolUse {
        /// The name of the tool being invoked.
        name: String,
        /// Tool input parameters.
        #[serde(default)]
        input: serde_json::Value,
    },

    /// A tool result block (ignored, we track completion via progress).
    ToolResult {
        /// We don't extract tool results for privacy.
        #[serde(skip)]
        _result: (),
    },

    /// Thinking content (ignored for privacy).
    Thinking {
        /// We don't extract thinking content for privacy.
        #[serde(skip)]
        _thinking: (),
    },
}

/// Progress data for tracking tool completion.
#[derive(Debug, Deserialize)]
pub struct ProgressData {
    /// The type of progress event.
    #[serde(rename = "type")]
    pub progress_type: Option<String>,

    /// The name of the tool (for PostToolUse events).
    #[serde(default)]
    pub tool_name: Option<String>,

    /// The result of the tool invocation.
    #[serde(default)]
    pub result: Option<ToolResult>,
}

/// Tool execution result.
#[derive(Debug, Deserialize)]
pub struct ToolResult {
    /// Whether the tool execution was successful.
    #[serde(default)]
    pub success: Option<bool>,

    /// Error message if the tool failed.
    #[serde(default)]
    pub error: Option<String>,
}

/// Session parser state for processing Claude Code JSONL files.
///
/// Maintains state across multiple lines of a session file, tracking whether
/// this is the first event (to emit session started) and the session metadata.
///
/// # Example
///
/// ```ignore
/// let mut parser = SessionParser::from_path(
///     "/home/user/.claude/projects/-home-user-project/abc123.jsonl"
/// ).unwrap();
///
/// // First event will include SessionStarted
/// let events = parser.parse_line(r#"{"type":"user"}"#);
/// ```
#[derive(Debug)]
pub struct SessionParser {
    /// The session ID extracted from the filename.
    session_id: Uuid,

    /// The project name extracted from the path.
    project: String,

    /// Whether this is the first event being parsed.
    is_first_event: bool,
}

impl SessionParser {
    /// Creates a new `SessionParser` from a file path.
    ///
    /// Extracts the session ID from the filename (expected to be a UUID)
    /// and the project name from the parent directory.
    ///
    /// # Path Format
    ///
    /// Expected format: `~/.claude/projects/<slugified-path>/<uuid>.jsonl`
    ///
    /// - The `<slugified-path>` becomes the project name (URL decoded)
    /// - The `<uuid>` becomes the session ID
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSONL session file
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - The path has no filename
    /// - The filename is not a valid UUID
    /// - The path has no parent directory
    ///
    /// # Example
    ///
    /// ```ignore
    /// let parser = SessionParser::from_path(
    ///     "/home/user/.claude/projects/-home-user-my--project/a1b2c3d4-e5f6-7890-abcd-ef1234567890.jsonl"
    /// ).unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        let path = path.as_ref();

        // Extract filename and parse UUID
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| ParseError::InvalidPath("no filename".to_string()))?;

        let session_id = Uuid::parse_str(filename)
            .map_err(|_| ParseError::InvalidSessionId(filename.to_string()))?;

        // Extract project name from parent directory
        let project = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(decode_project_name)
            .ok_or_else(|| ParseError::InvalidPath("no parent directory".to_string()))?;

        Ok(Self {
            session_id,
            project,
            is_first_event: true,
        })
    }

    /// Creates a new `SessionParser` with explicit values.
    ///
    /// Useful for testing or when the session ID and project are known.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session UUID
    /// * `project` - The project name
    #[must_use]
    pub fn new(session_id: Uuid, project: String) -> Self {
        Self {
            session_id,
            project,
            is_first_event: true,
        }
    }

    /// Returns the session ID for this parser.
    #[must_use]
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Returns the project name for this parser.
    #[must_use]
    pub fn project(&self) -> &str {
        &self.project
    }

    /// Parses a single JSONL line, returning any extracted events.
    ///
    /// The first successful parse will also emit a `SessionStarted` event.
    /// Malformed JSON lines are logged as warnings and skipped.
    ///
    /// # Arguments
    ///
    /// * `line` - A single line from the JSONL file
    ///
    /// # Returns
    ///
    /// A vector of parsed events (may be empty for unrecognized events,
    /// or contain multiple events for the first line of a session).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut parser = SessionParser::new(Uuid::new_v4(), "my-project".to_string());
    ///
    /// // First line returns SessionStarted + the parsed event
    /// let events = parser.parse_line(r#"{"type":"user","timestamp":"2026-01-15T10:00:00Z"}"#);
    /// assert_eq!(events.len(), 2);
    /// ```
    pub fn parse_line(&mut self, line: &str) -> Vec<ParsedEvent> {
        let line = line.trim();
        if line.is_empty() {
            return Vec::new();
        }

        // Parse the raw event
        let raw_event: RawClaudeEvent = match serde_json::from_str(line) {
            Ok(event) => event,
            Err(e) => {
                warn!("Failed to parse JSONL line: {}", e);
                return Vec::new();
            }
        };

        let mut events = Vec::new();

        // Get timestamp, defaulting to now if not present
        let timestamp = raw_event.timestamp.unwrap_or_else(Utc::now);

        // Emit session started on first event
        if self.is_first_event {
            self.is_first_event = false;
            events.push(ParsedEvent {
                kind: ParsedEventKind::SessionStarted {
                    project: self.project.clone(),
                },
                timestamp,
            });
        }

        // Parse based on event type
        if let Some(event) = self.parse_event(&raw_event, timestamp) {
            events.push(event);
        }

        events
    }

    /// Parses a raw Claude Code event into a `ParsedEvent`.
    fn parse_event(&self, raw: &RawClaudeEvent, timestamp: DateTime<Utc>) -> Option<ParsedEvent> {
        match raw.event_type.as_str() {
            "assistant" => self.parse_assistant_event(raw, timestamp),
            "user" => Some(ParsedEvent {
                kind: ParsedEventKind::Activity,
                timestamp,
            }),
            "progress" => self.parse_progress_event(raw, timestamp),
            "summary" => Some(ParsedEvent {
                kind: ParsedEventKind::Summary,
                timestamp,
            }),
            _ => None,
        }
    }

    /// Parses an assistant event, looking for tool_use content blocks.
    fn parse_assistant_event(
        &self,
        raw: &RawClaudeEvent,
        timestamp: DateTime<Utc>,
    ) -> Option<ParsedEvent> {
        let message = raw.message.as_ref()?;

        // Find the first tool_use block
        for block in &message.content {
            if let ContentBlock::ToolUse { name, input } = block {
                let context = extract_context_from_input(input);
                return Some(ParsedEvent {
                    kind: ParsedEventKind::ToolStarted {
                        name: name.clone(),
                        context,
                    },
                    timestamp,
                });
            }
        }

        None
    }

    /// Parses a progress event, looking for PostToolUse data.
    fn parse_progress_event(
        &self,
        raw: &RawClaudeEvent,
        timestamp: DateTime<Utc>,
    ) -> Option<ParsedEvent> {
        let progress = raw.progress.as_ref()?;

        // Check if this is a PostToolUse progress event
        if progress.progress_type.as_deref() != Some("PostToolUse") {
            return None;
        }

        let tool_name = progress.tool_name.as_ref()?;

        // Determine success status
        let success = progress
            .result
            .as_ref()
            .and_then(|r| r.success)
            .unwrap_or(true); // Default to true if not specified

        Some(ParsedEvent {
            kind: ParsedEventKind::ToolCompleted {
                name: tool_name.clone(),
                success,
                context: None, // Context is extracted at tool start, not completion
            },
            timestamp,
        })
    }
}

/// Extracts a file basename context from tool input parameters.
///
/// Looks for common path-containing fields in tool inputs and extracts
/// just the basename (privacy: never transmit full paths).
fn extract_context_from_input(input: &serde_json::Value) -> Option<String> {
    // Common field names that might contain file paths
    const PATH_FIELDS: &[&str] = &["file_path", "path", "filename", "file", "notebook_path"];

    if let Some(obj) = input.as_object() {
        for field in PATH_FIELDS {
            if let Some(serde_json::Value::String(path)) = obj.get(*field) {
                return extract_basename(path);
            }
        }
    }

    None
}

/// Extracts the basename from a file path.
///
/// Returns `None` if the path is empty or has no valid basename.
fn extract_basename(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// Decodes a slugified project name.
///
/// Claude Code slugifies project paths by:
/// - Replacing `/` with `-`
/// - URL-encoding special characters
///
/// This function reverses common encodings.
/// The leading dash represents the root `/`.
fn decode_project_name(slugified: &str) -> String {
    urlencoding_decode(slugified)
}

/// Simple URL decoding for common sequences.
///
/// Handles the most common percent-encoded characters.
fn urlencoding_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Try to parse the next two characters as a hex code
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            // If parsing failed, keep the original
            result.push('%');
            result.push_str(&hex);
        } else {
            result.push(c);
        }
    }

    result
}

/// Parses a single JSONL line without session context.
///
/// This is a convenience function for parsing individual lines when
/// session tracking is not needed.
///
/// # Arguments
///
/// * `line` - A single line from a JSONL file
///
/// # Returns
///
/// `Some(ParsedEvent)` if the line contains a recognized event,
/// `None` for unrecognized or malformed lines.
///
/// # Example
///
/// ```ignore
/// let line = r#"{"type":"summary","timestamp":"2026-01-15T10:00:00Z"}"#;
/// if let Some(event) = parse_line(line) {
///     assert!(matches!(event.kind, ParsedEventKind::Summary));
/// }
/// ```
pub fn parse_line(line: &str) -> Option<ParsedEvent> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let raw_event: RawClaudeEvent = serde_json::from_str(line).ok()?;
    let timestamp = raw_event.timestamp.unwrap_or_else(Utc::now);

    // Create a temporary parser for single-line parsing
    let parser = SessionParser::new(Uuid::nil(), String::new());
    parser.parse_event(&raw_event, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Path Parsing Tests ====================

    #[test]
    fn from_path_extracts_session_id_and_project() {
        let path = "/home/user/.claude/projects/-home-user-my-project/a1b2c3d4-e5f6-7890-abcd-ef1234567890.jsonl";
        let parser = SessionParser::from_path(path).unwrap();

        assert_eq!(
            parser.session_id(),
            Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap()
        );
        assert_eq!(parser.project(), "-home-user-my-project");
    }

    #[test]
    fn from_path_handles_url_encoded_project_name() {
        let path = "/home/user/.claude/projects/-home-user-my%20project/a1b2c3d4-e5f6-7890-abcd-ef1234567890.jsonl";
        let parser = SessionParser::from_path(path).unwrap();

        assert_eq!(parser.project(), "-home-user-my project");
    }

    #[test]
    fn from_path_fails_on_invalid_uuid() {
        let path = "/home/user/.claude/projects/-home-user-project/not-a-uuid.jsonl";
        let result = SessionParser::from_path(path);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidSessionId(_)
        ));
    }

    #[test]
    fn from_path_fails_on_missing_filename() {
        let path = "/home/user/.claude/projects/-home-user-project/";
        let result = SessionParser::from_path(path);

        assert!(result.is_err());
    }

    // ==================== Tool Use Parsing Tests ====================

    #[test]
    fn parse_line_extracts_tool_use_from_assistant() {
        let line = r#"{
            "type": "assistant",
            "timestamp": "2026-01-15T10:00:00Z",
            "message": {
                "content": [
                    {"type": "tool_use", "name": "Read", "input": {"file_path": "/home/user/project/src/main.rs"}}
                ]
            }
        }"#;

        let event = parse_line(line).unwrap();

        match event.kind {
            ParsedEventKind::ToolStarted { name, context } => {
                assert_eq!(name, "Read");
                assert_eq!(context, Some("main.rs".to_string()));
            }
            _ => panic!("Expected ToolStarted event"),
        }
    }

    #[test]
    fn parse_line_extracts_tool_name_without_path() {
        let line = r#"{
            "type": "assistant",
            "timestamp": "2026-01-15T10:00:00Z",
            "message": {
                "content": [
                    {"type": "tool_use", "name": "Bash", "input": {"command": "ls -la"}}
                ]
            }
        }"#;

        let event = parse_line(line).unwrap();

        match event.kind {
            ParsedEventKind::ToolStarted { name, context } => {
                assert_eq!(name, "Bash");
                assert_eq!(context, None);
            }
            _ => panic!("Expected ToolStarted event"),
        }
    }

    #[test]
    fn parse_line_handles_multiple_content_blocks() {
        let line = r#"{
            "type": "assistant",
            "timestamp": "2026-01-15T10:00:00Z",
            "message": {
                "content": [
                    {"type": "text", "text": "Let me read that file."},
                    {"type": "tool_use", "name": "Read", "input": {"file_path": "/path/to/file.rs"}}
                ]
            }
        }"#;

        let event = parse_line(line).unwrap();

        match event.kind {
            ParsedEventKind::ToolStarted { name, .. } => {
                assert_eq!(name, "Read");
            }
            _ => panic!("Expected ToolStarted event"),
        }
    }

    // ==================== Progress/PostToolUse Tests ====================

    #[test]
    fn parse_line_extracts_post_tool_use_success() {
        let line = r#"{
            "type": "progress",
            "timestamp": "2026-01-15T10:00:01Z",
            "progress": {
                "type": "PostToolUse",
                "tool_name": "Read",
                "result": {"success": true}
            }
        }"#;

        let event = parse_line(line).unwrap();

        match event.kind {
            ParsedEventKind::ToolCompleted { name, success, .. } => {
                assert_eq!(name, "Read");
                assert!(success);
            }
            _ => panic!("Expected ToolCompleted event"),
        }
    }

    #[test]
    fn parse_line_extracts_post_tool_use_failure() {
        let line = r#"{
            "type": "progress",
            "timestamp": "2026-01-15T10:00:01Z",
            "progress": {
                "type": "PostToolUse",
                "tool_name": "Bash",
                "result": {"success": false, "error": "Command failed"}
            }
        }"#;

        let event = parse_line(line).unwrap();

        match event.kind {
            ParsedEventKind::ToolCompleted { name, success, .. } => {
                assert_eq!(name, "Bash");
                assert!(!success);
            }
            _ => panic!("Expected ToolCompleted event"),
        }
    }

    #[test]
    fn parse_line_ignores_non_post_tool_use_progress() {
        let line = r#"{
            "type": "progress",
            "timestamp": "2026-01-15T10:00:01Z",
            "progress": {
                "type": "Streaming",
                "data": "some data"
            }
        }"#;

        let event = parse_line(line);
        assert!(event.is_none());
    }

    // ==================== User Event Tests ====================

    #[test]
    fn parse_line_extracts_user_as_activity() {
        let line = r#"{
            "type": "user",
            "timestamp": "2026-01-15T10:00:00Z"
        }"#;

        let event = parse_line(line).unwrap();
        assert!(matches!(event.kind, ParsedEventKind::Activity));
    }

    // ==================== Summary Event Tests ====================

    #[test]
    fn parse_line_extracts_summary() {
        let line = r#"{
            "type": "summary",
            "timestamp": "2026-01-15T11:00:00Z"
        }"#;

        let event = parse_line(line).unwrap();
        assert!(matches!(event.kind, ParsedEventKind::Summary));
    }

    // ==================== Session Parser Tests ====================

    #[test]
    fn session_parser_emits_session_started_on_first_event() {
        let mut parser = SessionParser::new(
            Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
            "my-project".to_string(),
        );

        let line = r#"{"type": "user", "timestamp": "2026-01-15T10:00:00Z"}"#;
        let events = parser.parse_line(line);

        assert_eq!(events.len(), 2);

        // First event should be SessionStarted
        match &events[0].kind {
            ParsedEventKind::SessionStarted { project } => {
                assert_eq!(project, "my-project");
            }
            _ => panic!("Expected SessionStarted as first event"),
        }

        // Second event should be Activity
        assert!(matches!(events[1].kind, ParsedEventKind::Activity));
    }

    #[test]
    fn session_parser_only_emits_session_started_once() {
        let mut parser = SessionParser::new(Uuid::new_v4(), "my-project".to_string());

        // First line
        let events1 = parser.parse_line(r#"{"type": "user", "timestamp": "2026-01-15T10:00:00Z"}"#);
        assert_eq!(events1.len(), 2);

        // Second line
        let events2 = parser.parse_line(r#"{"type": "user", "timestamp": "2026-01-15T10:01:00Z"}"#);
        assert_eq!(events2.len(), 1);
        assert!(matches!(events2[0].kind, ParsedEventKind::Activity));
    }

    #[test]
    fn session_parser_summary_marks_session_end() {
        let mut parser = SessionParser::new(Uuid::new_v4(), "my-project".to_string());

        // Skip first event handling
        let _ = parser.parse_line(r#"{"type": "user"}"#);

        let events =
            parser.parse_line(r#"{"type": "summary", "timestamp": "2026-01-15T11:00:00Z"}"#);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].kind, ParsedEventKind::Summary));
    }

    // ==================== Malformed JSON Tests ====================

    #[test]
    fn parse_line_skips_malformed_json() {
        let line = "{ this is not valid json }";
        let event = parse_line(line);
        assert!(event.is_none());
    }

    #[test]
    fn session_parser_skips_malformed_json() {
        let mut parser = SessionParser::new(Uuid::new_v4(), "my-project".to_string());

        let events = parser.parse_line("{ this is not valid json }");
        assert!(events.is_empty());

        // Should still be first event after skipped line
        assert!(parser.is_first_event);
    }

    #[test]
    fn parse_line_skips_empty_lines() {
        assert!(parse_line("").is_none());
        assert!(parse_line("   ").is_none());
        assert!(parse_line("\n").is_none());
    }

    #[test]
    fn session_parser_skips_empty_lines() {
        let mut parser = SessionParser::new(Uuid::new_v4(), "my-project".to_string());

        let events = parser.parse_line("");
        assert!(events.is_empty());

        // Should still be first event
        assert!(parser.is_first_event);
    }

    // ==================== Context Extraction Tests ====================

    #[test]
    fn extract_context_from_file_path() {
        let input = serde_json::json!({
            "file_path": "/home/user/project/src/lib.rs"
        });

        let context = extract_context_from_input(&input);
        assert_eq!(context, Some("lib.rs".to_string()));
    }

    #[test]
    fn extract_context_from_path_field() {
        let input = serde_json::json!({
            "path": "/some/directory"
        });

        let context = extract_context_from_input(&input);
        assert_eq!(context, Some("directory".to_string()));
    }

    #[test]
    fn extract_context_from_notebook_path() {
        let input = serde_json::json!({
            "notebook_path": "/home/user/notebooks/analysis.ipynb"
        });

        let context = extract_context_from_input(&input);
        assert_eq!(context, Some("analysis.ipynb".to_string()));
    }

    #[test]
    fn extract_context_returns_none_for_no_path() {
        let input = serde_json::json!({
            "command": "ls -la",
            "description": "List files"
        });

        let context = extract_context_from_input(&input);
        assert!(context.is_none());
    }

    // ==================== Timestamp Tests ====================

    #[test]
    fn parse_line_preserves_timestamp() {
        let line = r#"{"type": "user", "timestamp": "2026-01-15T10:30:00Z"}"#;
        let event = parse_line(line).unwrap();

        let expected = DateTime::parse_from_rfc3339("2026-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(event.timestamp, expected);
    }

    #[test]
    fn parse_line_defaults_to_now_without_timestamp() {
        let line = r#"{"type": "user"}"#;
        let before = Utc::now();
        let event = parse_line(line).unwrap();
        let after = Utc::now();

        assert!(event.timestamp >= before);
        assert!(event.timestamp <= after);
    }

    // ==================== URL Decoding Tests ====================

    #[test]
    fn urlencoding_decode_handles_spaces() {
        assert_eq!(urlencoding_decode("hello%20world"), "hello world");
    }

    #[test]
    fn urlencoding_decode_handles_multiple_encodings() {
        assert_eq!(urlencoding_decode("a%20b%2Fc"), "a b/c");
    }

    #[test]
    fn urlencoding_decode_passes_through_plain_text() {
        assert_eq!(urlencoding_decode("hello-world"), "hello-world");
    }

    #[test]
    fn urlencoding_decode_handles_invalid_sequences() {
        // Invalid hex should be preserved
        assert_eq!(urlencoding_decode("a%ZZb"), "a%ZZb");
    }

    // ==================== Known Tool Names Tests ====================

    #[test]
    fn parse_line_handles_common_tool_names() {
        let tools = [
            "Read",
            "Bash",
            "Edit",
            "Glob",
            "Grep",
            "Write",
            "NotebookEdit",
            "WebFetch",
        ];

        for tool in tools {
            let line = format!(
                r#"{{"type": "assistant", "message": {{"content": [{{"type": "tool_use", "name": "{}", "input": {{}}}}]}}}}"#,
                tool
            );

            let event = parse_line(&line).unwrap();
            match event.kind {
                ParsedEventKind::ToolStarted { name, .. } => {
                    assert_eq!(name, tool);
                }
                _ => panic!("Expected ToolStarted for tool {}", tool),
            }
        }
    }

    // ==================== Edge Cases ====================

    #[test]
    fn parse_line_ignores_unknown_event_types() {
        let line = r#"{"type": "unknown_type", "timestamp": "2026-01-15T10:00:00Z"}"#;
        let event = parse_line(line);
        assert!(event.is_none());
    }

    #[test]
    fn parse_line_handles_assistant_without_tool_use() {
        let line = r#"{
            "type": "assistant",
            "timestamp": "2026-01-15T10:00:00Z",
            "message": {
                "content": [
                    {"type": "text", "text": "Just some text response"}
                ]
            }
        }"#;

        let event = parse_line(line);
        assert!(event.is_none());
    }

    #[test]
    fn parse_line_handles_assistant_with_empty_content() {
        let line = r#"{
            "type": "assistant",
            "timestamp": "2026-01-15T10:00:00Z",
            "message": {
                "content": []
            }
        }"#;

        let event = parse_line(line);
        assert!(event.is_none());
    }

    #[test]
    fn parse_line_handles_assistant_without_message() {
        let line = r#"{
            "type": "assistant",
            "timestamp": "2026-01-15T10:00:00Z"
        }"#;

        let event = parse_line(line);
        assert!(event.is_none());
    }

    #[test]
    fn parse_line_handles_progress_without_result() {
        let line = r#"{
            "type": "progress",
            "timestamp": "2026-01-15T10:00:01Z",
            "progress": {
                "type": "PostToolUse",
                "tool_name": "Read"
            }
        }"#;

        let event = parse_line(line).unwrap();

        match event.kind {
            ParsedEventKind::ToolCompleted { name, success, .. } => {
                assert_eq!(name, "Read");
                // Should default to true when result is missing
                assert!(success);
            }
            _ => panic!("Expected ToolCompleted event"),
        }
    }

    // ==================== Integration-style Tests ====================

    #[test]
    fn full_session_lifecycle() {
        let mut parser = SessionParser::from_path(
            "/home/user/.claude/projects/-home-user-vibetea/a1b2c3d4-e5f6-7890-abcd-ef1234567890.jsonl",
        )
        .unwrap();

        // First user message (session starts)
        let events = parser.parse_line(r#"{"type": "user", "timestamp": "2026-01-15T10:00:00Z"}"#);
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0].kind,
            ParsedEventKind::SessionStarted { .. }
        ));
        assert!(matches!(events[1].kind, ParsedEventKind::Activity));

        // Tool use
        let events = parser.parse_line(r#"{"type": "assistant", "timestamp": "2026-01-15T10:00:01Z", "message": {"content": [{"type": "tool_use", "name": "Read", "input": {"file_path": "/path/to/file.rs"}}]}}"#);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].kind,
            ParsedEventKind::ToolStarted { .. }
        ));

        // Tool completion
        let events = parser.parse_line(r#"{"type": "progress", "timestamp": "2026-01-15T10:00:02Z", "progress": {"type": "PostToolUse", "tool_name": "Read", "result": {"success": true}}}"#);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].kind,
            ParsedEventKind::ToolCompleted { .. }
        ));

        // Session end
        let events =
            parser.parse_line(r#"{"type": "summary", "timestamp": "2026-01-15T11:00:00Z"}"#);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].kind, ParsedEventKind::Summary));
    }
}
