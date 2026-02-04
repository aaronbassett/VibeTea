//! Agent tracker for detecting Task tool agent spawns.
//!
//! This module extracts [`AgentSpawnEvent`] data from Task tool invocations
//! in Claude Code session JSONL files.
//!
//! # Task Tool Format
//!
//! When Claude Code spawns a subagent using the Task tool, the JSONL contains:
//!
//! ```json
//! {
//!   "type": "assistant",
//!   "message": {
//!     "content": [
//!       {
//!         "type": "tool_use",
//!         "id": "toolu_01Fw1HCjXzYHNtWX7jXWzgBj",
//!         "name": "Task",
//!         "input": {
//!           "description": "Create SmileError enum",
//!           "prompt": "Create the SmileError enum...",
//!           "subagent_type": "devs:rust-dev"
//!         }
//!       }
//!     ]
//!   },
//!   "timestamp": "2026-02-03T05:01:57.678Z"
//! }
//! ```
//!
//! # Privacy
//!
//! This module follows the privacy-first principle: only the `subagent_type`
//! (as agent_type) and `description` are extracted. The `prompt` field is
//! never transmitted or stored.
//!
//! # Architecture
//!
//! This module provides parsing functions that can be called from the existing
//! parser when a Task tool_use is detected. It does NOT create its own file
//! watcher (that's handled by the existing watcher.rs).
//!
//! # Example
//!
//! ```
//! use chrono::Utc;
//! use vibetea_monitor::trackers::agent_tracker::{parse_task_tool_use, create_agent_spawn_event};
//!
//! let input = serde_json::json!({
//!     "description": "Run unit tests",
//!     "prompt": "...",
//!     "subagent_type": "devs:rust-dev"
//! });
//!
//! if let Some(task_input) = parse_task_tool_use("Task", &input) {
//!     let event = create_agent_spawn_event(
//!         "session-123".to_string(),
//!         Utc::now(),
//!         &task_input,
//!     );
//!     assert_eq!(event.agent_type, "devs:rust-dev");
//!     assert_eq!(event.description, "Run unit tests");
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::types::AgentSpawnEvent;

/// Task tool input parameters.
///
/// Represents the `input` field of a Task tool_use content block.
/// Only the metadata fields needed for event creation are extracted;
/// the `prompt` field is intentionally omitted for privacy.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TaskToolInput {
    /// The type of subagent being spawned (e.g., "devs:rust-dev", "task").
    ///
    /// If not present in the input, defaults to "task".
    #[serde(default = "default_subagent_type")]
    pub subagent_type: String,

    /// Description of the task being delegated to the subagent.
    ///
    /// If not present in the input, defaults to an empty string.
    #[serde(default)]
    pub description: String,
}

/// Provides the default value for `subagent_type` when not present in input.
fn default_subagent_type() -> String {
    "task".to_string()
}

/// Parses a Task tool_use input, extracting the relevant metadata.
///
/// This function takes the tool name and input from a `ContentBlock::ToolUse`
/// and returns `Some(TaskToolInput)` if the tool is the Task tool.
///
/// # Arguments
///
/// * `tool_name` - The name of the tool (must be "Task" for a match)
/// * `input` - The tool input as a JSON value
///
/// # Returns
///
/// * `Some(TaskToolInput)` if the tool is "Task" and the input can be parsed
/// * `None` if the tool is not "Task" or parsing fails
///
/// # Example
///
/// ```
/// use vibetea_monitor::trackers::agent_tracker::parse_task_tool_use;
///
/// // Valid Task tool use
/// let input = serde_json::json!({
///     "description": "Create error types",
///     "subagent_type": "devs:rust-dev"
/// });
/// let result = parse_task_tool_use("Task", &input);
/// assert!(result.is_some());
/// assert_eq!(result.unwrap().subagent_type, "devs:rust-dev");
///
/// // Non-Task tool use returns None
/// let result = parse_task_tool_use("Read", &input);
/// assert!(result.is_none());
/// ```
#[must_use]
pub fn parse_task_tool_use(tool_name: &str, input: &serde_json::Value) -> Option<TaskToolInput> {
    // Only process Task tool invocations
    if tool_name != "Task" {
        return None;
    }

    // Attempt to deserialize the input; return None on parse failure
    // This is lenient: missing fields get defaults
    serde_json::from_value(input.clone()).ok()
}

/// Creates an [`AgentSpawnEvent`] from parsed Task tool input.
///
/// This function constructs a complete event structure from the parsed
/// task input combined with session context (session ID and timestamp).
///
/// # Arguments
///
/// * `session_id` - The session in which the agent was spawned
/// * `timestamp` - When the agent spawn occurred
/// * `task_input` - The parsed Task tool input containing agent metadata
///
/// # Returns
///
/// A fully populated [`AgentSpawnEvent`] ready for transmission.
///
/// # Example
///
/// ```
/// use chrono::Utc;
/// use vibetea_monitor::trackers::agent_tracker::{TaskToolInput, create_agent_spawn_event};
///
/// let task_input = TaskToolInput {
///     subagent_type: "devs:rust-dev".to_string(),
///     description: "Implement error handling".to_string(),
/// };
///
/// let event = create_agent_spawn_event(
///     "sess_abc123".to_string(),
///     Utc::now(),
///     &task_input,
/// );
///
/// assert_eq!(event.session_id, "sess_abc123");
/// assert_eq!(event.agent_type, "devs:rust-dev");
/// assert_eq!(event.description, "Implement error handling");
/// ```
#[must_use]
pub fn create_agent_spawn_event(
    session_id: String,
    timestamp: DateTime<Utc>,
    task_input: &TaskToolInput,
) -> AgentSpawnEvent {
    AgentSpawnEvent {
        session_id,
        agent_type: task_input.subagent_type.clone(),
        description: task_input.description.clone(),
        timestamp,
    }
}

/// Attempts to extract an [`AgentSpawnEvent`] from a tool_use content block.
///
/// This is a convenience function that combines [`parse_task_tool_use`] and
/// [`create_agent_spawn_event`] for use in the main parser.
///
/// # Arguments
///
/// * `tool_name` - The name of the tool from the content block
/// * `input` - The tool input as a JSON value
/// * `session_id` - The current session ID
/// * `timestamp` - When the tool was invoked
///
/// # Returns
///
/// * `Some(AgentSpawnEvent)` if this is a valid Task tool invocation
/// * `None` if the tool is not Task or parsing fails
///
/// # Example
///
/// ```
/// use chrono::Utc;
/// use vibetea_monitor::trackers::agent_tracker::try_extract_agent_spawn;
///
/// let input = serde_json::json!({
///     "description": "Run tests",
///     "subagent_type": "task"
/// });
///
/// let event = try_extract_agent_spawn(
///     "Task",
///     &input,
///     "sess_123".to_string(),
///     Utc::now(),
/// );
///
/// assert!(event.is_some());
/// ```
#[must_use]
pub fn try_extract_agent_spawn(
    tool_name: &str,
    input: &serde_json::Value,
    session_id: String,
    timestamp: DateTime<Utc>,
) -> Option<AgentSpawnEvent> {
    let task_input = parse_task_tool_use(tool_name, input)?;
    Some(create_agent_spawn_event(session_id, timestamp, &task_input))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // =========================================================================
    // T070: Task tool_use Parsing Tests
    // =========================================================================

    #[test]
    fn parse_task_tool_use_with_all_fields() {
        let input = serde_json::json!({
            "description": "Create SmileError enum",
            "prompt": "Create the SmileError enum with variants for...",
            "subagent_type": "devs:rust-dev"
        });

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        let task = result.unwrap();
        assert_eq!(task.subagent_type, "devs:rust-dev");
        assert_eq!(task.description, "Create SmileError enum");
    }

    #[test]
    fn parse_task_tool_use_extracts_subagent_type() {
        let input = serde_json::json!({
            "subagent_type": "background:file-watcher",
            "description": "Watch for file changes"
        });

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        assert_eq!(result.unwrap().subagent_type, "background:file-watcher");
    }

    #[test]
    fn parse_task_tool_use_extracts_description() {
        let input = serde_json::json!({
            "description": "Implement unit tests for parser module",
            "subagent_type": "task"
        });

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        assert_eq!(
            result.unwrap().description,
            "Implement unit tests for parser module"
        );
    }

    #[test]
    fn parse_task_tool_use_returns_none_for_non_task_tool() {
        let input = serde_json::json!({
            "description": "Some description",
            "subagent_type": "task"
        });

        // Test various non-Task tool names
        assert!(parse_task_tool_use("Read", &input).is_none());
        assert!(parse_task_tool_use("Bash", &input).is_none());
        assert!(parse_task_tool_use("Edit", &input).is_none());
        assert!(parse_task_tool_use("Write", &input).is_none());
        assert!(parse_task_tool_use("Glob", &input).is_none());
        assert!(parse_task_tool_use("Grep", &input).is_none());
        assert!(parse_task_tool_use("WebFetch", &input).is_none());
        assert!(parse_task_tool_use("task", &input).is_none()); // case sensitive
        assert!(parse_task_tool_use("TASK", &input).is_none()); // case sensitive
    }

    #[test]
    fn parse_task_tool_use_handles_missing_subagent_type() {
        let input = serde_json::json!({
            "description": "A task without explicit subagent_type"
        });

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        let task = result.unwrap();
        // Should default to "task" when missing
        assert_eq!(task.subagent_type, "task");
        assert_eq!(task.description, "A task without explicit subagent_type");
    }

    #[test]
    fn parse_task_tool_use_handles_missing_description() {
        let input = serde_json::json!({
            "subagent_type": "devs:rust-dev"
        });

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        let task = result.unwrap();
        assert_eq!(task.subagent_type, "devs:rust-dev");
        // Should default to empty string when missing
        assert_eq!(task.description, "");
    }

    #[test]
    fn parse_task_tool_use_handles_both_fields_missing() {
        let input = serde_json::json!({
            "prompt": "Some prompt that we ignore for privacy"
        });

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        let task = result.unwrap();
        assert_eq!(task.subagent_type, "task"); // default
        assert_eq!(task.description, ""); // default
    }

    #[test]
    fn parse_task_tool_use_handles_empty_input() {
        let input = serde_json::json!({});

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        let task = result.unwrap();
        assert_eq!(task.subagent_type, "task");
        assert_eq!(task.description, "");
    }

    #[test]
    fn parse_task_tool_use_handles_null_input() {
        let input = serde_json::Value::Null;

        // Parsing null should fail since it's not an object
        let result = parse_task_tool_use("Task", &input);
        assert!(result.is_none());
    }

    #[test]
    fn parse_task_tool_use_ignores_prompt_field() {
        // The prompt field should be ignored for privacy
        let input = serde_json::json!({
            "description": "Test task",
            "prompt": "This is a very long prompt that contains sensitive information...",
            "subagent_type": "task"
        });

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        // The TaskToolInput struct doesn't have a prompt field
        let task = result.unwrap();
        assert_eq!(task.description, "Test task");
        assert_eq!(task.subagent_type, "task");
    }

    #[test]
    fn parse_task_tool_use_handles_extra_fields() {
        let input = serde_json::json!({
            "description": "Task with extras",
            "subagent_type": "devs:python-dev",
            "unknown_field": "should be ignored",
            "another_extra": 42
        });

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        let task = result.unwrap();
        assert_eq!(task.description, "Task with extras");
        assert_eq!(task.subagent_type, "devs:python-dev");
    }

    #[test]
    fn parse_task_tool_use_handles_empty_string_values() {
        let input = serde_json::json!({
            "description": "",
            "subagent_type": ""
        });

        let result = parse_task_tool_use("Task", &input);

        assert!(result.is_some());
        let task = result.unwrap();
        assert_eq!(task.subagent_type, "");
        assert_eq!(task.description, "");
    }

    // =========================================================================
    // T071: AgentSpawnEvent Emission Tests
    // =========================================================================

    #[test]
    fn create_agent_spawn_event_maps_fields_correctly() {
        let task_input = TaskToolInput {
            subagent_type: "devs:rust-dev".to_string(),
            description: "Implement error handling".to_string(),
        };
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 5, 1, 57).unwrap();

        let event = create_agent_spawn_event("sess_abc123".to_string(), timestamp, &task_input);

        assert_eq!(event.session_id, "sess_abc123");
        assert_eq!(event.agent_type, "devs:rust-dev");
        assert_eq!(event.description, "Implement error handling");
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn create_agent_spawn_event_uses_subagent_type_as_agent_type() {
        let task_input = TaskToolInput {
            subagent_type: "background:watcher".to_string(),
            description: "".to_string(),
        };
        let timestamp = Utc::now();

        let event = create_agent_spawn_event("session-1".to_string(), timestamp, &task_input);

        // agent_type should be the subagent_type from input
        assert_eq!(event.agent_type, "background:watcher");
    }

    #[test]
    fn create_agent_spawn_event_preserves_empty_description() {
        let task_input = TaskToolInput {
            subagent_type: "task".to_string(),
            description: "".to_string(),
        };
        let timestamp = Utc::now();

        let event = create_agent_spawn_event("session-2".to_string(), timestamp, &task_input);

        assert_eq!(event.description, "");
    }

    #[test]
    fn create_agent_spawn_event_preserves_timestamp() {
        let task_input = TaskToolInput {
            subagent_type: "task".to_string(),
            description: "Test".to_string(),
        };
        let expected_timestamp = Utc.with_ymd_and_hms(2025, 6, 15, 14, 30, 0).unwrap();

        let event =
            create_agent_spawn_event("session-3".to_string(), expected_timestamp, &task_input);

        assert_eq!(event.timestamp, expected_timestamp);
    }

    #[test]
    fn create_agent_spawn_event_handles_unicode_description() {
        let task_input = TaskToolInput {
            subagent_type: "devs:rust-dev".to_string(),
            description: "Handle UTF-8: emoji test".to_string(),
        };
        let timestamp = Utc::now();

        let event = create_agent_spawn_event("session-4".to_string(), timestamp, &task_input);

        assert_eq!(event.description, "Handle UTF-8: emoji test");
    }

    // =========================================================================
    // Integration Tests: Full Parse to Event Flow
    // =========================================================================

    #[test]
    fn try_extract_agent_spawn_full_flow() {
        let input = serde_json::json!({
            "description": "Create unit tests for parser",
            "prompt": "Write comprehensive unit tests...",
            "subagent_type": "devs:rust-dev"
        });
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();

        let event =
            try_extract_agent_spawn("Task", &input, "session-full-flow".to_string(), timestamp);

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.session_id, "session-full-flow");
        assert_eq!(event.agent_type, "devs:rust-dev");
        assert_eq!(event.description, "Create unit tests for parser");
        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn try_extract_agent_spawn_returns_none_for_non_task() {
        let input = serde_json::json!({
            "file_path": "/some/path/to/file.rs"
        });
        let timestamp = Utc::now();

        let event = try_extract_agent_spawn("Read", &input, "session-read".to_string(), timestamp);

        assert!(event.is_none());
    }

    #[test]
    fn try_extract_agent_spawn_with_defaults() {
        let input = serde_json::json!({});
        let timestamp = Utc::now();

        let event =
            try_extract_agent_spawn("Task", &input, "session-defaults".to_string(), timestamp);

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.agent_type, "task"); // default
        assert_eq!(event.description, ""); // default
    }

    // =========================================================================
    // TaskToolInput Struct Tests
    // =========================================================================

    #[test]
    fn task_tool_input_debug() {
        let input = TaskToolInput {
            subagent_type: "test".to_string(),
            description: "test description".to_string(),
        };

        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("TaskToolInput"));
        assert!(debug_str.contains("subagent_type"));
        assert!(debug_str.contains("description"));
    }

    #[test]
    fn task_tool_input_clone() {
        let original = TaskToolInput {
            subagent_type: "devs:rust-dev".to_string(),
            description: "Original description".to_string(),
        };

        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(cloned.subagent_type, "devs:rust-dev");
        assert_eq!(cloned.description, "Original description");
    }

    #[test]
    fn task_tool_input_equality() {
        let a = TaskToolInput {
            subagent_type: "task".to_string(),
            description: "Same".to_string(),
        };

        let b = TaskToolInput {
            subagent_type: "task".to_string(),
            description: "Same".to_string(),
        };

        let c = TaskToolInput {
            subagent_type: "task".to_string(),
            description: "Different".to_string(),
        };

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn parse_handles_numeric_type_values() {
        // What if someone passes a number instead of a string?
        let input = serde_json::json!({
            "description": 42,  // wrong type
            "subagent_type": "task"
        });

        // Should fail to parse because description is not a string
        let result = parse_task_tool_use("Task", &input);
        assert!(result.is_none());
    }

    #[test]
    fn parse_handles_array_input() {
        let input = serde_json::json!(["not", "an", "object"]);

        let result = parse_task_tool_use("Task", &input);
        assert!(result.is_none());
    }

    #[test]
    fn parse_handles_nested_objects() {
        let input = serde_json::json!({
            "description": "Nested test",
            "subagent_type": "task",
            "nested": {
                "should": "be ignored"
            }
        });

        let result = parse_task_tool_use("Task", &input);
        assert!(result.is_some());
        assert_eq!(result.unwrap().description, "Nested test");
    }

    // =========================================================================
    // Realistic JSONL Line Parsing Tests
    // =========================================================================

    /// Simulates parsing a realistic JSONL line from Claude Code
    #[test]
    fn parse_realistic_jsonl_task_tool_use() {
        // This simulates what you'd see in a real JSONL file
        let jsonl_line = r#"{
            "type": "assistant",
            "message": {
                "content": [
                    {
                        "type": "tool_use",
                        "id": "toolu_01Fw1HCjXzYHNtWX7jXWzgBj",
                        "name": "Task",
                        "input": {
                            "description": "Create SmileError enum",
                            "prompt": "Create the SmileError enum with the following variants...",
                            "subagent_type": "devs:rust-dev"
                        }
                    }
                ]
            },
            "timestamp": "2026-02-03T05:01:57.678Z"
        }"#;

        // Parse the full JSON structure
        let parsed: serde_json::Value = serde_json::from_str(jsonl_line).unwrap();

        // Extract the tool_use content block (simulating what the parser does)
        let content = &parsed["message"]["content"][0];
        let tool_name = content["name"].as_str().unwrap();
        let input = &content["input"];

        // Now use our module to parse
        let result = parse_task_tool_use(tool_name, input);

        assert!(result.is_some());
        let task = result.unwrap();
        assert_eq!(task.subagent_type, "devs:rust-dev");
        assert_eq!(task.description, "Create SmileError enum");
    }

    /// Tests that non-Task tools in a realistic JSONL context are rejected
    #[test]
    fn parse_realistic_jsonl_non_task_tool() {
        let jsonl_line = r#"{
            "type": "assistant",
            "message": {
                "content": [
                    {
                        "type": "tool_use",
                        "id": "toolu_01XYZ",
                        "name": "Read",
                        "input": {
                            "file_path": "/home/user/project/src/main.rs"
                        }
                    }
                ]
            },
            "timestamp": "2026-02-03T05:02:00.000Z"
        }"#;

        let parsed: serde_json::Value = serde_json::from_str(jsonl_line).unwrap();
        let content = &parsed["message"]["content"][0];
        let tool_name = content["name"].as_str().unwrap();
        let input = &content["input"];

        let result = parse_task_tool_use(tool_name, input);

        assert!(result.is_none());
    }
}
