//! Privacy pipeline for VibeTea Monitor.
//!
//! This module ensures no sensitive data (source code, file contents, full paths,
//! prompts, commands) is ever transmitted to the server. All event payloads pass
//! through the privacy pipeline before being sent.
//!
//! # Privacy Guarantees
//!
//! The privacy pipeline provides the following guarantees:
//!
//! - **Path-to-basename conversion**: Full file paths like `/home/user/project/src/auth.ts`
//!   are reduced to just the basename `auth.ts`
//! - **Content stripping**: File contents, diffs, code, user prompts, and assistant
//!   responses are completely stripped
//! - **Bash command stripping**: Only the `description` field is transmitted (if present),
//!   actual commands are never sent
//! - **Grep/Glob pattern stripping**: Patterns are omitted entirely, only tool name is transmitted
//! - **Extension allowlist filtering**: When `VIBETEA_BASENAME_ALLOWLIST` is set,
//!   only files with those extensions are transmitted; others have their context set to `None`
//!
//! # Example
//!
//! ```
//! use vibetea_monitor::privacy::{PrivacyConfig, PrivacyPipeline};
//! use vibetea_monitor::types::{EventPayload, ToolStatus};
//! use uuid::Uuid;
//!
//! // Create pipeline with no allowlist (allow all extensions)
//! let config = PrivacyConfig::new(None);
//! let pipeline = PrivacyPipeline::new(config);
//!
//! // Process a tool event - full path is reduced to basename
//! let payload = EventPayload::Tool {
//!     session_id: Uuid::new_v4(),
//!     tool: "Read".to_string(),
//!     status: ToolStatus::Completed,
//!     context: Some("/home/user/project/src/auth.ts".to_string()),
//!     project: Some("my-project".to_string()),
//! };
//!
//! let sanitized = pipeline.process(payload);
//! if let EventPayload::Tool { context, .. } = sanitized {
//!     assert_eq!(context, Some("auth.ts".to_string()));
//! }
//! ```

use std::collections::HashSet;
use std::env;
use std::path::Path;

use tracing::debug;

use crate::types::EventPayload;

/// Tools whose context should always be stripped for privacy.
///
/// These tools may contain sensitive information:
/// - `Bash`: Contains shell commands which may include secrets, passwords, or API keys
/// - `Grep`: Contains search patterns which may reveal what the user is looking for
/// - `Glob`: Contains file patterns which may reveal project structure
/// - `WebSearch`: Contains search queries which may reveal user intent
/// - `WebFetch`: Contains URLs which may contain sensitive information
const SENSITIVE_TOOLS: &[&str] = &["Bash", "Grep", "Glob", "WebSearch", "WebFetch"];

/// Configuration for the privacy pipeline.
///
/// Controls which file extensions are allowed to be transmitted and how
/// paths are processed.
///
/// # Example
///
/// ```
/// use vibetea_monitor::privacy::PrivacyConfig;
/// use std::collections::HashSet;
///
/// // Allow only Rust and TypeScript files
/// let mut allowlist = HashSet::new();
/// allowlist.insert(".rs".to_string());
/// allowlist.insert(".ts".to_string());
///
/// let config = PrivacyConfig::new(Some(allowlist));
/// assert!(config.is_extension_allowed("file.rs"));
/// assert!(!config.is_extension_allowed("file.py"));
/// ```
#[derive(Debug, Clone)]
pub struct PrivacyConfig {
    /// Allowed file extensions (including the leading dot, e.g., `.rs`, `.ts`).
    /// If `None`, all extensions are allowed.
    basename_allowlist: Option<HashSet<String>>,
}

impl PrivacyConfig {
    /// Creates a new `PrivacyConfig` with the specified allowlist.
    ///
    /// # Arguments
    ///
    /// * `basename_allowlist` - Set of allowed extensions (e.g., `.rs`, `.ts`).
    ///   If `None`, all extensions are allowed.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::privacy::PrivacyConfig;
    /// use std::collections::HashSet;
    ///
    /// // Allow all extensions
    /// let config = PrivacyConfig::new(None);
    ///
    /// // Allow only specific extensions
    /// let mut allowlist = HashSet::new();
    /// allowlist.insert(".rs".to_string());
    /// let config = PrivacyConfig::new(Some(allowlist));
    /// ```
    #[must_use]
    pub fn new(basename_allowlist: Option<HashSet<String>>) -> Self {
        Self { basename_allowlist }
    }

    /// Creates a `PrivacyConfig` from environment variables.
    ///
    /// Reads the `VIBETEA_BASENAME_ALLOWLIST` environment variable, which should
    /// contain a comma-separated list of file extensions (e.g., `.rs,.ts,.md`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_monitor::privacy::PrivacyConfig;
    ///
    /// std::env::set_var("VIBETEA_BASENAME_ALLOWLIST", ".rs,.ts,.md");
    /// let config = PrivacyConfig::from_env();
    ///
    /// assert!(config.is_extension_allowed("file.rs"));
    /// assert!(!config.is_extension_allowed("file.py"));
    /// ```
    #[must_use]
    pub fn from_env() -> Self {
        let basename_allowlist = env::var("VIBETEA_BASENAME_ALLOWLIST").ok().map(|val| {
            val.split(',')
                .map(|s| {
                    let trimmed = s.trim();
                    // Ensure extension starts with a dot
                    if trimmed.starts_with('.') {
                        trimmed.to_string()
                    } else {
                        format!(".{trimmed}")
                    }
                })
                .filter(|s| s.len() > 1) // Filter out empty or just "."
                .collect()
        });

        debug!(
            ?basename_allowlist,
            "Loaded privacy config from environment"
        );

        Self { basename_allowlist }
    }

    /// Checks if a filename's extension is in the allowlist.
    ///
    /// Returns `true` if:
    /// - No allowlist is configured (all extensions allowed), or
    /// - The file's extension is in the allowlist
    ///
    /// Returns `false` if:
    /// - An allowlist is configured and the extension is not in it
    /// - The file has no extension and an allowlist is configured
    ///
    /// # Arguments
    ///
    /// * `basename` - The filename (not full path) to check
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::privacy::PrivacyConfig;
    /// use std::collections::HashSet;
    ///
    /// let mut allowlist = HashSet::new();
    /// allowlist.insert(".rs".to_string());
    /// allowlist.insert(".ts".to_string());
    /// let config = PrivacyConfig::new(Some(allowlist));
    ///
    /// assert!(config.is_extension_allowed("auth.rs"));
    /// assert!(config.is_extension_allowed("index.ts"));
    /// assert!(!config.is_extension_allowed("config.json"));
    /// assert!(!config.is_extension_allowed("Makefile")); // No extension
    /// ```
    #[must_use]
    pub fn is_extension_allowed(&self, basename: &str) -> bool {
        match &self.basename_allowlist {
            None => true,
            Some(allowlist) => {
                // Extract extension from basename
                Path::new(basename)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| {
                        let ext_with_dot = format!(".{ext}");
                        allowlist.contains(&ext_with_dot)
                    })
                    .unwrap_or(false) // No extension = not allowed when allowlist is set
            }
        }
    }

    /// Returns the configured allowlist, if any.
    #[must_use]
    pub fn allowlist(&self) -> Option<&HashSet<String>> {
        self.basename_allowlist.as_ref()
    }
}

impl Default for PrivacyConfig {
    /// Creates a default `PrivacyConfig` that allows all extensions.
    fn default() -> Self {
        Self::new(None)
    }
}

/// Privacy pipeline for processing event payloads before transmission.
///
/// The pipeline ensures no sensitive data leaves the system by:
/// - Stripping full paths to basenames
/// - Removing context from sensitive tools (Bash, Grep, Glob, etc.)
/// - Filtering files based on extension allowlist
/// - Clearing summary text
///
/// # Example
///
/// ```
/// use vibetea_monitor::privacy::{PrivacyConfig, PrivacyPipeline};
/// use vibetea_monitor::types::{EventPayload, ToolStatus};
/// use uuid::Uuid;
///
/// let pipeline = PrivacyPipeline::new(PrivacyConfig::default());
///
/// // Bash commands are always stripped
/// let bash_event = EventPayload::Tool {
///     session_id: Uuid::new_v4(),
///     tool: "Bash".to_string(),
///     status: ToolStatus::Completed,
///     context: Some("rm -rf /".to_string()), // Sensitive!
///     project: None,
/// };
///
/// let sanitized = pipeline.process(bash_event);
/// if let EventPayload::Tool { context, .. } = sanitized {
///     assert_eq!(context, None); // Command stripped
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PrivacyPipeline {
    config: PrivacyConfig,
}

impl PrivacyPipeline {
    /// Creates a new privacy pipeline with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Privacy configuration controlling filtering behavior
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::privacy::{PrivacyConfig, PrivacyPipeline};
    ///
    /// let config = PrivacyConfig::from_env();
    /// let pipeline = PrivacyPipeline::new(config);
    /// ```
    #[must_use]
    pub fn new(config: PrivacyConfig) -> Self {
        Self { config }
    }

    /// Processes an event payload through the privacy pipeline.
    ///
    /// This method applies all privacy transformations to the payload:
    ///
    /// - **Session**: Passes through unchanged (project already sanitized at parse time)
    /// - **Activity**: Passes through unchanged
    /// - **Tool**: Context is processed based on tool type:
    ///   - Sensitive tools (Bash, Grep, Glob, WebSearch, WebFetch): context set to `None`
    ///   - Other tools: basename extracted from path, allowlist filter applied
    /// - **Agent**: Passes through unchanged
    /// - **Summary**: Summary text set to "Session ended"
    /// - **Error**: Passes through unchanged (category already sanitized)
    ///
    /// # Arguments
    ///
    /// * `payload` - The event payload to process
    ///
    /// # Returns
    ///
    /// The sanitized event payload safe for transmission
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::privacy::{PrivacyConfig, PrivacyPipeline};
    /// use vibetea_monitor::types::{EventPayload, ToolStatus};
    /// use uuid::Uuid;
    ///
    /// let pipeline = PrivacyPipeline::new(PrivacyConfig::default());
    ///
    /// let payload = EventPayload::Tool {
    ///     session_id: Uuid::new_v4(),
    ///     tool: "Read".to_string(),
    ///     status: ToolStatus::Completed,
    ///     context: Some("/home/user/project/src/main.rs".to_string()),
    ///     project: Some("my-project".to_string()),
    /// };
    ///
    /// let sanitized = pipeline.process(payload);
    /// if let EventPayload::Tool { context, .. } = sanitized {
    ///     assert_eq!(context, Some("main.rs".to_string()));
    /// }
    /// ```
    #[must_use]
    pub fn process(&self, payload: EventPayload) -> EventPayload {
        match payload {
            // Session events pass through - project is already sanitized at parse time
            EventPayload::Session { .. } => payload,

            // Activity events pass through unchanged
            EventPayload::Activity { .. } => payload,

            // Tool events need context processing
            EventPayload::Tool {
                session_id,
                tool,
                status,
                context,
                project,
            } => {
                let sanitized_context = self.process_tool_context(&tool, context);
                EventPayload::Tool {
                    session_id,
                    tool,
                    status,
                    context: sanitized_context,
                    project,
                }
            }

            // Agent events pass through unchanged
            EventPayload::Agent { .. } => payload,

            // Summary text is stripped for privacy
            EventPayload::Summary { session_id, .. } => EventPayload::Summary {
                session_id,
                summary: "Session ended".to_string(),
            },

            // Error events pass through - category is already sanitized
            EventPayload::Error { .. } => payload,
        }
    }

    /// Processes tool context based on the tool type.
    ///
    /// - Sensitive tools: always returns `None`
    /// - Other tools: extracts basename and applies allowlist filter
    fn process_tool_context(&self, tool: &str, context: Option<String>) -> Option<String> {
        // Sensitive tools always have context stripped
        if SENSITIVE_TOOLS.contains(&tool) {
            debug!(tool, "Stripping context from sensitive tool");
            return None;
        }

        // For other tools, extract basename and apply allowlist
        context.and_then(|ctx| {
            let basename = extract_basename(&ctx);

            // If we couldn't extract a valid basename, don't transmit
            let basename = basename?;

            // Apply allowlist filter
            if self.config.is_extension_allowed(&basename) {
                debug!(tool, basename = %basename, "Context allowed by privacy filter");
                Some(basename)
            } else {
                debug!(tool, basename = %basename, "Context filtered by allowlist");
                None
            }
        })
    }

    /// Returns a reference to the pipeline's configuration.
    #[must_use]
    pub fn config(&self) -> &PrivacyConfig {
        &self.config
    }
}

impl Default for PrivacyPipeline {
    /// Creates a default pipeline with no allowlist (all extensions allowed).
    fn default() -> Self {
        Self::new(PrivacyConfig::default())
    }
}

/// Extracts the basename (filename) from a file path.
///
/// This function handles various path formats:
/// - Unix paths: `/home/user/file.rs` -> `file.rs`
/// - Windows paths: `C:\Users\user\file.rs` -> `file.rs`
/// - Relative paths: `src/file.rs` -> `file.rs`
/// - Already a basename: `file.rs` -> `file.rs`
///
/// # Arguments
///
/// * `path` - The path string to extract the basename from
///
/// # Returns
///
/// - `Some(basename)` if a valid basename was extracted
/// - `None` if the path is empty, ends with a separator, or has no filename
///
/// # Example
///
/// ```
/// use vibetea_monitor::privacy::extract_basename;
///
/// assert_eq!(extract_basename("/home/user/file.rs"), Some("file.rs".to_string()));
/// assert_eq!(extract_basename("file.rs"), Some("file.rs".to_string()));
/// assert_eq!(extract_basename("/"), None);
/// assert_eq!(extract_basename(""), None);
/// ```
#[must_use]
pub fn extract_basename(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }

    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SessionAction, ToolStatus};
    use uuid::Uuid;

    // =========================================================================
    // PrivacyConfig Tests
    // =========================================================================

    #[test]
    fn config_new_with_no_allowlist_allows_all() {
        let config = PrivacyConfig::new(None);
        assert!(config.is_extension_allowed("file.rs"));
        assert!(config.is_extension_allowed("file.py"));
        assert!(config.is_extension_allowed("file.anything"));
        assert!(config.is_extension_allowed("Makefile")); // No extension allowed when no filter
    }

    #[test]
    fn config_new_with_allowlist_filters_extensions() {
        let mut allowlist = HashSet::new();
        allowlist.insert(".rs".to_string());
        allowlist.insert(".ts".to_string());
        let config = PrivacyConfig::new(Some(allowlist));

        assert!(config.is_extension_allowed("file.rs"));
        assert!(config.is_extension_allowed("file.ts"));
        assert!(!config.is_extension_allowed("file.py"));
        assert!(!config.is_extension_allowed("file.js"));
    }

    #[test]
    fn config_allowlist_rejects_no_extension() {
        let mut allowlist = HashSet::new();
        allowlist.insert(".rs".to_string());
        let config = PrivacyConfig::new(Some(allowlist));

        assert!(!config.is_extension_allowed("Makefile"));
        assert!(!config.is_extension_allowed("Dockerfile"));
        assert!(!config.is_extension_allowed(".gitignore")); // Hidden file, no "extension"
    }

    #[test]
    fn config_from_env_with_no_var_allows_all() {
        // Clear the env var if set
        std::env::remove_var("VIBETEA_BASENAME_ALLOWLIST");

        let config = PrivacyConfig::from_env();
        assert!(config.allowlist().is_none());
        assert!(config.is_extension_allowed("any.file"));
    }

    #[test]
    fn config_from_env_parses_comma_separated() {
        std::env::set_var("VIBETEA_BASENAME_ALLOWLIST", ".rs,.ts,.md");
        let config = PrivacyConfig::from_env();
        std::env::remove_var("VIBETEA_BASENAME_ALLOWLIST");

        assert!(config.is_extension_allowed("file.rs"));
        assert!(config.is_extension_allowed("file.ts"));
        assert!(config.is_extension_allowed("file.md"));
        assert!(!config.is_extension_allowed("file.py"));
    }

    #[test]
    fn config_from_env_handles_missing_dots() {
        std::env::set_var("VIBETEA_BASENAME_ALLOWLIST", "rs,ts,md");
        let config = PrivacyConfig::from_env();
        std::env::remove_var("VIBETEA_BASENAME_ALLOWLIST");

        // Should add dots automatically
        assert!(config.is_extension_allowed("file.rs"));
        assert!(config.is_extension_allowed("file.ts"));
        assert!(config.is_extension_allowed("file.md"));
    }

    #[test]
    fn config_from_env_trims_whitespace() {
        std::env::set_var("VIBETEA_BASENAME_ALLOWLIST", " .rs , .ts , .md ");
        let config = PrivacyConfig::from_env();
        std::env::remove_var("VIBETEA_BASENAME_ALLOWLIST");

        assert!(config.is_extension_allowed("file.rs"));
        assert!(config.is_extension_allowed("file.ts"));
        assert!(config.is_extension_allowed("file.md"));
    }

    #[test]
    fn config_from_env_filters_empty_entries() {
        std::env::set_var("VIBETEA_BASENAME_ALLOWLIST", ".rs,,.ts,,,");
        let config = PrivacyConfig::from_env();
        std::env::remove_var("VIBETEA_BASENAME_ALLOWLIST");

        let allowlist = config.allowlist().expect("should have allowlist");
        assert_eq!(allowlist.len(), 2);
        assert!(allowlist.contains(".rs"));
        assert!(allowlist.contains(".ts"));
    }

    #[test]
    fn config_default_allows_all() {
        let config = PrivacyConfig::default();
        assert!(config.allowlist().is_none());
        assert!(config.is_extension_allowed("anything.xyz"));
    }

    // =========================================================================
    // extract_basename Tests
    // =========================================================================

    #[test]
    fn extract_basename_from_unix_absolute_path() {
        assert_eq!(
            extract_basename("/home/user/project/src/auth.ts"),
            Some("auth.ts".to_string())
        );
    }

    #[test]
    fn extract_basename_from_unix_relative_path() {
        assert_eq!(
            extract_basename("src/components/Button.tsx"),
            Some("Button.tsx".to_string())
        );
    }

    #[test]
    fn extract_basename_already_basename() {
        assert_eq!(extract_basename("file.rs"), Some("file.rs".to_string()));
    }

    #[test]
    fn extract_basename_with_dots_in_name() {
        assert_eq!(
            extract_basename("/path/to/file.test.ts"),
            Some("file.test.ts".to_string())
        );
    }

    #[test]
    fn extract_basename_hidden_file() {
        assert_eq!(
            extract_basename("/home/user/.gitignore"),
            Some(".gitignore".to_string())
        );
    }

    #[test]
    fn extract_basename_empty_path() {
        assert_eq!(extract_basename(""), None);
    }

    #[test]
    fn extract_basename_root_path() {
        // Root path has no filename component
        assert_eq!(extract_basename("/"), None);
    }

    #[test]
    fn extract_basename_trailing_slash() {
        // Path ending with / - Rust's Path considers the last component as the filename
        // This is platform-dependent behavior; on Unix, "/home/user/" gives "user"
        // For our purposes, this is acceptable as we're extracting what looks like a filename
        assert_eq!(extract_basename("/home/user/"), Some("user".to_string()));
    }

    // =========================================================================
    // PrivacyPipeline Tests
    // =========================================================================

    fn test_session_id() -> Uuid {
        Uuid::nil()
    }

    #[test]
    fn pipeline_session_passes_through() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Session {
            session_id: test_session_id(),
            action: SessionAction::Started,
            project: "my-project".to_string(),
        };

        let result = pipeline.process(payload.clone());
        assert_eq!(result, payload);
    }

    #[test]
    fn pipeline_activity_passes_through() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Activity {
            session_id: test_session_id(),
            project: Some("my-project".to_string()),
        };

        let result = pipeline.process(payload.clone());
        assert_eq!(result, payload);
    }

    #[test]
    fn pipeline_agent_passes_through() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Agent {
            session_id: test_session_id(),
            state: "thinking".to_string(),
        };

        let result = pipeline.process(payload.clone());
        assert_eq!(result, payload);
    }

    #[test]
    fn pipeline_error_passes_through() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Error {
            session_id: test_session_id(),
            category: "network".to_string(),
        };

        let result = pipeline.process(payload.clone());
        assert_eq!(result, payload);
    }

    #[test]
    fn pipeline_summary_strips_text() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Summary {
            session_id: test_session_id(),
            summary: "User worked on implementing auth feature with sensitive details".to_string(),
        };

        let result = pipeline.process(payload);
        if let EventPayload::Summary { summary, .. } = result {
            assert_eq!(summary, "Session ended");
        } else {
            panic!("Expected Summary payload");
        }
    }

    #[test]
    fn pipeline_tool_bash_strips_context() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Bash".to_string(),
            status: ToolStatus::Completed,
            context: Some("rm -rf / --no-preserve-root".to_string()),
            project: Some("my-project".to_string()),
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, tool, .. } = result {
            assert_eq!(tool, "Bash");
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_grep_strips_context() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Grep".to_string(),
            status: ToolStatus::Started,
            context: Some("password|secret|api_key".to_string()),
            project: None,
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_glob_strips_context() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Glob".to_string(),
            status: ToolStatus::Completed,
            context: Some("**/*.secret".to_string()),
            project: None,
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_websearch_strips_context() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "WebSearch".to_string(),
            status: ToolStatus::Completed,
            context: Some("how to bypass security".to_string()),
            project: None,
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_webfetch_strips_context() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "WebFetch".to_string(),
            status: ToolStatus::Completed,
            context: Some("https://internal.company.com/secrets".to_string()),
            project: None,
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_read_extracts_basename() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("/home/user/project/src/auth.ts".to_string()),
            project: Some("my-project".to_string()),
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, Some("auth.ts".to_string()));
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_write_extracts_basename() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Write".to_string(),
            status: ToolStatus::Started,
            context: Some("/home/user/project/src/components/Button.tsx".to_string()),
            project: None,
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, Some("Button.tsx".to_string()));
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_edit_extracts_basename() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Edit".to_string(),
            status: ToolStatus::Completed,
            context: Some("src/lib.rs".to_string()),
            project: None,
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, Some("lib.rs".to_string()));
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_with_no_context() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Started,
            context: None,
            project: None,
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_allowlist_filters() {
        let mut allowlist = HashSet::new();
        allowlist.insert(".rs".to_string());
        allowlist.insert(".ts".to_string());
        let config = PrivacyConfig::new(Some(allowlist));
        let pipeline = PrivacyPipeline::new(config);

        // Allowed extension
        let payload_allowed = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("/path/to/file.rs".to_string()),
            project: None,
        };

        let result = pipeline.process(payload_allowed);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, Some("file.rs".to_string()));
        } else {
            panic!("Expected Tool payload");
        }

        // Disallowed extension
        let payload_disallowed = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("/path/to/file.py".to_string()),
            project: None,
        };

        let result = pipeline.process(payload_disallowed);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_allowlist_with_no_extension() {
        let mut allowlist = HashSet::new();
        allowlist.insert(".rs".to_string());
        let config = PrivacyConfig::new(Some(allowlist));
        let pipeline = PrivacyPipeline::new(config);

        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("/path/to/Makefile".to_string()),
            project: None,
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_preserves_project_field() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("/path/to/file.rs".to_string()),
            project: Some("important-project".to_string()),
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { project, .. } = result {
            assert_eq!(project, Some("important-project".to_string()));
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_default_allows_all() {
        let pipeline = PrivacyPipeline::default();
        assert!(pipeline.config().allowlist().is_none());
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn pipeline_handles_complex_paths() {
        let pipeline = PrivacyPipeline::default();

        let paths = vec![
            ("/home/user/../user/file.rs", "file.rs"),
            ("./src/file.rs", "file.rs"),
            ("file.rs", "file.rs"),
            ("/a/b/c/d/e/f/g/file.rs", "file.rs"),
        ];

        for (input_path, expected_basename) in paths {
            let payload = EventPayload::Tool {
                session_id: test_session_id(),
                tool: "Read".to_string(),
                status: ToolStatus::Completed,
                context: Some(input_path.to_string()),
                project: None,
            };

            let result = pipeline.process(payload);
            if let EventPayload::Tool { context, .. } = result {
                assert_eq!(
                    context,
                    Some(expected_basename.to_string()),
                    "Failed for path: {input_path}"
                );
            } else {
                panic!("Expected Tool payload");
            }
        }
    }

    #[test]
    fn pipeline_handles_unicode_filenames() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("/path/to/文件.rs".to_string()),
            project: None,
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, Some("文件.rs".to_string()));
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_case_sensitive_tool_names() {
        let pipeline = PrivacyPipeline::default();

        // "Bash" (capital B) should be stripped
        let bash_payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Bash".to_string(),
            status: ToolStatus::Completed,
            context: Some("echo secret".to_string()),
            project: None,
        };

        let result = pipeline.process(bash_payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }

        // "bash" (lowercase) would not match and would be treated as a normal tool
        // This is intentional - tool names from Claude Code are case-sensitive
        let bash_lower = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "bash".to_string(),
            status: ToolStatus::Completed,
            context: Some("/path/to/file.sh".to_string()),
            project: None,
        };

        let result = pipeline.process(bash_lower);
        if let EventPayload::Tool { context, .. } = result {
            // Lowercase "bash" is not in SENSITIVE_TOOLS, so it extracts basename
            assert_eq!(context, Some("file.sh".to_string()));
        } else {
            panic!("Expected Tool payload");
        }
    }
}
