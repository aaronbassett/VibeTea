//! Shared event types for the VibeTea server.
//!
//! This module defines the core data structures for events flowing through the system.
//! Events are immutable once created and follow a strict schema defined in the data model.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The type of event being transmitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Session,
    Activity,
    Tool,
    Agent,
    Summary,
    Error,
    AgentSpawn,
    SkillInvocation,
    TokenUsage,
    SessionMetrics,
    ActivityPattern,
    ModelDistribution,
    TodoProgress,
    FileChange,
    ProjectActivity,
}

/// Action performed on a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAction {
    Started,
    Ended,
}

/// Status of a tool invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Started,
    Completed,
}

/// Event tracking Task tool agent spawns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSpawnEvent {
    pub session_id: String,
    pub agent_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

/// Event tracking skill/slash command invocations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInvocationEvent {
    pub session_id: String,
    pub skill_name: String,
    pub project: String,
    pub timestamp: DateTime<Utc>,
}

/// Event tracking per-model token consumption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageEvent {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
}

/// Event tracking global session metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMetricsEvent {
    pub total_sessions: u64,
    pub total_messages: u64,
    pub total_tool_usage: u64,
    pub longest_session: String,
}

/// Event tracking hourly activity distribution.
///
/// Note: `hour_counts` uses `String` keys (e.g., "0" through "23") rather than `u8`
/// to ensure reliable JSON deserialization with serde's untagged enum support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPatternEvent {
    pub hour_counts: HashMap<String, u64>,
}

/// Summary of token usage for a model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageSummary {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
}

/// Event tracking usage distribution across models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDistributionEvent {
    pub model_usage: HashMap<String, TokenUsageSummary>,
}

/// Event tracking todo list progress per session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoProgressEvent {
    pub session_id: String,
    pub completed: u32,
    pub in_progress: u32,
    pub pending: u32,
    pub abandoned: bool,
}

/// Event tracking file edit history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeEvent {
    pub session_id: String,
    pub file_hash: String,
    pub version: u32,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub lines_modified: u32,
    pub timestamp: DateTime<Utc>,
}

/// Event tracking project activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectActivityEvent {
    pub project_path: String,
    pub session_id: String,
    pub is_active: bool,
}

/// Type-specific payload for events.
///
/// Each variant corresponds to an [`EventType`] and contains the relevant data
/// for that event type. The payload is serialized as an untagged union - the
/// correct variant is determined by the `type` field on the parent [`Event`].
/// Field names use `camelCase` to match the JSON API contract.
///
/// **Important**: Variants are ordered from most specific (most required fields)
/// to least specific for correct untagged deserialization. Do not reorder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    /// Tool invocation events.
    ///
    /// Most specific: requires `session_id`, `tool`, and `status`.
    #[serde(rename_all = "camelCase")]
    Tool {
        session_id: Uuid,
        tool: String,
        status: ToolStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        project: Option<String>,
    },

    /// Session lifecycle events (start/end).
    ///
    /// Requires `session_id`, `action`, and `project`.
    #[serde(rename_all = "camelCase")]
    Session {
        session_id: Uuid,
        action: SessionAction,
        project: String,
    },

    /// Session summary events (marks session end).
    ///
    /// Requires `session_id` and `summary`.
    #[serde(rename_all = "camelCase")]
    Summary { session_id: Uuid, summary: String },

    /// Agent state change events.
    ///
    /// Requires `session_id` and `state`.
    #[serde(rename_all = "camelCase")]
    Agent { session_id: Uuid, state: String },

    /// Error events for monitoring purposes.
    ///
    /// Requires `session_id` and `category`.
    #[serde(rename_all = "camelCase")]
    Error { session_id: Uuid, category: String },

    // New event variants for enhanced tracking - ordered from most specific to least specific
    // These must come BEFORE Activity since Activity only requires session_id

    /// File change tracking events.
    ///
    /// Very specific: has `version`, `lines_added`, `lines_removed`, `lines_modified`, `file_hash`.
    FileChange(FileChangeEvent),

    /// Agent spawn tracking events.
    ///
    /// Has `agent_type` and `description` fields.
    AgentSpawn(AgentSpawnEvent),

    /// Skill invocation tracking events.
    ///
    /// Has `skill_name` field.
    SkillInvocation(SkillInvocationEvent),

    /// Token usage tracking events.
    ///
    /// Has `model` and various token count fields.
    TokenUsage(TokenUsageEvent),

    /// Session metrics tracking events.
    ///
    /// Has `total_*` fields and `longest_session`.
    SessionMetrics(SessionMetricsEvent),

    /// Model distribution tracking events.
    ///
    /// Has `model_usage` HashMap.
    ModelDistribution(ModelDistributionEvent),

    /// Todo progress tracking events.
    ///
    /// Has `completed`, `pending`, `in_progress`, `abandoned` fields.
    TodoProgress(TodoProgressEvent),

    /// Activity pattern tracking events.
    ///
    /// Has `hour_counts` HashMap.
    ActivityPattern(ActivityPatternEvent),

    /// Project activity tracking events.
    ///
    /// Has `project_path` and `is_active` fields.
    ProjectActivity(ProjectActivityEvent),

    /// Activity heartbeat events.
    ///
    /// Least specific: only requires `session_id`. Must be last for correct
    /// untagged deserialization.
    #[serde(rename_all = "camelCase")]
    Activity {
        session_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        project: Option<String>,
    },
}

/// An event flowing through the VibeTea system.
///
/// Events are the core data unit and are immutable once created. Each event
/// has a unique ID, originates from a specific source (monitor), and contains
/// a type-specific payload.
///
/// # Event ID Format
///
/// Event IDs follow the format: `evt_` + 20 alphanumeric characters.
/// Example: `evt_a1b2c3d4e5f6g7h8i9j0`
///
/// # Example
///
/// ```
/// use vibetea_server::types::{Event, EventType, EventPayload, ToolStatus};
/// use chrono::Utc;
/// use uuid::Uuid;
///
/// let event = Event {
///     id: "evt_k7m2n9p4q1r6s3t8u5v0".to_string(),
///     source: "macbook-pro".to_string(),
///     timestamp: Utc::now(),
///     event_type: EventType::Tool,
///     payload: EventPayload::Tool {
///         session_id: Uuid::new_v4(),
///         tool: "Read".to_string(),
///         status: ToolStatus::Completed,
///         context: Some("main.rs".to_string()),
///         project: Some("vibetea".to_string()),
///     },
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// Unique event identifier (evt_ + 20 alphanumeric chars).
    pub id: String,

    /// Monitor identifier (hostname or custom ID).
    pub source: String,

    /// RFC 3339 UTC timestamp.
    pub timestamp: DateTime<Utc>,

    /// The type of event.
    #[serde(rename = "type")]
    pub event_type: EventType,

    /// Type-specific event payload.
    pub payload: EventPayload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_serialization() {
        assert_eq!(
            serde_json::to_string(&EventType::Session).unwrap(),
            r#""session""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::Activity).unwrap(),
            r#""activity""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::Tool).unwrap(),
            r#""tool""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::Agent).unwrap(),
            r#""agent""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::Summary).unwrap(),
            r#""summary""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::Error).unwrap(),
            r#""error""#
        );
        // New event types
        assert_eq!(
            serde_json::to_string(&EventType::AgentSpawn).unwrap(),
            r#""agent_spawn""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::SkillInvocation).unwrap(),
            r#""skill_invocation""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::TokenUsage).unwrap(),
            r#""token_usage""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::SessionMetrics).unwrap(),
            r#""session_metrics""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::ActivityPattern).unwrap(),
            r#""activity_pattern""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::ModelDistribution).unwrap(),
            r#""model_distribution""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::TodoProgress).unwrap(),
            r#""todo_progress""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::FileChange).unwrap(),
            r#""file_change""#
        );
        assert_eq!(
            serde_json::to_string(&EventType::ProjectActivity).unwrap(),
            r#""project_activity""#
        );
    }

    #[test]
    fn test_event_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<EventType>(r#""session""#).unwrap(),
            EventType::Session
        );
        assert_eq!(
            serde_json::from_str::<EventType>(r#""tool""#).unwrap(),
            EventType::Tool
        );
    }

    #[test]
    fn test_session_action_serialization() {
        assert_eq!(
            serde_json::to_string(&SessionAction::Started).unwrap(),
            r#""started""#
        );
        assert_eq!(
            serde_json::to_string(&SessionAction::Ended).unwrap(),
            r#""ended""#
        );
    }

    #[test]
    fn test_tool_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ToolStatus::Started).unwrap(),
            r#""started""#
        );
        assert_eq!(
            serde_json::to_string(&ToolStatus::Completed).unwrap(),
            r#""completed""#
        );
    }

    #[test]
    fn test_event_serialization_tool() {
        let session_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let timestamp = DateTime::parse_from_rfc3339("2026-02-02T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let event = Event {
            id: "evt_k7m2n9p4q1r6s3t8u5v0".to_string(),
            source: "macbook-pro".to_string(),
            timestamp,
            event_type: EventType::Tool,
            payload: EventPayload::Tool {
                session_id,
                tool: "Read".to_string(),
                status: ToolStatus::Completed,
                context: Some("main.rs".to_string()),
                project: Some("vibetea".to_string()),
            },
        };

        let json = serde_json::to_string_pretty(&event).unwrap();

        // Verify the JSON structure matches the expected format
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["id"], "evt_k7m2n9p4q1r6s3t8u5v0");
        assert_eq!(parsed["source"], "macbook-pro");
        assert_eq!(parsed["type"], "tool");
        assert_eq!(
            parsed["payload"]["sessionId"],
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(parsed["payload"]["tool"], "Read");
        assert_eq!(parsed["payload"]["status"], "completed");
        assert_eq!(parsed["payload"]["context"], "main.rs");
        assert_eq!(parsed["payload"]["project"], "vibetea");
    }

    #[test]
    fn test_event_serialization_session() {
        let session_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let timestamp = DateTime::parse_from_rfc3339("2026-02-02T14:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let event = Event {
            id: "evt_a1b2c3d4e5f6g7h8i9j0".to_string(),
            source: "macbook-pro".to_string(),
            timestamp,
            event_type: EventType::Session,
            payload: EventPayload::Session {
                session_id,
                action: SessionAction::Started,
                project: "vibetea".to_string(),
            },
        };

        let json = serde_json::to_string_pretty(&event).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "session");
        assert_eq!(parsed["payload"]["action"], "started");
        assert_eq!(parsed["payload"]["project"], "vibetea");
    }

    #[test]
    fn test_event_deserialization_from_json() {
        let json = r#"{
            "id": "evt_k7m2n9p4q1r6s3t8u5v0",
            "source": "macbook-pro",
            "timestamp": "2026-02-02T14:30:00Z",
            "type": "tool",
            "payload": {
                "sessionId": "550e8400-e29b-41d4-a716-446655440000",
                "tool": "Read",
                "status": "completed",
                "context": "main.rs",
                "project": "vibetea"
            }
        }"#;

        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, "evt_k7m2n9p4q1r6s3t8u5v0");
        assert_eq!(event.source, "macbook-pro");
        assert_eq!(event.event_type, EventType::Tool);

        if let EventPayload::Tool {
            session_id,
            tool,
            status,
            context,
            project,
        } = event.payload
        {
            assert_eq!(
                session_id,
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
            );
            assert_eq!(tool, "Read");
            assert_eq!(status, ToolStatus::Completed);
            assert_eq!(context, Some("main.rs".to_string()));
            assert_eq!(project, Some("vibetea".to_string()));
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn test_optional_fields_omitted_when_none() {
        let session_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let payload = EventPayload::Activity {
            session_id,
            project: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("project"));
    }

    #[test]
    fn test_roundtrip_all_event_types() {
        let session_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let payloads = vec![
            EventPayload::Session {
                session_id,
                action: SessionAction::Started,
                project: "test".to_string(),
            },
            EventPayload::Activity {
                session_id,
                project: Some("test".to_string()),
            },
            EventPayload::Tool {
                session_id,
                tool: "Write".to_string(),
                status: ToolStatus::Started,
                context: None,
                project: None,
            },
            EventPayload::Agent {
                session_id,
                state: "thinking".to_string(),
            },
            EventPayload::Summary {
                session_id,
                summary: "Completed refactoring".to_string(),
            },
            EventPayload::Error {
                session_id,
                category: "network".to_string(),
            },
            // New event types
            EventPayload::FileChange(FileChangeEvent {
                session_id: session_id.to_string(),
                file_hash: "abc123".to_string(),
                version: 1,
                lines_added: 10,
                lines_removed: 5,
                lines_modified: 3,
                timestamp,
            }),
            EventPayload::AgentSpawn(AgentSpawnEvent {
                session_id: session_id.to_string(),
                agent_type: "task".to_string(),
                description: "Running tests".to_string(),
                timestamp,
            }),
            EventPayload::SkillInvocation(SkillInvocationEvent {
                session_id: session_id.to_string(),
                skill_name: "commit".to_string(),
                project: "vibetea".to_string(),
                timestamp,
            }),
            EventPayload::TokenUsage(TokenUsageEvent {
                model: "claude-3".to_string(),
                input_tokens: 1000,
                output_tokens: 500,
                cache_read_tokens: 200,
                cache_creation_tokens: 100,
            }),
            EventPayload::SessionMetrics(SessionMetricsEvent {
                total_sessions: 10,
                total_messages: 100,
                total_tool_usage: 50,
                longest_session: "2h 30m".to_string(),
            }),
            EventPayload::ModelDistribution(ModelDistributionEvent {
                model_usage: {
                    let mut map = HashMap::new();
                    map.insert(
                        "claude-3".to_string(),
                        TokenUsageSummary {
                            input_tokens: 5000,
                            output_tokens: 2500,
                            cache_read_tokens: 1000,
                            cache_creation_tokens: 500,
                        },
                    );
                    map
                },
            }),
            EventPayload::TodoProgress(TodoProgressEvent {
                session_id: session_id.to_string(),
                completed: 5,
                in_progress: 2,
                pending: 3,
                abandoned: false,
            }),
            EventPayload::ActivityPattern(ActivityPatternEvent {
                hour_counts: {
                    let mut map = HashMap::new();
                    map.insert("9".to_string(), 10);
                    map.insert("14".to_string(), 25);
                    map
                },
            }),
            EventPayload::ProjectActivity(ProjectActivityEvent {
                project_path: "/home/user/project".to_string(),
                session_id: session_id.to_string(),
                is_active: true,
            }),
        ];

        for (i, payload) in payloads.into_iter().enumerate() {
            let event = Event {
                id: format!("evt_test{:0>19}", i),
                source: "test".to_string(),
                timestamp,
                event_type: match &payload {
                    EventPayload::Session { .. } => EventType::Session,
                    EventPayload::Activity { .. } => EventType::Activity,
                    EventPayload::Tool { .. } => EventType::Tool,
                    EventPayload::Agent { .. } => EventType::Agent,
                    EventPayload::Summary { .. } => EventType::Summary,
                    EventPayload::Error { .. } => EventType::Error,
                    EventPayload::FileChange(_) => EventType::FileChange,
                    EventPayload::AgentSpawn(_) => EventType::AgentSpawn,
                    EventPayload::SkillInvocation(_) => EventType::SkillInvocation,
                    EventPayload::TokenUsage(_) => EventType::TokenUsage,
                    EventPayload::SessionMetrics(_) => EventType::SessionMetrics,
                    EventPayload::ModelDistribution(_) => EventType::ModelDistribution,
                    EventPayload::TodoProgress(_) => EventType::TodoProgress,
                    EventPayload::ActivityPattern(_) => EventType::ActivityPattern,
                    EventPayload::ProjectActivity(_) => EventType::ProjectActivity,
                },
                payload,
            };

            let json = serde_json::to_string(&event).unwrap();
            let roundtrip: Event = serde_json::from_str(&json).unwrap();
            assert_eq!(event, roundtrip);
        }
    }

}
