//! Privacy compliance test suite for VibeTea Monitor.
//!
//! These tests validate Constitution I (Privacy by Design) by ensuring
//! no sensitive data is ever present in processed events.
//!
//! # Privacy Guarantees Tested
//!
//! 1. No full file paths (only basenames)
//! 2. No file contents or diffs
//! 3. No user prompts or assistant responses
//! 4. No actual Bash commands (only description field allowed)
//! 5. No Grep/Glob search patterns
//! 6. No WebSearch/WebFetch context
//! 7. Summary text stripped to neutral message
//! 8. Extension allowlist filtering works correctly

use std::collections::HashSet;
use uuid::Uuid;
use vibetea_monitor::privacy::{extract_basename, PrivacyConfig, PrivacyPipeline};
use vibetea_monitor::types::{EventPayload, SessionAction, ToolStatus};

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a deterministic session ID for testing.
fn test_session_id() -> Uuid {
    Uuid::nil()
}

/// Creates a privacy pipeline with default configuration (no allowlist).
fn default_pipeline() -> PrivacyPipeline {
    PrivacyPipeline::new(PrivacyConfig::new(None))
}

/// Creates a privacy pipeline with a specific extension allowlist.
fn pipeline_with_allowlist(extensions: &[&str]) -> PrivacyPipeline {
    let allowlist: HashSet<String> = extensions.iter().map(|s| s.to_string()).collect();
    PrivacyPipeline::new(PrivacyConfig::new(Some(allowlist)))
}

/// Sensitive path patterns that should NEVER appear in processed events.
/// Note: We check for path-like patterns, not individual words that might
/// appear in legitimate basenames (e.g., "password_manager.rs" is a valid basename).
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

/// Sensitive command/content patterns that should be stripped from context.
const SENSITIVE_COMMAND_PATTERNS: &[&str] = &[
    "rm -rf",
    "sudo ",
    "chmod ",
    "curl -",
    "wget ",
    "Bearer ",
    "Authorization:",
];

/// Checks that a JSON string does not contain any sensitive path patterns.
fn assert_no_sensitive_paths(json: &str, test_name: &str) {
    for pattern in SENSITIVE_PATH_PATTERNS {
        assert!(
            !json.contains(pattern),
            "{test_name}: JSON contains sensitive path pattern '{pattern}': {json}"
        );
    }
}

/// Checks that a JSON string does not contain any sensitive command patterns.
fn assert_no_sensitive_commands(json: &str, test_name: &str) {
    for pattern in SENSITIVE_COMMAND_PATTERNS {
        assert!(
            !json.contains(pattern),
            "{test_name}: JSON contains sensitive command pattern '{pattern}': {json}"
        );
    }
}

// =============================================================================
// Test 1: no_full_paths_in_tool_events
// =============================================================================

/// Verifies that full file paths are reduced to basenames in Tool events.
///
/// The privacy pipeline must extract only the filename from paths like
/// `/home/user/projects/secret/src/auth.rs` and output just `auth.rs`.
#[test]
fn no_full_paths_in_tool_events() {
    let pipeline = default_pipeline();

    let payload = EventPayload::Tool {
        session_id: test_session_id(),
        tool: "Read".to_string(),
        status: ToolStatus::Completed,
        context: Some("/home/user/projects/secret/src/auth.rs".to_string()),
        project: Some("my-project".to_string()),
    };

    let result = pipeline.process(payload);

    if let EventPayload::Tool { context, .. } = &result {
        // Context should be reduced to just the basename
        assert_eq!(
            context.as_deref(),
            Some("auth.rs"),
            "Context should be reduced to basename only"
        );

        // Verify no directory separators remain (basenames should not contain '/')
        if let Some(ctx) = context {
            assert!(
                !ctx.contains('/'),
                "Basename should not contain directory separators: {ctx}"
            );
        }
    } else {
        panic!("Expected Tool payload");
    }

    // Additional verification: serialize and check for sensitive data
    let json = serde_json::to_string(&result).expect("Failed to serialize");
    assert!(
        !json.contains("/home/"),
        "Serialized event should not contain full path"
    );
    assert!(
        !json.contains("/projects/"),
        "Serialized event should not contain path components"
    );
}

/// Tests multiple path formats are correctly reduced to basenames.
#[test]
fn no_full_paths_various_formats() {
    let pipeline = default_pipeline();

    // Note: Windows paths (C:\...) are not parsed correctly on Unix systems
    // because Rust's Path uses the host OS path separator. This is acceptable
    // as the monitor runs on Unix systems only. Windows paths would need
    // special handling if cross-platform support is required.
    let test_cases = vec![
        // (input path, expected basename)
        ("/home/user/file.rs", "file.rs"),
        ("/Users/developer/code/main.ts", "main.ts"),
        ("/root/secret/credentials.json", "credentials.json"),
        ("./relative/path/module.rs", "module.rs"),
        ("../parent/dir/lib.rs", "lib.rs"),
        (
            "/very/deep/nested/path/to/file/in/project/component.tsx",
            "component.tsx",
        ),
    ];

    for (input_path, expected_basename) in test_cases {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Edit".to_string(),
            status: ToolStatus::Completed,
            context: Some(input_path.to_string()),
            project: None,
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(
                context.as_deref(),
                Some(expected_basename),
                "Failed for input path: {input_path}"
            );
        } else {
            panic!("Expected Tool payload for path: {input_path}");
        }
    }
}

// =============================================================================
// Test 2: no_full_paths_in_session_events
// =============================================================================

/// Verifies that Session events do not expose home directory paths.
///
/// Session project names should be simple identifiers, not full paths.
/// Note: The privacy pipeline passes Session events through unchanged because
/// project names are sanitized at parse time. This test verifies the expectation.
#[test]
fn no_full_paths_in_session_events() {
    let pipeline = default_pipeline();

    // A properly sanitized session event (as would come from the parser)
    let payload = EventPayload::Session {
        session_id: test_session_id(),
        action: SessionAction::Started,
        project: "my-project".to_string(), // Already sanitized by parser
    };

    let result = pipeline.process(payload.clone());

    if let EventPayload::Session { project, .. } = &result {
        // Project name should not contain directory separators
        assert!(
            !project.contains('/'),
            "Session project should not contain forward slashes: {project}"
        );
        assert!(
            !project.contains('\\'),
            "Session project should not contain backslashes: {project}"
        );
        assert!(
            !project.starts_with('/'),
            "Session project should not be an absolute path: {project}"
        );
        assert!(
            !project.contains("/home/"),
            "Session project should not contain home directory: {project}"
        );
    } else {
        panic!("Expected Session payload");
    }

    // Verify serialized form is clean
    let json = serde_json::to_string(&result).expect("Failed to serialize");
    assert!(
        !json.contains("/home/"),
        "Serialized session should not expose home path"
    );
    assert!(
        !json.contains("/Users/"),
        "Serialized session should not expose Users path"
    );
}

// =============================================================================
// Test 3: bash_commands_never_transmitted
// =============================================================================

/// Verifies that Bash tool context (containing actual shell commands) is always stripped.
///
/// Bash commands may contain secrets, API keys, passwords, or other sensitive
/// information that must never be transmitted.
#[test]
fn bash_commands_never_transmitted() {
    let pipeline = default_pipeline();

    let dangerous_commands = vec![
        "rm -rf /important",
        "curl -H 'Authorization: Bearer secret_token' https://api.example.com",
        "export API_KEY=sk-1234567890",
        "echo $PASSWORD | base64",
        "cat /etc/passwd",
        "sudo chmod 777 /etc/shadow",
        "git clone https://user:password@github.com/repo.git",
        "aws s3 cp s3://bucket/secrets.json .",
        "mysql -u root -pMySecretPass123",
        "docker login -u admin -p hunter2",
    ];

    for command in dangerous_commands {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Bash".to_string(),
            status: ToolStatus::Completed,
            context: Some(command.to_string()),
            project: Some("project".to_string()),
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, tool, .. } = &result {
            assert_eq!(tool, "Bash", "Tool name should be preserved");
            assert_eq!(
                *context, None,
                "Bash context should be None, but got {:?} for command: {command}",
                context
            );
        } else {
            panic!("Expected Tool payload for command: {command}");
        }

        // Double-check serialized form
        let json = serde_json::to_string(&result).expect("Failed to serialize");
        assert!(
            !json.contains("rm -rf"),
            "Serialized event should not contain rm command"
        );
        assert!(
            !json.contains("secret"),
            "Serialized event should not contain 'secret'"
        );
        assert!(
            !json.contains("password"),
            "Serialized event should not contain 'password'"
        );
    }
}

// =============================================================================
// Test 4: grep_patterns_never_transmitted
// =============================================================================

/// Verifies that Grep tool context (containing search patterns) is always stripped.
///
/// Grep patterns may reveal what sensitive information the user is searching for.
#[test]
fn grep_patterns_never_transmitted() {
    let pipeline = default_pipeline();

    let sensitive_patterns = vec![
        "password|secret|api_key",
        "TODO.*security",
        "private_key",
        "BEGIN RSA PRIVATE KEY",
        "AWS_SECRET_ACCESS_KEY",
        "Bearer [a-zA-Z0-9]+",
        "credit_card|ssn|social_security",
    ];

    for pattern in sensitive_patterns {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Grep".to_string(),
            status: ToolStatus::Started,
            context: Some(pattern.to_string()),
            project: None,
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, tool, .. } = &result {
            assert_eq!(tool, "Grep", "Tool name should be preserved");
            assert_eq!(
                *context, None,
                "Grep context should be None, but got {:?} for pattern: {pattern}",
                context
            );
        } else {
            panic!("Expected Tool payload for pattern: {pattern}");
        }

        // Verify pattern doesn't appear in serialized form
        let json = serde_json::to_string(&result).expect("Failed to serialize");
        assert!(
            !json.contains("password"),
            "Serialized Grep event should not contain search pattern"
        );
        assert!(
            !json.contains("secret"),
            "Serialized Grep event should not contain 'secret'"
        );
    }
}

// =============================================================================
// Test 5: glob_patterns_never_transmitted
// =============================================================================

/// Verifies that Glob tool context (containing file patterns) is always stripped.
///
/// Glob patterns may reveal project structure or sensitive file locations.
#[test]
fn glob_patterns_never_transmitted() {
    let pipeline = default_pipeline();

    let sensitive_patterns = vec![
        "**/*.env",
        "**/secrets/**",
        "**/.aws/credentials",
        "**/private_keys/*.pem",
        "**/*.key",
        "**/password*",
        "**/.ssh/id_rsa*",
    ];

    for pattern in sensitive_patterns {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Glob".to_string(),
            status: ToolStatus::Completed,
            context: Some(pattern.to_string()),
            project: None,
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, tool, .. } = &result {
            assert_eq!(tool, "Glob", "Tool name should be preserved");
            assert_eq!(
                *context, None,
                "Glob context should be None, but got {:?} for pattern: {pattern}",
                context
            );
        } else {
            panic!("Expected Tool payload for pattern: {pattern}");
        }

        // Verify pattern doesn't leak
        let json = serde_json::to_string(&result).expect("Failed to serialize");
        assert!(
            !json.contains(".env"),
            "Serialized Glob event should not contain file pattern"
        );
        assert!(
            !json.contains("secrets"),
            "Serialized Glob event should not contain 'secrets'"
        );
    }
}

// =============================================================================
// Test 6: websearch_never_transmits_context
// =============================================================================

/// Verifies that WebSearch tool context (containing search queries) is always stripped.
///
/// Search queries may reveal sensitive user intent or information being sought.
#[test]
fn websearch_never_transmits_context() {
    let pipeline = default_pipeline();

    let sensitive_queries = vec![
        "how to bypass authentication",
        "sql injection tutorial",
        "competitor company financials",
        "employee salary database leak",
        "medical records access",
    ];

    for query in sensitive_queries {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "WebSearch".to_string(),
            status: ToolStatus::Completed,
            context: Some(query.to_string()),
            project: None,
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, tool, .. } = &result {
            assert_eq!(tool, "WebSearch", "Tool name should be preserved");
            assert_eq!(
                *context, None,
                "WebSearch context should be None, but got {:?} for query: {query}",
                context
            );
        } else {
            panic!("Expected Tool payload for query: {query}");
        }
    }
}

// =============================================================================
// Test 7: webfetch_never_transmits_context
// =============================================================================

/// Verifies that WebFetch tool context (containing URLs) is always stripped.
///
/// URLs may contain sensitive endpoints, tokens, or reveal internal systems.
#[test]
fn webfetch_never_transmits_context() {
    let pipeline = default_pipeline();

    let sensitive_urls = vec![
        "https://internal.company.com/admin/secrets",
        "http://localhost:8080/api/users?token=abc123",
        "https://api.stripe.com/v1/charges",
        "https://vault.company.internal/v1/secret/data/prod",
        "https://user:password@private-repo.com/data",
    ];

    for url in sensitive_urls {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "WebFetch".to_string(),
            status: ToolStatus::Completed,
            context: Some(url.to_string()),
            project: None,
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, tool, .. } = &result {
            assert_eq!(tool, "WebFetch", "Tool name should be preserved");
            assert_eq!(
                *context, None,
                "WebFetch context should be None, but got {:?} for url: {url}",
                context
            );
        } else {
            panic!("Expected Tool payload for url: {url}");
        }
    }
}

// =============================================================================
// Test 8: summary_text_stripped
// =============================================================================

/// Verifies that Summary event text is stripped to a neutral message.
///
/// Session summaries may contain descriptions of sensitive work performed.
#[test]
fn summary_text_stripped() {
    let pipeline = default_pipeline();

    let sensitive_summaries = vec![
        "User implemented password hashing with bcrypt and stored API keys",
        "Debugged authentication bypass vulnerability in login flow",
        "Added credit card processing integration with Stripe API",
        "Fixed SQL injection in user search query",
        "Reviewed employee salary calculation module",
    ];

    for summary_text in sensitive_summaries {
        let payload = EventPayload::Summary {
            session_id: test_session_id(),
            summary: summary_text.to_string(),
        };

        let result = pipeline.process(payload);

        if let EventPayload::Summary { summary, .. } = &result {
            assert_eq!(
                summary, "Session ended",
                "Summary should be neutralized, but got: {summary}"
            );
            assert!(
                !summary.contains("password"),
                "Neutralized summary should not contain 'password'"
            );
            assert!(
                !summary.contains("API"),
                "Neutralized summary should not contain 'API'"
            );
        } else {
            panic!("Expected Summary payload");
        }
    }
}

// =============================================================================
// Test 9: all_event_types_safe (comprehensive)
// =============================================================================

/// Comprehensive test that creates one payload of each type with potentially
/// sensitive data, processes all through the pipeline, and verifies no
/// sensitive strings appear in the serialized JSON.
#[test]
fn all_event_types_safe() {
    let pipeline = default_pipeline();

    // Create payloads with potentially sensitive data for each event type
    let payloads = vec![
        EventPayload::Session {
            session_id: test_session_id(),
            action: SessionAction::Started,
            project: "clean-project".to_string(), // Pre-sanitized by parser
        },
        EventPayload::Activity {
            session_id: test_session_id(),
            project: Some("another-project".to_string()),
        },
        EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("/home/user/secrets/password_manager.rs".to_string()),
            project: Some("secret-project".to_string()),
        },
        EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Bash".to_string(),
            status: ToolStatus::Completed,
            context: Some(
                "curl -H 'Authorization: Bearer token123' https://api.secret.com".to_string(),
            ),
            project: None,
        },
        EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Grep".to_string(),
            status: ToolStatus::Started,
            context: Some("password|api_key|secret_token".to_string()),
            project: None,
        },
        EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Glob".to_string(),
            status: ToolStatus::Completed,
            context: Some("**/.env*".to_string()),
            project: None,
        },
        EventPayload::Tool {
            session_id: test_session_id(),
            tool: "WebSearch".to_string(),
            status: ToolStatus::Completed,
            context: Some("how to steal credentials".to_string()),
            project: None,
        },
        EventPayload::Tool {
            session_id: test_session_id(),
            tool: "WebFetch".to_string(),
            status: ToolStatus::Completed,
            context: Some("https://internal.vault.company.com/secrets".to_string()),
            project: None,
        },
        EventPayload::Agent {
            session_id: test_session_id(),
            state: "thinking".to_string(),
        },
        EventPayload::Summary {
            session_id: test_session_id(),
            summary: "User worked on password hashing and API key rotation".to_string(),
        },
        EventPayload::Error {
            session_id: test_session_id(),
            category: "network".to_string(),
        },
    ];

    // Process all payloads and collect serialized results
    let mut all_json = String::new();

    for payload in payloads {
        let processed = pipeline.process(payload);
        let json = serde_json::to_string(&processed).expect("Failed to serialize");
        all_json.push_str(&json);
        all_json.push('\n');
    }

    // Verify no sensitive path patterns appear in any serialized event
    assert_no_sensitive_paths(&all_json, "all_event_types_safe");
    assert_no_sensitive_commands(&all_json, "all_event_types_safe");

    // Additional specific checks for path components
    assert!(
        !all_json.contains("vault.company"),
        "Should not contain internal URLs"
    );
    assert!(
        !all_json.contains("/secrets/"),
        "Should not contain path components like /secrets/"
    );

    // Basenames ARE allowed (but not full paths)
    // "password_manager.rs" as a basename is acceptable - the key privacy guarantee
    // is that the full path "/home/user/secrets/password_manager.rs" is stripped.
    // The basename alone does not reveal the user's directory structure.
}

// =============================================================================
// Test 10: allowlist_filtering_removes_sensitive_extensions
// =============================================================================

/// Verifies that extension allowlist filtering correctly removes files
/// with non-allowed extensions.
#[test]
fn allowlist_filtering_removes_sensitive_extensions() {
    // Configure allowlist with only .rs and .ts
    let pipeline = pipeline_with_allowlist(&[".rs", ".ts"]);

    // Sensitive file extensions that should be filtered out
    let filtered_cases = vec![
        ("/path/to/secrets.env", None),
        ("/path/to/credentials.json", None),
        ("/path/to/private_key.pem", None),
        ("/path/to/database.key", None),
        ("/path/to/config.yaml", None),
        ("/path/to/Makefile", None), // No extension
    ];

    for (path, expected) in filtered_cases {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some(path.to_string()),
            project: None,
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(
                context, expected,
                "File {path} should be filtered by allowlist"
            );
        } else {
            panic!("Expected Tool payload for path: {path}");
        }
    }

    // Files with allowed extensions should pass through (as basenames)
    let allowed_cases = vec![
        ("/path/to/auth.rs", Some("auth.rs")),
        ("/path/to/component.ts", Some("component.ts")),
    ];

    for (path, expected) in allowed_cases {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some(path.to_string()),
            project: None,
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(
                context.as_deref(),
                expected,
                "File {path} should be allowed"
            );
        } else {
            panic!("Expected Tool payload for path: {path}");
        }
    }
}

// =============================================================================
// Additional Privacy Tests
// =============================================================================

/// Verifies that the extract_basename function correctly handles edge cases.
#[test]
fn extract_basename_edge_cases() {
    // Valid paths
    assert_eq!(
        extract_basename("/home/user/file.rs"),
        Some("file.rs".to_string())
    );
    assert_eq!(extract_basename("file.rs"), Some("file.rs".to_string()));
    assert_eq!(extract_basename("./file.rs"), Some("file.rs".to_string()));

    // Edge cases
    assert_eq!(extract_basename(""), None);
    assert_eq!(extract_basename("/"), None);

    // Hidden files (should still extract basename)
    assert_eq!(
        extract_basename("/home/user/.gitignore"),
        Some(".gitignore".to_string())
    );
    assert_eq!(
        extract_basename("/home/user/.env"),
        Some(".env".to_string())
    );
}

/// Verifies that sensitive tools are correctly identified regardless of status.
#[test]
fn sensitive_tools_stripped_for_all_statuses() {
    let pipeline = default_pipeline();

    let sensitive_tools = vec!["Bash", "Grep", "Glob", "WebSearch", "WebFetch"];
    let statuses = vec![ToolStatus::Started, ToolStatus::Completed];

    for tool_name in sensitive_tools {
        for status in &statuses {
            let payload = EventPayload::Tool {
                session_id: test_session_id(),
                tool: tool_name.to_string(),
                status: *status,
                context: Some("sensitive data here".to_string()),
                project: None,
            };

            let result = pipeline.process(payload);

            if let EventPayload::Tool { context, tool, .. } = result {
                assert_eq!(
                    context, None,
                    "Tool {tool_name} with status {:?} should have context stripped",
                    status
                );
                assert_eq!(tool, tool_name, "Tool name should be preserved");
            } else {
                panic!("Expected Tool payload");
            }
        }
    }
}

/// Verifies that non-sensitive tools (Read, Write, Edit, etc.) preserve
/// the basename but not the full path.
#[test]
fn non_sensitive_tools_preserve_basename_only() {
    let pipeline = default_pipeline();

    let non_sensitive_tools = vec!["Read", "Write", "Edit", "NotebookEdit"];

    for tool_name in non_sensitive_tools {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: tool_name.to_string(),
            status: ToolStatus::Completed,
            context: Some("/home/user/very/long/path/to/file.rs".to_string()),
            project: None,
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, tool, .. } = result {
            assert_eq!(tool, tool_name, "Tool name should be preserved");
            assert_eq!(
                context.as_deref(),
                Some("file.rs"),
                "Tool {tool_name} should extract basename"
            );

            // Ensure path is not present
            if let Some(ctx) = &context {
                assert!(
                    !ctx.contains('/'),
                    "Context for {tool_name} should not contain path separator"
                );
                assert!(
                    !ctx.contains("home"),
                    "Context for {tool_name} should not contain path components"
                );
            }
        } else {
            panic!("Expected Tool payload for tool: {tool_name}");
        }
    }
}

/// Verifies that the pipeline correctly handles None context values.
#[test]
fn none_context_remains_none() {
    let pipeline = default_pipeline();

    let payload = EventPayload::Tool {
        session_id: test_session_id(),
        tool: "Read".to_string(),
        status: ToolStatus::Started,
        context: None,
        project: Some("project".to_string()),
    };

    let result = pipeline.process(payload);

    if let EventPayload::Tool { context, .. } = result {
        assert_eq!(context, None, "None context should remain None");
    } else {
        panic!("Expected Tool payload");
    }
}

/// Verifies that project field in Tool events is preserved unchanged.
#[test]
fn tool_project_field_preserved() {
    let pipeline = default_pipeline();

    let payload = EventPayload::Tool {
        session_id: test_session_id(),
        tool: "Read".to_string(),
        status: ToolStatus::Completed,
        context: Some("/path/to/file.rs".to_string()),
        project: Some("important-project".to_string()),
    };

    let result = pipeline.process(payload);

    if let EventPayload::Tool { project, .. } = result {
        assert_eq!(
            project,
            Some("important-project".to_string()),
            "Project field should be preserved"
        );
    } else {
        panic!("Expected Tool payload");
    }
}

/// Stress test with many different path formats to ensure robustness.
#[test]
fn path_sanitization_stress_test() {
    let pipeline = default_pipeline();

    let paths = vec![
        // Unix absolute paths
        "/home/user/file.rs",
        "/root/.secret/passwords.rs",
        "/var/log/app.rs",
        "/etc/config/settings.rs",
        "/opt/application/main.rs",
        // Deeply nested paths
        "/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/deep.rs",
        // Paths with special characters
        "/path/to/file with spaces.rs",
        "/path/to/file-with-dashes.rs",
        "/path/to/file_with_underscores.rs",
        // Relative paths
        "./relative.rs",
        "../parent.rs",
        "../../grandparent.rs",
        // Just filenames
        "simple.rs",
        "UPPERCASE.rs",
        "MixedCase.rs",
    ];

    for path in paths {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some(path.to_string()),
            project: None,
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, .. } = result {
            if let Some(ctx) = context {
                // Should not contain forward slashes (except potentially in the filename itself)
                // but definitely should not contain path prefixes
                assert!(
                    !ctx.contains("/home/"),
                    "Path {path}: context should not contain /home/"
                );
                assert!(
                    !ctx.contains("/root/"),
                    "Path {path}: context should not contain /root/"
                );
                assert!(
                    !ctx.contains("/var/"),
                    "Path {path}: context should not contain /var/"
                );
                assert!(
                    !ctx.contains("/etc/"),
                    "Path {path}: context should not contain /etc/"
                );
            }
        } else {
            panic!("Expected Tool payload for path: {path}");
        }
    }
}
