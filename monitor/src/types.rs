//! Event types for VibeTea session monitoring.
//!
//! This module defines the shared event schema used for communication between
//! the monitor and server components. All types serialize to camelCase JSON.

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
}
