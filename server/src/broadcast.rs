//! Event broadcasting infrastructure for the VibeTea server.
//!
//! This module provides the core event distribution mechanism, enabling real-time
//! broadcasting of events to multiple WebSocket subscribers. It uses tokio's
//! broadcast channel for efficient multi-producer, multi-consumer communication.
//!
//! # Architecture
//!
//! The broadcast system consists of two main components:
//!
//! - [`EventBroadcaster`] - The central hub that distributes events to all subscribers
//! - [`SubscriberFilter`] - Optional filtering criteria for subscribers to receive
//!   only events they care about
//!
//! # Example
//!
//! ```rust
//! use vibetea_server::broadcast::{EventBroadcaster, SubscriberFilter};
//! use vibetea_server::types::{Event, EventType, EventPayload, SessionAction};
//! use chrono::Utc;
//! use uuid::Uuid;
//!
//! // Create a broadcaster
//! let broadcaster = EventBroadcaster::new();
//!
//! // Subscribe to receive events
//! let mut rx = broadcaster.subscribe();
//!
//! // Create and broadcast an event
//! let event = Event {
//!     id: "evt_k7m2n9p4q1r6s3t8u5v0".to_string(),
//!     source: "monitor-1".to_string(),
//!     timestamp: Utc::now(),
//!     event_type: EventType::Session,
//!     payload: EventPayload::Session {
//!         session_id: Uuid::new_v4(),
//!         action: SessionAction::Started,
//!         project: "my-project".to_string(),
//!     },
//! };
//!
//! broadcaster.broadcast(event.clone());
//!
//! // Filter events by criteria
//! let filter = SubscriberFilter::new()
//!     .with_event_type(EventType::Session)
//!     .with_project("my-project");
//!
//! assert!(filter.matches(&event));
//! ```

use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, trace, warn};

use crate::types::{Event, EventPayload, EventType};

/// Default channel capacity for high-throughput event distribution.
///
/// This value (1000) provides a balance between memory usage and the ability
/// to handle burst traffic. If subscribers fall too far behind, they will
/// start receiving `RecvError::Lagged` errors indicating missed events.
pub const DEFAULT_CHANNEL_CAPACITY: usize = 1000;

/// Central event distribution hub for broadcasting events to multiple subscribers.
///
/// `EventBroadcaster` wraps a tokio broadcast channel and provides a simple
/// interface for sending events to all connected WebSocket clients. It is
/// designed to be cloned and shared across multiple tasks, as the underlying
/// sender is reference-counted.
///
/// # Thread Safety
///
/// `EventBroadcaster` is `Clone`, `Send`, and `Sync`, making it safe to share
/// across multiple async tasks and threads.
///
/// # Channel Capacity
///
/// The broadcast channel has a fixed capacity. When the channel is full and
/// a new event is broadcast, the oldest event is dropped and slow receivers
/// will receive a `RecvError::Lagged` error on their next receive attempt.
///
/// # Example
///
/// ```rust
/// use vibetea_server::broadcast::EventBroadcaster;
///
/// // Create a broadcaster with default capacity
/// let broadcaster = EventBroadcaster::new();
///
/// // Create a broadcaster with custom capacity
/// let broadcaster = EventBroadcaster::with_capacity(500);
///
/// // Clone the broadcaster for use in another task
/// let broadcaster_clone = broadcaster.clone();
/// ```
#[derive(Debug, Clone)]
pub struct EventBroadcaster {
    sender: Sender<Event>,
}

impl EventBroadcaster {
    /// Creates a new `EventBroadcaster` with the default channel capacity.
    ///
    /// The default capacity is [`DEFAULT_CHANNEL_CAPACITY`] (1000 events).
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::EventBroadcaster;
    ///
    /// let broadcaster = EventBroadcaster::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHANNEL_CAPACITY)
    }

    /// Creates a new `EventBroadcaster` with the specified channel capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of events the channel can hold before
    ///   slow receivers start missing events.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is 0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::EventBroadcaster;
    ///
    /// // Create a broadcaster with smaller capacity for testing
    /// let broadcaster = EventBroadcaster::with_capacity(100);
    /// ```
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        debug!(capacity, "Created event broadcaster");
        Self { sender }
    }

    /// Subscribes to receive broadcast events.
    ///
    /// Returns a `Receiver` that will receive all events broadcast after
    /// the subscription is created. Events broadcast before subscribing
    /// will not be received.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::EventBroadcaster;
    ///
    /// let broadcaster = EventBroadcaster::new();
    /// let mut rx = broadcaster.subscribe();
    ///
    /// // In an async context:
    /// // while let Ok(event) = rx.recv().await {
    /// //     println!("Received event: {:?}", event);
    /// // }
    /// ```
    #[must_use]
    pub fn subscribe(&self) -> Receiver<Event> {
        let rx = self.sender.subscribe();
        debug!(
            subscriber_count = self.subscriber_count(),
            "New subscriber added"
        );
        rx
    }

    /// Broadcasts an event to all current subscribers.
    ///
    /// Returns the number of subscribers that received the event, or 0 if
    /// there are no active subscribers. This method never blocks; if the
    /// channel is full, the oldest event is dropped to make room.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to broadcast to all subscribers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::EventBroadcaster;
    /// use vibetea_server::types::{Event, EventType, EventPayload};
    /// use chrono::Utc;
    /// use uuid::Uuid;
    ///
    /// let broadcaster = EventBroadcaster::new();
    /// let _rx = broadcaster.subscribe();
    ///
    /// let event = Event {
    ///     id: "evt_k7m2n9p4q1r6s3t8u5v0".to_string(),
    ///     source: "monitor-1".to_string(),
    ///     timestamp: Utc::now(),
    ///     event_type: EventType::Activity,
    ///     payload: EventPayload::Activity {
    ///         session_id: Uuid::new_v4(),
    ///         project: None,
    ///     },
    /// };
    ///
    /// let receivers = broadcaster.broadcast(event);
    /// assert_eq!(receivers, 1);
    /// ```
    pub fn broadcast(&self, event: Event) -> usize {
        trace!(
            event_id = %event.id,
            event_type = ?event.event_type,
            source = %event.source,
            "Broadcasting event"
        );

        match self.sender.send(event) {
            Ok(receivers) => {
                trace!(receivers, "Event broadcast successful");
                receivers
            }
            Err(_) => {
                // This happens when there are no active receivers
                warn!("No active subscribers to receive event");
                0
            }
        }
    }

    /// Returns the current number of active subscribers.
    ///
    /// This count includes all receivers that have been created via
    /// [`subscribe()`](Self::subscribe) and have not yet been dropped.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::EventBroadcaster;
    ///
    /// let broadcaster = EventBroadcaster::new();
    /// assert_eq!(broadcaster.subscriber_count(), 0);
    ///
    /// let _rx1 = broadcaster.subscribe();
    /// assert_eq!(broadcaster.subscriber_count(), 1);
    ///
    /// let _rx2 = broadcaster.subscribe();
    /// assert_eq!(broadcaster.subscriber_count(), 2);
    /// ```
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter criteria for selecting which events a subscriber receives.
///
/// `SubscriberFilter` allows clients to specify optional filtering criteria
/// so they only receive events they're interested in. All specified filters
/// use AND logic - an event must match ALL specified criteria to pass.
///
/// Unset filters (None) always match, allowing for flexible partial filtering.
///
/// # Example
///
/// ```rust
/// use vibetea_server::broadcast::SubscriberFilter;
/// use vibetea_server::types::EventType;
///
/// // Filter for tool events from a specific source
/// let filter = SubscriberFilter::new()
///     .with_source("monitor-1")
///     .with_event_type(EventType::Tool);
///
/// // Filter for all events from a specific project
/// let filter = SubscriberFilter::new()
///     .with_project("my-app");
///
/// // Empty filter matches all events
/// let filter = SubscriberFilter::new();
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubscriberFilter {
    /// Filter by source ID (monitor identifier).
    pub source: Option<String>,

    /// Filter by event type.
    pub event_type: Option<EventType>,

    /// Filter by project name.
    pub project: Option<String>,
}

impl SubscriberFilter {
    /// Creates a new empty filter that matches all events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::SubscriberFilter;
    ///
    /// let filter = SubscriberFilter::new();
    /// assert!(filter.source.is_none());
    /// assert!(filter.event_type.is_none());
    /// assert!(filter.project.is_none());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the source filter (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `source` - The source ID to filter by.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::SubscriberFilter;
    ///
    /// let filter = SubscriberFilter::new().with_source("monitor-1");
    /// assert_eq!(filter.source, Some("monitor-1".to_string()));
    /// ```
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Sets the event type filter (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `event_type` - The event type to filter by.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::SubscriberFilter;
    /// use vibetea_server::types::EventType;
    ///
    /// let filter = SubscriberFilter::new().with_event_type(EventType::Session);
    /// assert_eq!(filter.event_type, Some(EventType::Session));
    /// ```
    #[must_use]
    pub fn with_event_type(mut self, event_type: EventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Sets the project filter (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `project` - The project name to filter by.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::SubscriberFilter;
    ///
    /// let filter = SubscriberFilter::new().with_project("vibetea");
    /// assert_eq!(filter.project, Some("vibetea".to_string()));
    /// ```
    #[must_use]
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Checks if an event matches this filter's criteria.
    ///
    /// Returns `true` if the event matches ALL specified filter criteria.
    /// Unset filters (None) always match. This implements AND logic across
    /// all filter fields.
    ///
    /// # Filter Matching Rules
    ///
    /// - **source**: Matches if filter source is None OR equals event.source
    /// - **event_type**: Matches if filter event_type is None OR equals event.event_type
    /// - **project**: Matches if filter project is None OR the event's payload
    ///   contains a matching project field
    ///
    /// # Project Field Extraction
    ///
    /// The project field is extracted from the event payload based on the
    /// payload variant:
    ///
    /// - `Session`: Always has a project field
    /// - `Tool`: Optional project field
    /// - `Activity`: Optional project field
    /// - `Agent`, `Summary`, `Error`: No project field (will only match if
    ///   filter.project is None)
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::SubscriberFilter;
    /// use vibetea_server::types::{Event, EventType, EventPayload, SessionAction};
    /// use chrono::Utc;
    /// use uuid::Uuid;
    ///
    /// let event = Event {
    ///     id: "evt_test1234567890abcdef".to_string(),
    ///     source: "monitor-1".to_string(),
    ///     timestamp: Utc::now(),
    ///     event_type: EventType::Session,
    ///     payload: EventPayload::Session {
    ///         session_id: Uuid::new_v4(),
    ///         action: SessionAction::Started,
    ///         project: "my-project".to_string(),
    ///     },
    /// };
    ///
    /// // Empty filter matches all events
    /// assert!(SubscriberFilter::new().matches(&event));
    ///
    /// // Source filter
    /// assert!(SubscriberFilter::new().with_source("monitor-1").matches(&event));
    /// assert!(!SubscriberFilter::new().with_source("other").matches(&event));
    ///
    /// // Event type filter
    /// assert!(SubscriberFilter::new().with_event_type(EventType::Session).matches(&event));
    /// assert!(!SubscriberFilter::new().with_event_type(EventType::Tool).matches(&event));
    ///
    /// // Project filter
    /// assert!(SubscriberFilter::new().with_project("my-project").matches(&event));
    /// assert!(!SubscriberFilter::new().with_project("other").matches(&event));
    /// ```
    #[must_use]
    pub fn matches(&self, event: &Event) -> bool {
        // Check source filter
        if let Some(ref filter_source) = self.source {
            if &event.source != filter_source {
                return false;
            }
        }

        // Check event type filter
        if let Some(filter_event_type) = self.event_type {
            if event.event_type != filter_event_type {
                return false;
            }
        }

        // Check project filter
        if let Some(ref filter_project) = self.project {
            let event_project = Self::extract_project(&event.payload);
            match event_project {
                Some(project) if project == filter_project => {}
                _ => return false,
            }
        }

        true
    }

    /// Extracts the project field from an event payload, if present.
    fn extract_project(payload: &EventPayload) -> Option<&str> {
        match payload {
            EventPayload::Session { project, .. } => Some(project.as_str()),
            EventPayload::Tool { project, .. } => project.as_deref(),
            EventPayload::Activity { project, .. } => project.as_deref(),
            EventPayload::Agent { .. }
            | EventPayload::Summary { .. }
            | EventPayload::Error { .. } => None,
        }
    }

    /// Returns `true` if this filter has no criteria set (matches all events).
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::broadcast::SubscriberFilter;
    /// use vibetea_server::types::EventType;
    ///
    /// assert!(SubscriberFilter::new().is_empty());
    /// assert!(!SubscriberFilter::new().with_source("test").is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.source.is_none() && self.event_type.is_none() && self.project.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SessionAction, ToolStatus};
    use chrono::Utc;
    use uuid::Uuid;

    /// Helper to create a test event with customizable fields.
    fn make_event(source: &str, event_type: EventType, payload: EventPayload) -> Event {
        Event {
            id: format!("evt_test{:0>16}", rand_id()),
            source: source.to_string(),
            timestamp: Utc::now(),
            event_type,
            payload,
        }
    }

    /// Generate a simple pseudo-random ID for test events.
    fn rand_id() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn make_session_event(source: &str, project: &str) -> Event {
        make_event(
            source,
            EventType::Session,
            EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: project.to_string(),
            },
        )
    }

    fn make_tool_event(source: &str, project: Option<&str>) -> Event {
        make_event(
            source,
            EventType::Tool,
            EventPayload::Tool {
                session_id: Uuid::new_v4(),
                tool: "Read".to_string(),
                status: ToolStatus::Completed,
                context: None,
                project: project.map(String::from),
            },
        )
    }

    fn make_activity_event(source: &str, project: Option<&str>) -> Event {
        make_event(
            source,
            EventType::Activity,
            EventPayload::Activity {
                session_id: Uuid::new_v4(),
                project: project.map(String::from),
            },
        )
    }

    fn make_agent_event(source: &str) -> Event {
        make_event(
            source,
            EventType::Agent,
            EventPayload::Agent {
                session_id: Uuid::new_v4(),
                state: "thinking".to_string(),
            },
        )
    }

    fn make_summary_event(source: &str) -> Event {
        make_event(
            source,
            EventType::Summary,
            EventPayload::Summary {
                session_id: Uuid::new_v4(),
                summary: "Session completed".to_string(),
            },
        )
    }

    fn make_error_event(source: &str) -> Event {
        make_event(
            source,
            EventType::Error,
            EventPayload::Error {
                session_id: Uuid::new_v4(),
                category: "network".to_string(),
            },
        )
    }

    // ========================================================================
    // EventBroadcaster tests
    // ========================================================================

    #[test]
    fn broadcaster_new_creates_with_default_capacity() {
        let broadcaster = EventBroadcaster::new();
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[test]
    fn broadcaster_with_capacity_creates_custom_capacity() {
        let broadcaster = EventBroadcaster::with_capacity(500);
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[test]
    fn broadcaster_default_creates_new() {
        let broadcaster = EventBroadcaster::default();
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[test]
    fn broadcaster_subscribe_increases_count() {
        let broadcaster = EventBroadcaster::new();
        assert_eq!(broadcaster.subscriber_count(), 0);

        let _rx1 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 1);

        let _rx2 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 2);
    }

    #[test]
    fn broadcaster_subscriber_count_decreases_on_drop() {
        let broadcaster = EventBroadcaster::new();

        let rx1 = broadcaster.subscribe();
        let rx2 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 2);

        drop(rx1);
        assert_eq!(broadcaster.subscriber_count(), 1);

        drop(rx2);
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[test]
    fn broadcaster_clone_shares_channel() {
        let broadcaster1 = EventBroadcaster::new();
        let broadcaster2 = broadcaster1.clone();

        let _rx = broadcaster1.subscribe();
        assert_eq!(broadcaster1.subscriber_count(), 1);
        assert_eq!(broadcaster2.subscriber_count(), 1);
    }

    #[test]
    fn broadcaster_broadcast_returns_zero_with_no_subscribers() {
        let broadcaster = EventBroadcaster::new();
        let event = make_session_event("monitor-1", "test-project");

        let receivers = broadcaster.broadcast(event);
        assert_eq!(receivers, 0);
    }

    #[tokio::test]
    async fn broadcaster_broadcast_to_single_subscriber() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        let event = make_session_event("monitor-1", "test-project");
        let event_clone = event.clone();

        let receivers = broadcaster.broadcast(event);
        assert_eq!(receivers, 1);

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, event_clone.id);
        assert_eq!(received.source, "monitor-1");
    }

    #[tokio::test]
    async fn broadcaster_broadcast_to_multiple_subscribers() {
        let broadcaster = EventBroadcaster::new();
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();
        let mut rx3 = broadcaster.subscribe();

        let event = make_session_event("monitor-1", "test-project");
        let event_id = event.id.clone();

        let receivers = broadcaster.broadcast(event);
        assert_eq!(receivers, 3);

        // All subscribers should receive the same event
        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();
        let received3 = rx3.recv().await.unwrap();

        assert_eq!(received1.id, event_id);
        assert_eq!(received2.id, event_id);
        assert_eq!(received3.id, event_id);
    }

    #[tokio::test]
    async fn broadcaster_multiple_events_in_order() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        let event1 = make_session_event("monitor-1", "project-1");
        let event2 = make_tool_event("monitor-2", Some("project-2"));
        let event3 = make_activity_event("monitor-3", None);

        let id1 = event1.id.clone();
        let id2 = event2.id.clone();
        let id3 = event3.id.clone();

        broadcaster.broadcast(event1);
        broadcaster.broadcast(event2);
        broadcaster.broadcast(event3);

        assert_eq!(rx.recv().await.unwrap().id, id1);
        assert_eq!(rx.recv().await.unwrap().id, id2);
        assert_eq!(rx.recv().await.unwrap().id, id3);
    }

    #[tokio::test]
    async fn broadcaster_clone_can_broadcast() {
        let broadcaster1 = EventBroadcaster::new();
        let broadcaster2 = broadcaster1.clone();

        let mut rx = broadcaster1.subscribe();

        let event1 = make_session_event("from-1", "project");
        let event2 = make_session_event("from-2", "project");
        let id1 = event1.id.clone();
        let id2 = event2.id.clone();

        // Broadcast from both clones
        broadcaster1.broadcast(event1);
        broadcaster2.broadcast(event2);

        // Receiver should get both events
        let received1 = rx.recv().await.unwrap();
        let received2 = rx.recv().await.unwrap();

        assert_eq!(received1.id, id1);
        assert_eq!(received2.id, id2);
    }

    // ========================================================================
    // SubscriberFilter tests
    // ========================================================================

    #[test]
    fn filter_new_creates_empty_filter() {
        let filter = SubscriberFilter::new();
        assert!(filter.source.is_none());
        assert!(filter.event_type.is_none());
        assert!(filter.project.is_none());
        assert!(filter.is_empty());
    }

    #[test]
    fn filter_default_creates_empty_filter() {
        let filter = SubscriberFilter::default();
        assert!(filter.is_empty());
    }

    #[test]
    fn filter_with_source_sets_source() {
        let filter = SubscriberFilter::new().with_source("monitor-1");
        assert_eq!(filter.source, Some("monitor-1".to_string()));
        assert!(!filter.is_empty());
    }

    #[test]
    fn filter_with_event_type_sets_event_type() {
        let filter = SubscriberFilter::new().with_event_type(EventType::Tool);
        assert_eq!(filter.event_type, Some(EventType::Tool));
        assert!(!filter.is_empty());
    }

    #[test]
    fn filter_with_project_sets_project() {
        let filter = SubscriberFilter::new().with_project("my-project");
        assert_eq!(filter.project, Some("my-project".to_string()));
        assert!(!filter.is_empty());
    }

    #[test]
    fn filter_builder_chain() {
        let filter = SubscriberFilter::new()
            .with_source("monitor-1")
            .with_event_type(EventType::Session)
            .with_project("vibetea");

        assert_eq!(filter.source, Some("monitor-1".to_string()));
        assert_eq!(filter.event_type, Some(EventType::Session));
        assert_eq!(filter.project, Some("vibetea".to_string()));
    }

    #[test]
    fn filter_clone_and_eq() {
        let filter1 = SubscriberFilter::new()
            .with_source("test")
            .with_event_type(EventType::Tool);
        let filter2 = filter1.clone();
        assert_eq!(filter1, filter2);
    }

    // ========================================================================
    // SubscriberFilter::matches tests
    // ========================================================================

    #[test]
    fn empty_filter_matches_all_events() {
        let filter = SubscriberFilter::new();

        assert!(filter.matches(&make_session_event("any", "any")));
        assert!(filter.matches(&make_tool_event("any", Some("any"))));
        assert!(filter.matches(&make_tool_event("any", None)));
        assert!(filter.matches(&make_activity_event("any", Some("any"))));
        assert!(filter.matches(&make_activity_event("any", None)));
        assert!(filter.matches(&make_agent_event("any")));
        assert!(filter.matches(&make_summary_event("any")));
        assert!(filter.matches(&make_error_event("any")));
    }

    #[test]
    fn source_filter_matches_correct_source() {
        let filter = SubscriberFilter::new().with_source("monitor-1");

        assert!(filter.matches(&make_session_event("monitor-1", "project")));
        assert!(!filter.matches(&make_session_event("monitor-2", "project")));
        assert!(!filter.matches(&make_session_event("other", "project")));
    }

    #[test]
    fn event_type_filter_matches_correct_type() {
        let session_filter = SubscriberFilter::new().with_event_type(EventType::Session);
        let tool_filter = SubscriberFilter::new().with_event_type(EventType::Tool);
        let activity_filter = SubscriberFilter::new().with_event_type(EventType::Activity);
        let agent_filter = SubscriberFilter::new().with_event_type(EventType::Agent);
        let summary_filter = SubscriberFilter::new().with_event_type(EventType::Summary);
        let error_filter = SubscriberFilter::new().with_event_type(EventType::Error);

        let session_event = make_session_event("m", "p");
        let tool_event = make_tool_event("m", None);
        let activity_event = make_activity_event("m", None);
        let agent_event = make_agent_event("m");
        let summary_event = make_summary_event("m");
        let error_event = make_error_event("m");

        // Session filter
        assert!(session_filter.matches(&session_event));
        assert!(!session_filter.matches(&tool_event));
        assert!(!session_filter.matches(&activity_event));

        // Tool filter
        assert!(tool_filter.matches(&tool_event));
        assert!(!tool_filter.matches(&session_event));
        assert!(!tool_filter.matches(&activity_event));

        // Activity filter
        assert!(activity_filter.matches(&activity_event));
        assert!(!activity_filter.matches(&session_event));
        assert!(!activity_filter.matches(&tool_event));

        // Agent filter
        assert!(agent_filter.matches(&agent_event));
        assert!(!agent_filter.matches(&session_event));

        // Summary filter
        assert!(summary_filter.matches(&summary_event));
        assert!(!summary_filter.matches(&session_event));

        // Error filter
        assert!(error_filter.matches(&error_event));
        assert!(!error_filter.matches(&session_event));
    }

    #[test]
    fn project_filter_matches_session_events() {
        let filter = SubscriberFilter::new().with_project("vibetea");

        assert!(filter.matches(&make_session_event("m", "vibetea")));
        assert!(!filter.matches(&make_session_event("m", "other-project")));
    }

    #[test]
    fn project_filter_matches_tool_events_with_project() {
        let filter = SubscriberFilter::new().with_project("vibetea");

        assert!(filter.matches(&make_tool_event("m", Some("vibetea"))));
        assert!(!filter.matches(&make_tool_event("m", Some("other"))));
        assert!(!filter.matches(&make_tool_event("m", None)));
    }

    #[test]
    fn project_filter_matches_activity_events_with_project() {
        let filter = SubscriberFilter::new().with_project("vibetea");

        assert!(filter.matches(&make_activity_event("m", Some("vibetea"))));
        assert!(!filter.matches(&make_activity_event("m", Some("other"))));
        assert!(!filter.matches(&make_activity_event("m", None)));
    }

    #[test]
    fn project_filter_does_not_match_events_without_project_field() {
        let filter = SubscriberFilter::new().with_project("vibetea");

        // Agent, Summary, and Error events don't have project fields
        assert!(!filter.matches(&make_agent_event("m")));
        assert!(!filter.matches(&make_summary_event("m")));
        assert!(!filter.matches(&make_error_event("m")));
    }

    #[test]
    fn combined_filters_use_and_logic() {
        let filter = SubscriberFilter::new()
            .with_source("monitor-1")
            .with_event_type(EventType::Session)
            .with_project("vibetea");

        // All criteria match
        assert!(filter.matches(&make_event(
            "monitor-1",
            EventType::Session,
            EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "vibetea".to_string(),
            }
        )));

        // Wrong source
        assert!(!filter.matches(&make_event(
            "monitor-2",
            EventType::Session,
            EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "vibetea".to_string(),
            }
        )));

        // Wrong event type
        assert!(!filter.matches(&make_event(
            "monitor-1",
            EventType::Tool,
            EventPayload::Tool {
                session_id: Uuid::new_v4(),
                tool: "Read".to_string(),
                status: ToolStatus::Completed,
                context: None,
                project: Some("vibetea".to_string()),
            }
        )));

        // Wrong project
        assert!(!filter.matches(&make_event(
            "monitor-1",
            EventType::Session,
            EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "other".to_string(),
            }
        )));
    }

    #[test]
    fn source_and_event_type_filter() {
        let filter = SubscriberFilter::new()
            .with_source("monitor-1")
            .with_event_type(EventType::Tool);

        // Matches: correct source and type
        assert!(filter.matches(&make_tool_event("monitor-1", None)));
        assert!(filter.matches(&make_tool_event("monitor-1", Some("any"))));

        // Doesn't match: wrong source
        assert!(!filter.matches(&make_tool_event("monitor-2", None)));

        // Doesn't match: wrong type
        assert!(!filter.matches(&make_session_event("monitor-1", "any")));
    }

    #[test]
    fn source_and_project_filter() {
        let filter = SubscriberFilter::new()
            .with_source("monitor-1")
            .with_project("vibetea");

        // Matches
        assert!(filter.matches(&make_session_event("monitor-1", "vibetea")));
        assert!(filter.matches(&make_tool_event("monitor-1", Some("vibetea"))));

        // Doesn't match: wrong source
        assert!(!filter.matches(&make_session_event("other", "vibetea")));

        // Doesn't match: wrong project
        assert!(!filter.matches(&make_session_event("monitor-1", "other")));
    }

    #[test]
    fn event_type_and_project_filter() {
        let filter = SubscriberFilter::new()
            .with_event_type(EventType::Session)
            .with_project("vibetea");

        // Matches
        assert!(filter.matches(&make_session_event("any", "vibetea")));

        // Doesn't match: wrong type
        assert!(!filter.matches(&make_tool_event("any", Some("vibetea"))));

        // Doesn't match: wrong project
        assert!(!filter.matches(&make_session_event("any", "other")));
    }

    #[test]
    fn filter_is_empty_check() {
        assert!(SubscriberFilter::new().is_empty());
        assert!(!SubscriberFilter::new().with_source("x").is_empty());
        assert!(!SubscriberFilter::new()
            .with_event_type(EventType::Tool)
            .is_empty());
        assert!(!SubscriberFilter::new().with_project("x").is_empty());
        assert!(!SubscriberFilter::new()
            .with_source("x")
            .with_event_type(EventType::Tool)
            .is_empty());
    }

    #[test]
    fn filter_debug_format() {
        let filter = SubscriberFilter::new()
            .with_source("monitor-1")
            .with_event_type(EventType::Session);

        let debug = format!("{:?}", filter);
        assert!(debug.contains("SubscriberFilter"));
        assert!(debug.contains("monitor-1"));
        assert!(debug.contains("Session"));
    }
}
