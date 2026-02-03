//! Event types for VibeTea session monitoring.
//!
//! This module defines the shared event schema used for communication between
//! the monitor and server components. All types serialize to camelCase JSON.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Length of the random alphanumeric suffix in event IDs.
const EVENT_ID_SUFFIX_LEN: usize = 20;

/// Prefix for all event IDs.
const EVENT_ID_PREFIX: &str = "evt_";

/// Type classification for events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Session,
    Activity,
    Tool,
    Agent,
    Summary,
    Error,
    // Enhanced tracking event types
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

/// Actions that can occur during a session lifecycle.
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

// ============================================================================
// Enhanced Tracking Event Structs
// ============================================================================

/// Event tracking Task tool agent spawns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSpawnEvent {
    /// The session in which the agent was spawned.
    pub session_id: String,
    /// Type of agent (e.g., "task", "background").
    pub agent_type: String,
    /// Description of the agent's task.
    pub description: String,
    /// When the agent was spawned.
    pub timestamp: DateTime<Utc>,
}

/// Event tracking skill/slash command invocations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInvocationEvent {
    /// The session in which the skill was invoked.
    pub session_id: String,
    /// Name of the skill (e.g., "commit", "review-pr").
    pub skill_name: String,
    /// Project context for the skill invocation.
    pub project: String,
    /// When the skill was invoked.
    pub timestamp: DateTime<Utc>,
}

/// Event tracking per-model token consumption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageEvent {
    /// The model that consumed the tokens.
    pub model: String,
    /// Number of input tokens consumed.
    pub input_tokens: u64,
    /// Number of output tokens generated.
    pub output_tokens: u64,
    /// Number of tokens read from cache.
    pub cache_read_tokens: u64,
    /// Number of tokens written to cache.
    pub cache_creation_tokens: u64,
}

/// Event tracking global session metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMetricsEvent {
    /// Total number of sessions tracked.
    pub total_sessions: u64,
    /// Total number of messages across all sessions.
    pub total_messages: u64,
    /// Total number of tool invocations.
    pub total_tool_usage: u64,
    /// Identifier of the longest session.
    pub longest_session: String,
}

/// Event tracking hourly activity distribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPatternEvent {
    /// Map of hour (0-23 as string) to activity count.
    /// Hours are stored as strings for JSON compatibility.
    pub hour_counts: HashMap<String, u64>,
}

/// Summary of token usage for a specific model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageSummary {
    /// Number of input tokens consumed.
    pub input_tokens: u64,
    /// Number of output tokens generated.
    pub output_tokens: u64,
    /// Number of tokens read from cache.
    pub cache_read_tokens: u64,
    /// Number of tokens written to cache.
    pub cache_creation_tokens: u64,
}

/// Event tracking usage distribution across models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDistributionEvent {
    /// Map of model name to token usage summary.
    pub model_usage: HashMap<String, TokenUsageSummary>,
}

/// Event tracking todo list progress per session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoProgressEvent {
    /// The session being tracked.
    pub session_id: String,
    /// Number of completed todo items.
    pub completed: u32,
    /// Number of in-progress todo items.
    pub in_progress: u32,
    /// Number of pending todo items.
    pub pending: u32,
    /// Whether the todo list was abandoned.
    pub abandoned: bool,
}

/// Event tracking file edit history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeEvent {
    /// The session in which the file was changed.
    pub session_id: String,
    /// Hash identifying the file (for privacy).
    pub file_hash: String,
    /// Version number of this change.
    pub version: u32,
    /// Number of lines added.
    pub lines_added: u32,
    /// Number of lines removed.
    pub lines_removed: u32,
    /// Number of lines modified.
    pub lines_modified: u32,
    /// When the file was changed.
    pub timestamp: DateTime<Utc>,
}

/// Event tracking project activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectActivityEvent {
    /// Path to the project.
    pub project_path: String,
    /// The active session in this project.
    pub session_id: String,
    /// Whether the project is currently active.
    pub is_active: bool,
}

/// Payload variants for different event types.
///
/// Uses serde's internally tagged representation for clean JSON output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    /// Session lifecycle event.
    Session {
        #[serde(rename = "sessionId")]
        session_id: Uuid,
        action: SessionAction,
        project: String,
    },
    /// Activity heartbeat event.
    Activity {
        #[serde(rename = "sessionId")]
        session_id: Uuid,
        project: Option<String>,
    },
    /// Tool usage event.
    Tool {
        #[serde(rename = "sessionId")]
        session_id: Uuid,
        tool: String,
        status: ToolStatus,
        context: Option<String>,
        project: Option<String>,
    },
    /// Agent state change event.
    Agent {
        #[serde(rename = "sessionId")]
        session_id: Uuid,
        state: String,
    },
    /// Session summary event.
    Summary {
        #[serde(rename = "sessionId")]
        session_id: Uuid,
        summary: String,
    },
    /// Error event.
    Error {
        #[serde(rename = "sessionId")]
        session_id: Uuid,
        category: String,
    },

    // ========================================================================
    // Enhanced Tracking Event Variants
    // ========================================================================

    /// Agent spawn event for Task tool tracking.
    AgentSpawn(AgentSpawnEvent),

    /// Skill/slash command invocation event.
    SkillInvocation(SkillInvocationEvent),

    /// Token usage event for per-model consumption tracking.
    TokenUsage(TokenUsageEvent),

    /// Session metrics event for global statistics.
    SessionMetrics(SessionMetricsEvent),

    /// Activity pattern event for hourly distribution tracking.
    ActivityPattern(ActivityPatternEvent),

    /// Model distribution event for usage across models.
    ModelDistribution(ModelDistributionEvent),

    /// Todo progress event for tracking task completion.
    TodoProgress(TodoProgressEvent),

    /// File change event for edit history tracking.
    FileChange(FileChangeEvent),

    /// Project activity event for tracking active projects.
    ProjectActivity(ProjectActivityEvent),
}

/// A VibeTea monitoring event.
///
/// Events capture session activity and are transmitted to the server for
/// aggregation and analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// Unique event identifier with format `evt_` followed by 20 alphanumeric characters.
    pub id: String,

    /// Source identifier (typically the monitor instance).
    pub source: String,

    /// When the event occurred.
    pub timestamp: DateTime<Utc>,

    /// Classification of the event.
    #[serde(rename = "type")]
    pub event_type: EventType,

    /// Event-specific payload data.
    pub payload: EventPayload,
}

impl Event {
    /// Creates a new event with a randomly generated ID.
    ///
    /// # Arguments
    ///
    /// * `source` - Identifier for the event source
    /// * `event_type` - Classification of this event
    /// * `payload` - Event-specific data
    ///
    /// # Examples
    ///
    /// ```
    /// use uuid::Uuid;
    /// use vibetea_monitor::types::{Event, EventType, EventPayload, SessionAction};
    ///
    /// let event = Event::new(
    ///     "monitor-1".to_string(),
    ///     EventType::Session,
    ///     EventPayload::Session {
    ///         session_id: Uuid::new_v4(),
    ///         action: SessionAction::Started,
    ///         project: "my-project".to_string(),
    ///     },
    /// );
    ///
    /// assert!(event.id.starts_with("evt_"));
    /// assert_eq!(event.id.len(), 24); // "evt_" + 20 chars
    /// ```
    #[must_use]
    pub fn new(source: String, event_type: EventType, payload: EventPayload) -> Self {
        Self {
            id: generate_event_id(),
            source,
            timestamp: Utc::now(),
            event_type,
            payload,
        }
    }
}

/// Generates a unique event ID with the format `evt_` followed by 20 alphanumeric characters.
fn generate_event_id() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

    let mut rng = rand::rng();
    let suffix: String = (0..EVENT_ID_SUFFIX_LEN)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("{EVENT_ID_PREFIX}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_id_has_correct_format() {
        let id = generate_event_id();
        assert!(id.starts_with("evt_"));
        assert_eq!(id.len(), 24); // "evt_" (4) + 20 alphanumeric
    }

    #[test]
    fn event_id_is_alphanumeric_suffix() {
        let id = generate_event_id();
        let suffix = &id[4..];
        assert!(suffix.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn event_type_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&EventType::Session).unwrap(),
            "\"session\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::Activity).unwrap(),
            "\"activity\""
        );
        assert_eq!(serde_json::to_string(&EventType::Tool).unwrap(), "\"tool\"");
        assert_eq!(
            serde_json::to_string(&EventType::Agent).unwrap(),
            "\"agent\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::Summary).unwrap(),
            "\"summary\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::Error).unwrap(),
            "\"error\""
        );
    }

    #[test]
    fn session_action_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&SessionAction::Started).unwrap(),
            "\"started\""
        );
        assert_eq!(
            serde_json::to_string(&SessionAction::Ended).unwrap(),
            "\"ended\""
        );
    }

    #[test]
    fn tool_status_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&ToolStatus::Started).unwrap(),
            "\"started\""
        );
        assert_eq!(
            serde_json::to_string(&ToolStatus::Completed).unwrap(),
            "\"completed\""
        );
    }

    #[test]
    fn event_payload_session_serializes_correctly() {
        let session_id = Uuid::nil();
        let payload = EventPayload::Session {
            session_id,
            action: SessionAction::Started,
            project: "test-project".to_string(),
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "session");
        assert_eq!(json["sessionId"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(json["action"], "started");
        assert_eq!(json["project"], "test-project");
    }

    #[test]
    fn event_payload_tool_serializes_correctly() {
        let session_id = Uuid::nil();
        let payload = EventPayload::Tool {
            session_id,
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("file.rs".to_string()),
            project: None,
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "tool");
        assert_eq!(json["tool"], "Read");
        assert_eq!(json["status"], "completed");
        assert_eq!(json["context"], "file.rs");
        assert!(json["project"].is_null());
    }

    #[test]
    fn event_serializes_with_camel_case_fields() {
        let session_id = Uuid::nil();
        let event = Event {
            id: "evt_12345678901234567890".to_string(),
            source: "test-monitor".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            event_type: EventType::Session,
            payload: EventPayload::Session {
                session_id,
                action: SessionAction::Started,
                project: "test".to_string(),
            },
        };

        let json = serde_json::to_value(&event).unwrap();
        assert!(json.get("id").is_some());
        assert!(json.get("source").is_some());
        assert!(json.get("timestamp").is_some());
        assert!(json.get("type").is_some()); // renamed from event_type
        assert!(json.get("payload").is_some());
        assert!(json.get("eventType").is_none()); // should be renamed to "type"
    }

    #[test]
    fn event_new_generates_valid_id() {
        let event = Event::new(
            "test".to_string(),
            EventType::Activity,
            EventPayload::Activity {
                session_id: Uuid::new_v4(),
                project: None,
            },
        );

        assert!(event.id.starts_with("evt_"));
        assert_eq!(event.id.len(), 24);
    }

    #[test]
    fn event_roundtrip_serialization() {
        let session_id = Uuid::new_v4();
        let original = Event::new(
            "monitor".to_string(),
            EventType::Tool,
            EventPayload::Tool {
                session_id,
                tool: "Bash".to_string(),
                status: ToolStatus::Started,
                context: Some("ls -la".to_string()),
                project: Some("my-project".to_string()),
            },
        );

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.source, deserialized.source);
        assert_eq!(original.event_type, deserialized.event_type);
        assert_eq!(original.payload, deserialized.payload);
    }

    // ========================================================================
    // Enhanced Tracking Event Type Tests
    // ========================================================================

    #[test]
    fn enhanced_event_types_serialize_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&EventType::AgentSpawn).unwrap(),
            "\"agent_spawn\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::SkillInvocation).unwrap(),
            "\"skill_invocation\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::TokenUsage).unwrap(),
            "\"token_usage\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::SessionMetrics).unwrap(),
            "\"session_metrics\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::ActivityPattern).unwrap(),
            "\"activity_pattern\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::ModelDistribution).unwrap(),
            "\"model_distribution\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::TodoProgress).unwrap(),
            "\"todo_progress\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::FileChange).unwrap(),
            "\"file_change\""
        );
        assert_eq!(
            serde_json::to_string(&EventType::ProjectActivity).unwrap(),
            "\"project_activity\""
        );
    }

    #[test]
    fn agent_spawn_event_serializes_with_camel_case() {
        let event = AgentSpawnEvent {
            session_id: "sess_123".to_string(),
            agent_type: "task".to_string(),
            description: "Run unit tests".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["sessionId"], "sess_123");
        assert_eq!(json["agentType"], "task");
        assert_eq!(json["description"], "Run unit tests");
        assert!(json.get("timestamp").is_some());
    }

    #[test]
    fn skill_invocation_event_serializes_with_camel_case() {
        let event = SkillInvocationEvent {
            session_id: "sess_456".to_string(),
            skill_name: "commit".to_string(),
            project: "my-project".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["sessionId"], "sess_456");
        assert_eq!(json["skillName"], "commit");
        assert_eq!(json["project"], "my-project");
    }

    #[test]
    fn token_usage_event_serializes_with_camel_case() {
        let event = TokenUsageEvent {
            model: "claude-3-opus".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 200,
            cache_creation_tokens: 100,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["model"], "claude-3-opus");
        assert_eq!(json["inputTokens"], 1000);
        assert_eq!(json["outputTokens"], 500);
        assert_eq!(json["cacheReadTokens"], 200);
        assert_eq!(json["cacheCreationTokens"], 100);
    }

    #[test]
    fn session_metrics_event_serializes_with_camel_case() {
        let event = SessionMetricsEvent {
            total_sessions: 42,
            total_messages: 1234,
            total_tool_usage: 567,
            longest_session: "sess_longest".to_string(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["totalSessions"], 42);
        assert_eq!(json["totalMessages"], 1234);
        assert_eq!(json["totalToolUsage"], 567);
        assert_eq!(json["longestSession"], "sess_longest");
    }

    #[test]
    fn activity_pattern_event_serializes_with_camel_case() {
        let mut hour_counts = HashMap::new();
        hour_counts.insert("9".to_string(), 15);
        hour_counts.insert("14".to_string(), 25);
        hour_counts.insert("17".to_string(), 10);

        let event = ActivityPatternEvent { hour_counts };

        let json = serde_json::to_value(&event).unwrap();
        assert!(json.get("hourCounts").is_some());
        let counts = &json["hourCounts"];
        assert_eq!(counts["9"], 15);
        assert_eq!(counts["14"], 25);
        assert_eq!(counts["17"], 10);
    }

    #[test]
    fn model_distribution_event_serializes_with_camel_case() {
        let mut model_usage = HashMap::new();
        model_usage.insert(
            "claude-3-opus".to_string(),
            TokenUsageSummary {
                input_tokens: 5000,
                output_tokens: 2500,
                cache_read_tokens: 1000,
                cache_creation_tokens: 500,
            },
        );
        model_usage.insert(
            "claude-3-sonnet".to_string(),
            TokenUsageSummary {
                input_tokens: 3000,
                output_tokens: 1500,
                cache_read_tokens: 600,
                cache_creation_tokens: 300,
            },
        );

        let event = ModelDistributionEvent { model_usage };

        let json = serde_json::to_value(&event).unwrap();
        assert!(json.get("modelUsage").is_some());
        let opus = &json["modelUsage"]["claude-3-opus"];
        assert_eq!(opus["inputTokens"], 5000);
        assert_eq!(opus["outputTokens"], 2500);
    }

    #[test]
    fn todo_progress_event_serializes_with_camel_case() {
        let event = TodoProgressEvent {
            session_id: "sess_789".to_string(),
            completed: 5,
            in_progress: 2,
            pending: 3,
            abandoned: false,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["sessionId"], "sess_789");
        assert_eq!(json["completed"], 5);
        assert_eq!(json["inProgress"], 2);
        assert_eq!(json["pending"], 3);
        assert_eq!(json["abandoned"], false);
    }

    #[test]
    fn file_change_event_serializes_with_camel_case() {
        let event = FileChangeEvent {
            session_id: "sess_abc".to_string(),
            file_hash: "sha256_abc123".to_string(),
            version: 3,
            lines_added: 50,
            lines_removed: 20,
            lines_modified: 15,
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["sessionId"], "sess_abc");
        assert_eq!(json["fileHash"], "sha256_abc123");
        assert_eq!(json["version"], 3);
        assert_eq!(json["linesAdded"], 50);
        assert_eq!(json["linesRemoved"], 20);
        assert_eq!(json["linesModified"], 15);
    }

    #[test]
    fn project_activity_event_serializes_with_camel_case() {
        let event = ProjectActivityEvent {
            project_path: "/home/user/projects/vibetea".to_string(),
            session_id: "sess_xyz".to_string(),
            is_active: true,
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["projectPath"], "/home/user/projects/vibetea");
        assert_eq!(json["sessionId"], "sess_xyz");
        assert_eq!(json["isActive"], true);
    }

    #[test]
    fn event_payload_agent_spawn_serializes_correctly() {
        let payload = EventPayload::AgentSpawn(AgentSpawnEvent {
            session_id: "sess_123".to_string(),
            agent_type: "task".to_string(),
            description: "Run tests".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        });

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "agent_spawn");
        assert_eq!(json["sessionId"], "sess_123");
        assert_eq!(json["agentType"], "task");
    }

    #[test]
    fn event_payload_token_usage_serializes_correctly() {
        let payload = EventPayload::TokenUsage(TokenUsageEvent {
            model: "claude-3-opus".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 200,
            cache_creation_tokens: 100,
        });

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "token_usage");
        assert_eq!(json["model"], "claude-3-opus");
        assert_eq!(json["inputTokens"], 1000);
    }

    #[test]
    fn event_payload_activity_pattern_roundtrip() {
        let mut hour_counts = HashMap::new();
        hour_counts.insert("9".to_string(), 15);
        hour_counts.insert("14".to_string(), 25);

        let original = EventPayload::ActivityPattern(ActivityPatternEvent { hour_counts });

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: EventPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn event_payload_model_distribution_roundtrip() {
        let mut model_usage = HashMap::new();
        model_usage.insert(
            "claude-3-opus".to_string(),
            TokenUsageSummary {
                input_tokens: 5000,
                output_tokens: 2500,
                cache_read_tokens: 1000,
                cache_creation_tokens: 500,
            },
        );

        let original = EventPayload::ModelDistribution(ModelDistributionEvent { model_usage });

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: EventPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn full_event_with_enhanced_payload_roundtrip() {
        let original = Event::new(
            "test-monitor".to_string(),
            EventType::TodoProgress,
            EventPayload::TodoProgress(TodoProgressEvent {
                session_id: "sess_test".to_string(),
                completed: 5,
                in_progress: 2,
                pending: 3,
                abandoned: false,
            }),
        );

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.source, deserialized.source);
        assert_eq!(original.event_type, deserialized.event_type);
        assert_eq!(original.payload, deserialized.payload);
    }
}
