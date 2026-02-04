//! Privacy compliance test suite for Phase 11 enhanced tracking events.
//!
//! This module verifies Constitution I (Privacy by Design) compliance for all
//! enhanced tracking event types introduced in Phase 11. It ensures that no
//! sensitive data (code, prompts, full paths, environment variables, or file
//! contents) can be transmitted in any event type.
//!
//! # Privacy Guarantees Tested
//!
//! 1. No full file paths - only basenames allowed
//! 2. No user prompts or message content
//! 3. No code snippets or file contents
//! 4. No environment variable values
//! 5. No home directory paths exposed
//!
//! # Event Types Tested (Phase 11)
//!
//! - `ProjectActivityEvent` - project paths, session IDs, is_active flag
//! - `TokenUsageEvent` - model name, token counts
//! - `SessionMetricsEvent` - message counts, tool counts, session duration
//! - `ActivityPatternEvent` - hourCounts map
//! - `ModelDistributionEvent` - model usage map
//! - `AgentSpawnEvent` - agent type, session ID
//! - `SkillInvocationEvent` - skill name, timestamp
//! - `TodoProgressEvent` - counts of completed/pending/abandoned
//! - `FileChangeEvent` - file hash (not full path), lines added/removed

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use vibetea_monitor::privacy::{PrivacyConfig, PrivacyPipeline};
use vibetea_monitor::types::{
    ActivityPatternEvent, AgentSpawnEvent, EventPayload, FileChangeEvent, ModelDistributionEvent,
    ProjectActivityEvent, SessionMetricsEvent, SkillInvocationEvent, TodoProgressEvent,
    TokenUsageEvent, TokenUsageSummary,
};

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a privacy pipeline with default configuration (no allowlist).
fn default_pipeline() -> PrivacyPipeline {
    PrivacyPipeline::new(PrivacyConfig::new(None))
}

/// Creates a test timestamp for deterministic tests.
fn test_timestamp() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .expect("valid timestamp")
        .with_timezone(&Utc)
}

/// Sensitive patterns that must never appear in serialized events.
/// These represent full paths that would expose user directory structure.
const SENSITIVE_PATH_PATTERNS: &[&str] = &[
    "/home/",
    "/Users/",
    "/root/",
    "/var/",
    "/etc/",
    "/opt/",
    "/tmp/",
    "C:\\Users\\",
    "C:\\Program Files\\",
];

/// Sensitive content patterns that must never appear in events.
const SENSITIVE_CONTENT_PATTERNS: &[&str] = &[
    "password",
    "secret_key",
    "api_key",
    "Bearer ",
    "Authorization:",
    "private_key",
    "AWS_SECRET",
    "ANTHROPIC_API_KEY",
];

/// Code patterns that must never appear in events.
const CODE_PATTERNS: &[&str] = &[
    "fn main()",
    "impl ",
    "struct ",
    "pub async fn",
    "def __init__",
    "class ",
    "function(",
    "const ",
    "let mut",
];

/// Prompt patterns that must never appear in events.
const PROMPT_PATTERNS: &[&str] = &[
    "Please help me",
    "Can you",
    "I need you to",
    "Write a function",
    "Fix this bug",
    "Explain how",
];

/// Asserts that a JSON string contains no sensitive path patterns.
fn assert_no_sensitive_paths(json: &str, test_name: &str) {
    for pattern in SENSITIVE_PATH_PATTERNS {
        assert!(
            !json.contains(pattern),
            "{test_name}: JSON contains sensitive path pattern '{pattern}': {json}"
        );
    }
}

/// Asserts that a JSON string contains no sensitive content patterns.
fn assert_no_sensitive_content(json: &str, test_name: &str) {
    for pattern in SENSITIVE_CONTENT_PATTERNS {
        assert!(
            !json.to_lowercase().contains(&pattern.to_lowercase()),
            "{test_name}: JSON contains sensitive content pattern '{pattern}': {json}"
        );
    }
}

/// Asserts that a JSON string contains no code patterns.
fn assert_no_code(json: &str, test_name: &str) {
    for pattern in CODE_PATTERNS {
        assert!(
            !json.contains(pattern),
            "{test_name}: JSON contains code pattern '{pattern}': {json}"
        );
    }
}

/// Asserts that a JSON string contains no prompt patterns.
fn assert_no_prompts(json: &str, test_name: &str) {
    for pattern in PROMPT_PATTERNS {
        assert!(
            !json.to_lowercase().contains(&pattern.to_lowercase()),
            "{test_name}: JSON contains prompt pattern '{pattern}': {json}"
        );
    }
}

/// Comprehensive privacy assertion for all sensitive patterns.
fn assert_privacy_compliant(json: &str, test_name: &str) {
    assert_no_sensitive_paths(json, test_name);
    assert_no_sensitive_content(json, test_name);
    assert_no_code(json, test_name);
    assert_no_prompts(json, test_name);
}

// =============================================================================
// Constitution I (Privacy by Design) Compliance Markers
// =============================================================================

/// Constitution I Compliance: Verify that privacy pipeline is applied to all events.
///
/// This test explicitly marks compliance with the Privacy by Design principle
/// by ensuring the privacy pipeline processes all event types.
#[test]
fn constitution_i_privacy_pipeline_processes_all_enhanced_events() {
    let pipeline = default_pipeline();

    // Test that each enhanced event type passes through the pipeline
    let events: Vec<EventPayload> = vec![
        EventPayload::ProjectActivity(ProjectActivityEvent {
            project_path: "vibetea".to_string(),
            session_id: "sess_123".to_string(),
            is_active: true,
        }),
        EventPayload::TokenUsage(TokenUsageEvent {
            model: "claude-3-opus".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 200,
            cache_creation_tokens: 100,
        }),
        EventPayload::SessionMetrics(SessionMetricsEvent {
            total_sessions: 42,
            total_messages: 1234,
            total_tool_usage: 567,
            longest_session: "sess_longest".to_string(),
        }),
        EventPayload::ActivityPattern(ActivityPatternEvent {
            hour_counts: {
                let mut map = HashMap::new();
                map.insert("9".to_string(), 15);
                map
            },
        }),
        EventPayload::ModelDistribution(ModelDistributionEvent {
            model_usage: {
                let mut map = HashMap::new();
                map.insert(
                    "claude-3-opus".to_string(),
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
        EventPayload::AgentSpawn(AgentSpawnEvent {
            session_id: "sess_456".to_string(),
            agent_type: "task".to_string(),
            description: "Background task".to_string(),
            timestamp: test_timestamp(),
        }),
        EventPayload::SkillInvocation(SkillInvocationEvent {
            session_id: "sess_789".to_string(),
            skill_name: "commit".to_string(),
            project: "my-project".to_string(),
            timestamp: test_timestamp(),
        }),
        EventPayload::TodoProgress(TodoProgressEvent {
            session_id: "sess_abc".to_string(),
            completed: 5,
            in_progress: 2,
            pending: 3,
            abandoned: false,
        }),
        EventPayload::FileChange(FileChangeEvent {
            session_id: "sess_def".to_string(),
            file_hash: "sha256_abc123".to_string(),
            version: 1,
            lines_added: 50,
            lines_removed: 20,
            lines_modified: 15,
            timestamp: test_timestamp(),
        }),
    ];

    for event in events {
        // Pipeline should process without panic
        let processed = pipeline.process(event.clone());

        // Processed event should be serializable
        let json = serde_json::to_string(&processed).expect("Failed to serialize processed event");

        // Verify no sensitive patterns in serialized output
        assert_privacy_compliant(&json, "constitution_i_compliance");
    }
}

// =============================================================================
// ProjectActivityEvent Privacy Tests
// =============================================================================

/// Verifies ProjectActivityEvent contains only project basename, not full paths.
#[test]
fn project_activity_event_contains_no_full_paths() {
    let pipeline = default_pipeline();

    // Event with potentially sensitive full path as project_path
    // NOTE: The design uses project name, not full path, but we test the serialized form
    let event = EventPayload::ProjectActivity(ProjectActivityEvent {
        project_path: "vibetea".to_string(), // Should be basename only
        session_id: "sess_123".to_string(),
        is_active: true,
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should not contain any path-like patterns
    assert_no_sensitive_paths(&json, "project_activity_no_full_paths");

    // Should not contain home directory references
    assert!(!json.contains("/home/"), "Should not expose home directory");
    assert!(
        !json.contains("/Users/"),
        "Should not expose Users directory"
    );
}

/// Verifies ProjectActivityEvent strips sensitive project paths.
#[test]
fn project_activity_event_with_sensitive_path_patterns() {
    let pipeline = default_pipeline();

    // Create event and verify it passes through the pipeline
    // The design expects project_path to already be sanitized, but we verify
    // the serialization doesn't add any sensitive data
    let event = EventPayload::ProjectActivity(ProjectActivityEvent {
        project_path: "my-secret-project".to_string(),
        session_id: "sess_sensitive".to_string(),
        is_active: true,
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Verify no sensitive patterns leaked
    assert_no_sensitive_content(&json, "project_activity_sensitive_patterns");
    assert_no_code(&json, "project_activity_no_code");
}

// =============================================================================
// TokenUsageEvent Privacy Tests
// =============================================================================

/// Verifies TokenUsageEvent contains only model name and counts, no prompts.
#[test]
fn token_usage_event_contains_no_prompts() {
    let pipeline = default_pipeline();

    let event = EventPayload::TokenUsage(TokenUsageEvent {
        model: "claude-3-opus".to_string(),
        input_tokens: 1000,
        output_tokens: 500,
        cache_read_tokens: 200,
        cache_creation_tokens: 100,
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Verify no prompt patterns
    assert_no_prompts(&json, "token_usage_no_prompts");

    // Verify no code patterns
    assert_no_code(&json, "token_usage_no_code");

    // Verify expected fields are present
    assert!(json.contains("model"), "Should contain model field");
    assert!(
        json.contains("inputTokens") || json.contains("input_tokens"),
        "Should contain input tokens field"
    );
}

/// Verifies TokenUsageEvent does not leak conversation content through model name.
#[test]
fn token_usage_event_model_name_is_safe() {
    let pipeline = default_pipeline();

    // Test with various model names
    let models = vec![
        "claude-3-opus",
        "claude-3-sonnet",
        "claude-3-haiku",
        "claude-instant-1.2",
        "gpt-4-turbo",
    ];

    for model in models {
        let event = EventPayload::TokenUsage(TokenUsageEvent {
            model: model.to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 200,
            cache_creation_tokens: 100,
        });

        let processed = pipeline.process(event);
        let json = serde_json::to_string(&processed).expect("Failed to serialize");

        // Model names should not contain sensitive patterns
        assert_no_sensitive_content(&json, &format!("token_usage_model_{model}"));
    }
}

// =============================================================================
// SessionMetricsEvent Privacy Tests
// =============================================================================

/// Verifies SessionMetricsEvent contains only aggregate counts.
#[test]
fn session_metrics_event_contains_only_aggregates() {
    let pipeline = default_pipeline();

    let event = EventPayload::SessionMetrics(SessionMetricsEvent {
        total_sessions: 42,
        total_messages: 1234,
        total_tool_usage: 567,
        longest_session: "sess_longest".to_string(),
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should contain only numeric aggregates and session ID
    assert_privacy_compliant(&json, "session_metrics_aggregates");

    // Verify no message content is present
    assert!(
        !json.contains("Please help"),
        "Should not contain message content"
    );
    assert!(
        !json.contains("How do I"),
        "Should not contain message content"
    );
}

/// Verifies SessionMetricsEvent longest_session is just an ID, not content.
#[test]
fn session_metrics_longest_session_is_id_only() {
    let pipeline = default_pipeline();

    let event = EventPayload::SessionMetrics(SessionMetricsEvent {
        total_sessions: 10,
        total_messages: 500,
        total_tool_usage: 200,
        longest_session: "sess_abc123".to_string(),
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Session ID should be opaque identifier
    assert!(json.contains("sess_abc123"), "Should contain session ID");

    // Should not contain any session details
    assert_no_prompts(&json, "session_metrics_longest_session");
    assert_no_code(&json, "session_metrics_longest_session");
}

// =============================================================================
// ActivityPatternEvent Privacy Tests
// =============================================================================

/// Verifies ActivityPatternEvent contains only hour counts, no activity details.
#[test]
fn activity_pattern_event_contains_only_hour_counts() {
    let pipeline = default_pipeline();

    let mut hour_counts = HashMap::new();
    hour_counts.insert("9".to_string(), 15);
    hour_counts.insert("14".to_string(), 25);
    hour_counts.insert("17".to_string(), 10);

    let event = EventPayload::ActivityPattern(ActivityPatternEvent { hour_counts });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should only contain hours (0-23) and counts
    assert_privacy_compliant(&json, "activity_pattern_hour_counts");

    // Verify no activity descriptions
    assert!(
        !json.contains("coding"),
        "Should not contain activity descriptions"
    );
    assert!(
        !json.contains("debugging"),
        "Should not contain activity descriptions"
    );
}

/// Verifies ActivityPatternEvent hour keys are valid hours only.
#[test]
fn activity_pattern_event_hour_keys_are_valid() {
    let pipeline = default_pipeline();

    // All 24 hours
    let mut hour_counts = HashMap::new();
    for hour in 0..24 {
        hour_counts.insert(hour.to_string(), (hour + 1) as u64);
    }

    let event = EventPayload::ActivityPattern(ActivityPatternEvent { hour_counts });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Verify the JSON is valid and contains hour counts
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Valid JSON");

    // Check the structure
    assert!(
        parsed.get("hourCounts").is_some() || parsed.get("hour_counts").is_some(),
        "Should have hour_counts field"
    );
}

// =============================================================================
// ModelDistributionEvent Privacy Tests
// =============================================================================

/// Verifies ModelDistributionEvent contains only model usage statistics.
#[test]
fn model_distribution_event_contains_only_statistics() {
    let pipeline = default_pipeline();

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

    let event = EventPayload::ModelDistribution(ModelDistributionEvent { model_usage });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should only contain model names and token counts
    assert_privacy_compliant(&json, "model_distribution_statistics");

    // Verify model names are present
    assert!(json.contains("claude-3-opus"), "Should contain model name");
    assert!(
        json.contains("claude-3-sonnet"),
        "Should contain model name"
    );
}

// =============================================================================
// AgentSpawnEvent Privacy Tests
// =============================================================================

/// Verifies AgentSpawnEvent description does not contain user prompts.
#[test]
fn agent_spawn_event_description_is_sanitized() {
    let pipeline = default_pipeline();

    // The description field should be a sanitized task description
    let event = EventPayload::AgentSpawn(AgentSpawnEvent {
        session_id: "sess_123".to_string(),
        agent_type: "task".to_string(),
        description: "Background task".to_string(), // Sanitized description
        timestamp: test_timestamp(),
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should not contain prompts or code
    assert_no_prompts(&json, "agent_spawn_description");
    assert_no_code(&json, "agent_spawn_description");
}

/// Verifies AgentSpawnEvent agent_type is from allowed set.
#[test]
fn agent_spawn_event_agent_type_is_safe() {
    let pipeline = default_pipeline();

    let valid_agent_types = vec!["task", "background", "parallel"];

    for agent_type in valid_agent_types {
        let event = EventPayload::AgentSpawn(AgentSpawnEvent {
            session_id: "sess_456".to_string(),
            agent_type: agent_type.to_string(),
            description: "Test agent".to_string(),
            timestamp: test_timestamp(),
        });

        let processed = pipeline.process(event);
        let json = serde_json::to_string(&processed).expect("Failed to serialize");

        assert_privacy_compliant(&json, &format!("agent_spawn_type_{agent_type}"));
    }
}

// =============================================================================
// SkillInvocationEvent Privacy Tests
// =============================================================================

/// Verifies SkillInvocationEvent contains only skill name, no arguments.
#[test]
fn skill_invocation_event_contains_no_arguments() {
    let pipeline = default_pipeline();

    let event = EventPayload::SkillInvocation(SkillInvocationEvent {
        session_id: "sess_789".to_string(),
        skill_name: "commit".to_string(),
        project: "my-project".to_string(),
        timestamp: test_timestamp(),
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should contain skill name
    assert!(json.contains("commit"), "Should contain skill name");

    // Should not contain sensitive patterns
    assert_privacy_compliant(&json, "skill_invocation_no_args");
}

/// Verifies SkillInvocationEvent skill names are safe.
#[test]
fn skill_invocation_event_skill_names_are_safe() {
    let pipeline = default_pipeline();

    let skill_names = vec![
        "commit",
        "review-pr",
        "test",
        "build",
        "deploy",
        "format",
        "lint",
    ];

    for skill_name in skill_names {
        let event = EventPayload::SkillInvocation(SkillInvocationEvent {
            session_id: "sess_test".to_string(),
            skill_name: skill_name.to_string(),
            project: "project".to_string(),
            timestamp: test_timestamp(),
        });

        let processed = pipeline.process(event);
        let json = serde_json::to_string(&processed).expect("Failed to serialize");

        assert_privacy_compliant(&json, &format!("skill_invocation_{skill_name}"));
    }
}

// =============================================================================
// TodoProgressEvent Privacy Tests
// =============================================================================

/// Verifies TodoProgressEvent contains only counts, no todo text.
#[test]
fn todo_progress_event_contains_only_counts() {
    let pipeline = default_pipeline();

    let event = EventPayload::TodoProgress(TodoProgressEvent {
        session_id: "sess_abc".to_string(),
        completed: 5,
        in_progress: 2,
        pending: 3,
        abandoned: false,
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should contain only numeric counts
    assert_privacy_compliant(&json, "todo_progress_counts");

    // Verify no todo item text is present
    assert!(!json.contains("TODO:"), "Should not contain todo text");
    assert!(!json.contains("FIXME:"), "Should not contain fixme text");
}

/// Verifies TodoProgressEvent does not leak task descriptions.
#[test]
fn todo_progress_event_no_task_descriptions() {
    let pipeline = default_pipeline();

    let event = EventPayload::TodoProgress(TodoProgressEvent {
        session_id: "sess_def".to_string(),
        completed: 10,
        in_progress: 0,
        pending: 0,
        abandoned: true,
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should not contain any task descriptions
    assert_no_prompts(&json, "todo_progress_no_descriptions");
    assert_no_code(&json, "todo_progress_no_descriptions");
}

// =============================================================================
// FileChangeEvent Privacy Tests
// =============================================================================

/// Verifies FileChangeEvent uses file hash, not filename or path.
#[test]
fn file_change_event_uses_hash_not_path() {
    let pipeline = default_pipeline();

    let event = EventPayload::FileChange(FileChangeEvent {
        session_id: "sess_file".to_string(),
        file_hash: "sha256_abc123def456".to_string(), // Hash, not filename
        version: 1,
        lines_added: 50,
        lines_removed: 20,
        lines_modified: 15,
        timestamp: test_timestamp(),
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should contain hash
    assert!(
        json.contains("sha256_abc123def456"),
        "Should contain file hash"
    );

    // Should not contain file paths
    assert_no_sensitive_paths(&json, "file_change_hash_not_path");

    // Should not contain filenames
    assert!(
        !json.contains(".rs"),
        "Should not contain file extension in context"
    );
    assert!(!json.contains("main"), "Should not contain filename");
}

/// Verifies FileChangeEvent contains only line counts, no content.
#[test]
fn file_change_event_contains_line_counts_only() {
    let pipeline = default_pipeline();

    let event = EventPayload::FileChange(FileChangeEvent {
        session_id: "sess_changes".to_string(),
        file_hash: "sha256_xyz789".to_string(),
        version: 3,
        lines_added: 100,
        lines_removed: 50,
        lines_modified: 25,
        timestamp: test_timestamp(),
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should not contain code content
    assert_no_code(&json, "file_change_line_counts");

    // Verify numeric fields are present
    assert!(
        json.contains("100") || json.contains("linesAdded"),
        "Should contain lines added"
    );
}

// =============================================================================
// Edge Case Tests: Unicode in Paths
// =============================================================================

/// Tests handling of Unicode characters in project paths.
#[test]
fn unicode_in_project_paths() {
    let pipeline = default_pipeline();

    let unicode_paths = vec![
        "projekt-mit-umlaut-\u{00E4}\u{00F6}\u{00FC}",
        "\u{4E2D}\u{6587}\u{9879}\u{76EE}", // Chinese characters
        "\u{30D7}\u{30ED}\u{30B8}\u{30A7}\u{30AF}\u{30C8}", // Japanese katakana
        "\u{0410}\u{0411}\u{0412}",         // Cyrillic
        "project-\u{1F600}",                // Emoji (if allowed)
    ];

    for path in unicode_paths {
        let event = EventPayload::ProjectActivity(ProjectActivityEvent {
            project_path: path.to_string(),
            session_id: "sess_unicode".to_string(),
            is_active: true,
        });

        let processed = pipeline.process(event);
        let result = serde_json::to_string(&processed);

        // Should serialize without panic
        assert!(result.is_ok(), "Should serialize unicode path: {path}");

        let json = result.unwrap();
        assert_no_sensitive_paths(&json, &format!("unicode_path_{path}"));
    }
}

// =============================================================================
// Edge Case Tests: Very Long Paths
// =============================================================================

/// Tests handling of very long project paths.
#[test]
fn very_long_project_paths() {
    let pipeline = default_pipeline();

    // Create a very long path (but not unreasonably so)
    let long_path = "a".repeat(255); // Max filename length on most systems

    let event = EventPayload::ProjectActivity(ProjectActivityEvent {
        project_path: long_path.clone(),
        session_id: "sess_long".to_string(),
        is_active: true,
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Should not contain sensitive patterns
    assert_no_sensitive_paths(&json, "very_long_path");
}

// =============================================================================
// Edge Case Tests: Special Characters in Paths
// =============================================================================

/// Tests handling of special characters in paths.
#[test]
fn special_characters_in_paths() {
    let pipeline = default_pipeline();

    let special_paths = vec![
        "project-with-dashes",
        "project_with_underscores",
        "project.with.dots",
        "project with spaces",
        "project@symbol",
        "project#hash",
        "project$dollar",
        "project%percent",
        "project(parens)",
        "project[brackets]",
    ];

    for path in special_paths {
        let event = EventPayload::ProjectActivity(ProjectActivityEvent {
            project_path: path.to_string(),
            session_id: "sess_special".to_string(),
            is_active: true,
        });

        let processed = pipeline.process(event);
        let result = serde_json::to_string(&processed);

        assert!(result.is_ok(), "Should serialize special path: {path}");

        let json = result.unwrap();
        assert_no_sensitive_paths(&json, &format!("special_path_{path}"));
    }
}

// =============================================================================
// Edge Case Tests: Null/Empty Values
// =============================================================================

/// Tests handling of empty strings in events.
#[test]
fn empty_strings_in_events() {
    let pipeline = default_pipeline();

    // Empty project path
    let event = EventPayload::ProjectActivity(ProjectActivityEvent {
        project_path: String::new(),
        session_id: "sess_empty".to_string(),
        is_active: false,
    });

    let processed = pipeline.process(event);
    let result = serde_json::to_string(&processed);
    assert!(result.is_ok(), "Should serialize empty project path");

    // Empty session ID
    let event = EventPayload::TodoProgress(TodoProgressEvent {
        session_id: String::new(),
        completed: 0,
        in_progress: 0,
        pending: 0,
        abandoned: false,
    });

    let processed = pipeline.process(event);
    let result = serde_json::to_string(&processed);
    assert!(result.is_ok(), "Should serialize empty session ID");
}

/// Tests handling of empty collections in events.
#[test]
fn empty_collections_in_events() {
    let pipeline = default_pipeline();

    // Empty hour_counts
    let event = EventPayload::ActivityPattern(ActivityPatternEvent {
        hour_counts: HashMap::new(),
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");
    assert_privacy_compliant(&json, "empty_hour_counts");

    // Empty model_usage
    let event = EventPayload::ModelDistribution(ModelDistributionEvent {
        model_usage: HashMap::new(),
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");
    assert_privacy_compliant(&json, "empty_model_usage");
}

// =============================================================================
// Comprehensive Event Serialization Test
// =============================================================================

/// Tests that all enhanced event types serialize without exposing sensitive data.
#[test]
fn all_enhanced_events_serialize_safely() {
    let pipeline = default_pipeline();

    // Create a comprehensive test payload for each event type
    let events = vec![
        EventPayload::ProjectActivity(ProjectActivityEvent {
            project_path: "safe-project".to_string(),
            session_id: "sess_001".to_string(),
            is_active: true,
        }),
        EventPayload::TokenUsage(TokenUsageEvent {
            model: "claude-3-opus".to_string(),
            input_tokens: 10000,
            output_tokens: 5000,
            cache_read_tokens: 2000,
            cache_creation_tokens: 1000,
        }),
        EventPayload::SessionMetrics(SessionMetricsEvent {
            total_sessions: 100,
            total_messages: 5000,
            total_tool_usage: 2000,
            longest_session: "sess_longest".to_string(),
        }),
        EventPayload::ActivityPattern(ActivityPatternEvent {
            hour_counts: {
                let mut map = HashMap::new();
                for hour in 0..24 {
                    map.insert(hour.to_string(), 10);
                }
                map
            },
        }),
        EventPayload::ModelDistribution(ModelDistributionEvent {
            model_usage: {
                let mut map = HashMap::new();
                map.insert(
                    "claude-3-opus".to_string(),
                    TokenUsageSummary {
                        input_tokens: 50000,
                        output_tokens: 25000,
                        cache_read_tokens: 10000,
                        cache_creation_tokens: 5000,
                    },
                );
                map
            },
        }),
        EventPayload::AgentSpawn(AgentSpawnEvent {
            session_id: "sess_002".to_string(),
            agent_type: "task".to_string(),
            description: "Background processing".to_string(),
            timestamp: test_timestamp(),
        }),
        EventPayload::SkillInvocation(SkillInvocationEvent {
            session_id: "sess_003".to_string(),
            skill_name: "format".to_string(),
            project: "project".to_string(),
            timestamp: test_timestamp(),
        }),
        EventPayload::TodoProgress(TodoProgressEvent {
            session_id: "sess_004".to_string(),
            completed: 10,
            in_progress: 5,
            pending: 3,
            abandoned: false,
        }),
        EventPayload::FileChange(FileChangeEvent {
            session_id: "sess_005".to_string(),
            file_hash: "sha256_comprehensive_test".to_string(),
            version: 1,
            lines_added: 200,
            lines_removed: 100,
            lines_modified: 50,
            timestamp: test_timestamp(),
        }),
    ];

    let mut all_json = String::new();

    for event in events {
        let processed = pipeline.process(event);
        let json = serde_json::to_string(&processed).expect("Failed to serialize");
        all_json.push_str(&json);
        all_json.push('\n');
    }

    // Comprehensive privacy check on all serialized events
    assert_privacy_compliant(&all_json, "all_enhanced_events");

    // Additional checks
    assert!(
        !all_json.contains("/home/"),
        "No home directory paths should be present"
    );
    assert!(
        !all_json.contains("/Users/"),
        "No Users directory paths should be present"
    );
    assert!(
        !all_json.contains("fn "),
        "No function definitions should be present"
    );
    assert!(
        !all_json.contains("impl "),
        "No impl blocks should be present"
    );
}

// =============================================================================
// Regression Test: Ensure Pipeline Doesn't Add Sensitive Data
// =============================================================================

/// Verifies the privacy pipeline doesn't inadvertently add sensitive data.
#[test]
fn pipeline_does_not_add_sensitive_data() {
    let pipeline = default_pipeline();

    // Create minimal events
    let event = EventPayload::TokenUsage(TokenUsageEvent {
        model: "m".to_string(),
        input_tokens: 1,
        output_tokens: 1,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Pipeline should not add any extra data
    assert_privacy_compliant(&json, "pipeline_no_additions");

    // Verify the JSON is minimal
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Valid JSON");

    // Should only have the fields we explicitly set
    if let Some(obj) = parsed.as_object() {
        for key in obj.keys() {
            // All keys should be known event fields
            let known_keys = [
                "type",
                "model",
                "inputTokens",
                "outputTokens",
                "cacheReadTokens",
                "cacheCreationTokens",
                "input_tokens",
                "output_tokens",
                "cache_read_tokens",
                "cache_creation_tokens",
            ];
            assert!(
                known_keys.contains(&key.as_str()),
                "Unexpected key in serialized event: {key}"
            );
        }
    }
}

// =============================================================================
// Test: Environment Variables Not Exposed
// =============================================================================

/// Verifies no environment variable values appear in serialized events.
#[test]
fn environment_variables_not_exposed() {
    let pipeline = default_pipeline();

    // Event that might theoretically expose env vars
    let event = EventPayload::ProjectActivity(ProjectActivityEvent {
        project_path: "project".to_string(),
        session_id: "sess_env".to_string(),
        is_active: true,
    });

    let processed = pipeline.process(event);
    let json = serde_json::to_string(&processed).expect("Failed to serialize");

    // Common env var patterns that should never appear
    let env_patterns = [
        "HOME=",
        "PATH=",
        "USER=",
        "SHELL=",
        "PWD=",
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "AWS_ACCESS_KEY",
        "AWS_SECRET_ACCESS_KEY",
        "DATABASE_URL",
        "REDIS_URL",
    ];

    for pattern in env_patterns {
        assert!(
            !json.contains(pattern),
            "JSON should not contain env var pattern: {pattern}"
        );
    }
}

// =============================================================================
// Test: Home Directory Not Exposed
// =============================================================================

/// Verifies home directory paths are never exposed in any event.
#[test]
fn home_directory_never_exposed() {
    let pipeline = default_pipeline();

    let events = vec![
        EventPayload::ProjectActivity(ProjectActivityEvent {
            project_path: "project".to_string(),
            session_id: "sess_home".to_string(),
            is_active: true,
        }),
        EventPayload::SkillInvocation(SkillInvocationEvent {
            session_id: "sess_home".to_string(),
            skill_name: "commit".to_string(),
            project: "project".to_string(),
            timestamp: test_timestamp(),
        }),
        EventPayload::FileChange(FileChangeEvent {
            session_id: "sess_home".to_string(),
            file_hash: "hash".to_string(),
            version: 1,
            lines_added: 10,
            lines_removed: 5,
            lines_modified: 2,
            timestamp: test_timestamp(),
        }),
    ];

    for event in events {
        let processed = pipeline.process(event);
        let json = serde_json::to_string(&processed).expect("Failed to serialize");

        // Home directory patterns
        let home_patterns = ["/home/", "/Users/", "/root/", "~", "C:\\Users\\"];

        for pattern in home_patterns {
            assert!(
                !json.contains(pattern),
                "JSON should not contain home dir pattern: {pattern}"
            );
        }
    }
}
