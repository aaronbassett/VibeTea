//! Session filename parser utilities for Claude Code file paths.
//!
//! This module provides utilities for extracting session UUIDs and metadata
//! from various Claude Code file paths:
//!
//! - **Todo files**: `~/.claude/todos/<session-uuid>-agent-<session-uuid>.json`
//! - **File history**: `~/.claude/file-history/<session-uuid>/<hash>@vN`
//! - **Session JSONL**: `~/.claude/projects/<path-slug>/<session-uuid>.jsonl`
//!
//! # Example
//!
//! ```
//! use std::path::Path;
//! use vibetea_monitor::utils::session_filename::{
//!     parse_todo_filename,
//!     parse_file_history_path,
//!     parse_session_jsonl_path,
//! };
//!
//! // Parse a todo filename
//! let todo_path = Path::new("/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json");
//! let session_id = parse_todo_filename(todo_path);
//! assert_eq!(session_id.as_deref(), Some("6e45a55c-3124-4cc8-ad85-040a5c316009"));
//!
//! // Parse a file history path
//! let history_path = Path::new("/home/user/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/3a8f2c1b9e4d7a6f@v2");
//! let info = parse_file_history_path(history_path).unwrap();
//! assert_eq!(info.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
//! assert_eq!(info.file_hash, "3a8f2c1b9e4d7a6f");
//! assert_eq!(info.version, 2);
//!
//! // Parse a session JSONL path
//! let jsonl_path = Path::new("/home/user/.claude/projects/-home-user-myproject/6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl");
//! let info = parse_session_jsonl_path(jsonl_path).unwrap();
//! assert_eq!(info.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
//! assert_eq!(info.project_slug, "-home-user-myproject");
//! ```

use std::path::Path;

/// Information extracted from a file history path.
///
/// File history paths have the format:
/// `~/.claude/file-history/<session-uuid>/<16-char-hex-hash>@v<version>`
///
/// # Fields
///
/// * `session_id` - The session UUID extracted from the parent directory name
/// * `file_hash` - The 16-character hexadecimal file hash
/// * `version` - The version number from the `@vN` suffix
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use vibetea_monitor::utils::session_filename::parse_file_history_path;
///
/// let path = Path::new("/home/.claude/file-history/a1b2c3d4-e5f6-7890-abcd-ef1234567890/deadbeef12345678@v3");
/// let info = parse_file_history_path(path).unwrap();
///
/// assert_eq!(info.session_id, "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
/// assert_eq!(info.file_hash, "deadbeef12345678");
/// assert_eq!(info.version, 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHistoryInfo {
    /// The session UUID from the parent directory.
    pub session_id: String,
    /// The 16-character hexadecimal file hash.
    pub file_hash: String,
    /// The version number from the filename suffix.
    pub version: u32,
}

/// Information extracted from a session JSONL path.
///
/// Session JSONL paths have the format:
/// `~/.claude/projects/<project-slug>/<session-uuid>.jsonl`
///
/// # Fields
///
/// * `session_id` - The session UUID extracted from the filename
/// * `project_slug` - The slugified project path from the parent directory
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use vibetea_monitor::utils::session_filename::parse_session_jsonl_path;
///
/// let path = Path::new("/home/.claude/projects/-home-user-project/a1b2c3d4-e5f6-7890-abcd-ef1234567890.jsonl");
/// let info = parse_session_jsonl_path(path).unwrap();
///
/// assert_eq!(info.session_id, "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
/// assert_eq!(info.project_slug, "-home-user-project");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionJsonlInfo {
    /// The session UUID from the filename.
    pub session_id: String,
    /// The slugified project path from the parent directory.
    pub project_slug: String,
}

/// Parses a todo filename to extract the session UUID.
///
/// Todo files have the format:
/// `{session-uuid}-agent-{session-uuid}.json`
///
/// The session UUID is extracted by splitting on `-agent-` and taking the first part.
///
/// # Arguments
///
/// * `path` - The path to the todo file
///
/// # Returns
///
/// - `Some(session_id)` if the filename matches the expected pattern and contains a valid UUID
/// - `None` if the path has no filename, doesn't end in `.json`, doesn't contain `-agent-`,
///   or the extracted session ID is not a valid UUID
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use vibetea_monitor::utils::session_filename::parse_todo_filename;
///
/// // Valid todo filename
/// let path = Path::new("/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json");
/// assert_eq!(
///     parse_todo_filename(path),
///     Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
/// );
///
/// // Invalid: no `-agent-` separator
/// let path = Path::new("/home/user/.claude/todos/some-file.json");
/// assert_eq!(parse_todo_filename(path), None);
///
/// // Invalid: not a .json file
/// let path = Path::new("/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.txt");
/// assert_eq!(parse_todo_filename(path), None);
/// ```
#[must_use]
pub fn parse_todo_filename(path: &Path) -> Option<String> {
    // Get the filename
    let filename = path.file_name()?.to_str()?;

    // Must end with .json
    let stem = filename.strip_suffix(".json")?;

    // Must contain "-agent-" separator - split_once returns None if not found
    let (session_id, _rest) = stem.split_once("-agent-")?;

    // Validate that it's a valid UUID format
    if is_valid_uuid(session_id) {
        Some(session_id.to_string())
    } else {
        None
    }
}

/// Parses a file history path to extract session info, file hash, and version.
///
/// File history paths have the format:
/// `~/.claude/file-history/<session-uuid>/<16-char-hex-hash>@v<version>`
///
/// # Arguments
///
/// * `path` - The path to the file history entry
///
/// # Returns
///
/// - `Some(FileHistoryInfo)` if the path matches the expected pattern
/// - `None` if the path doesn't match (missing parent, invalid format, etc.)
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use vibetea_monitor::utils::session_filename::parse_file_history_path;
///
/// // Valid file history path
/// let path = Path::new("/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/3a8f2c1b9e4d7a6f@v2");
/// let info = parse_file_history_path(path).unwrap();
/// assert_eq!(info.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
/// assert_eq!(info.file_hash, "3a8f2c1b9e4d7a6f");
/// assert_eq!(info.version, 2);
///
/// // Invalid: no @v version suffix
/// let path = Path::new("/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/3a8f2c1b9e4d7a6f");
/// assert!(parse_file_history_path(path).is_none());
///
/// // Invalid: parent is not a valid UUID
/// let path = Path::new("/home/.claude/file-history/invalid-id/3a8f2c1b9e4d7a6f@v1");
/// assert!(parse_file_history_path(path).is_none());
/// ```
#[must_use]
pub fn parse_file_history_path(path: &Path) -> Option<FileHistoryInfo> {
    // Get the filename (e.g., "3a8f2c1b9e4d7a6f@v2")
    let filename = path.file_name()?.to_str()?;

    // Split on "@v" to get hash and version
    let (hash_part, version_part) = filename.split_once("@v")?;

    // Validate hash is 16 hex characters
    if !is_valid_hex_hash(hash_part) {
        return None;
    }

    // Parse version number
    let version: u32 = version_part.parse().ok()?;

    // Get the parent directory (session UUID)
    let parent = path.parent()?;
    let session_id = parent.file_name()?.to_str()?;

    // Validate session ID is a valid UUID
    if !is_valid_uuid(session_id) {
        return None;
    }

    Some(FileHistoryInfo {
        session_id: session_id.to_string(),
        file_hash: hash_part.to_string(),
        version,
    })
}

/// Parses a session JSONL path to extract the session UUID and project slug.
///
/// Session JSONL paths have the format:
/// `~/.claude/projects/<project-slug>/<session-uuid>.jsonl`
///
/// # Arguments
///
/// * `path` - The path to the session JSONL file
///
/// # Returns
///
/// - `Some(SessionJsonlInfo)` if the path matches the expected pattern
/// - `None` if the path doesn't match (missing parent, invalid UUID, etc.)
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use vibetea_monitor::utils::session_filename::parse_session_jsonl_path;
///
/// // Valid session JSONL path
/// let path = Path::new("/home/.claude/projects/-home-user-project/6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl");
/// let info = parse_session_jsonl_path(path).unwrap();
/// assert_eq!(info.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
/// assert_eq!(info.project_slug, "-home-user-project");
///
/// // Invalid: not a .jsonl file
/// let path = Path::new("/home/.claude/projects/-home-user-project/6e45a55c-3124-4cc8-ad85-040a5c316009.json");
/// assert!(parse_session_jsonl_path(path).is_none());
///
/// // Invalid: filename is not a valid UUID
/// let path = Path::new("/home/.claude/projects/-home-user-project/not-a-uuid.jsonl");
/// assert!(parse_session_jsonl_path(path).is_none());
/// ```
#[must_use]
pub fn parse_session_jsonl_path(path: &Path) -> Option<SessionJsonlInfo> {
    // Get the filename
    let filename = path.file_name()?.to_str()?;

    // Must end with .jsonl
    let session_id = filename.strip_suffix(".jsonl")?;

    // Validate that it's a valid UUID format
    if !is_valid_uuid(session_id) {
        return None;
    }

    // Get the parent directory (project slug)
    let parent = path.parent()?;
    let project_slug = parent.file_name()?.to_str()?;

    // Project slug should not be empty
    if project_slug.is_empty() {
        return None;
    }

    Some(SessionJsonlInfo {
        session_id: session_id.to_string(),
        project_slug: project_slug.to_string(),
    })
}

/// Validates that a string is a valid UUID format.
///
/// Checks for the standard UUID format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
/// where each x is a hexadecimal digit.
fn is_valid_uuid(s: &str) -> bool {
    // UUID format: 8-4-4-4-12 = 36 characters total
    if s.len() != 36 {
        return false;
    }

    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return false;
    }

    // Check each part has the correct length and is valid hex
    let expected_lengths = [8, 4, 4, 4, 12];
    for (part, &expected_len) in parts.iter().zip(expected_lengths.iter()) {
        if part.len() != expected_len {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }

    true
}

/// Validates that a string is a valid 16-character hexadecimal hash.
fn is_valid_hex_hash(s: &str) -> bool {
    s.len() == 16 && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Tests for parse_todo_filename =====

    #[test]
    fn test_parse_todo_filename_valid() {
        let path = Path::new(
            "/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json",
        );
        assert_eq!(
            parse_todo_filename(path),
            Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
        );
    }

    #[test]
    fn test_parse_todo_filename_different_uuids() {
        // The two UUIDs in the filename can be different
        let path = Path::new(
            "/home/user/.claude/todos/a1b2c3d4-e5f6-7890-abcd-ef1234567890-agent-f1e2d3c4-b5a6-0987-fedc-ba9876543210.json",
        );
        assert_eq!(
            parse_todo_filename(path),
            Some("a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string())
        );
    }

    #[test]
    fn test_parse_todo_filename_no_agent_separator() {
        let path = Path::new("/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009.json");
        assert_eq!(parse_todo_filename(path), None);
    }

    #[test]
    fn test_parse_todo_filename_wrong_extension() {
        let path = Path::new(
            "/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.txt",
        );
        assert_eq!(parse_todo_filename(path), None);
    }

    #[test]
    fn test_parse_todo_filename_no_extension() {
        let path = Path::new(
            "/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009",
        );
        assert_eq!(parse_todo_filename(path), None);
    }

    #[test]
    fn test_parse_todo_filename_invalid_uuid_format() {
        let path = Path::new("/home/user/.claude/todos/not-a-valid-uuid-agent-something.json");
        assert_eq!(parse_todo_filename(path), None);
    }

    #[test]
    fn test_parse_todo_filename_empty_path() {
        let path = Path::new("");
        assert_eq!(parse_todo_filename(path), None);
    }

    #[test]
    fn test_parse_todo_filename_just_filename() {
        let path = Path::new(
            "6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json",
        );
        assert_eq!(
            parse_todo_filename(path),
            Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
        );
    }

    #[test]
    fn test_parse_todo_filename_uppercase_uuid() {
        let path = Path::new(
            "/home/user/.claude/todos/6E45A55C-3124-4CC8-AD85-040A5C316009-agent-6E45A55C-3124-4CC8-AD85-040A5C316009.json",
        );
        assert_eq!(
            parse_todo_filename(path),
            Some("6E45A55C-3124-4CC8-AD85-040A5C316009".to_string())
        );
    }

    #[test]
    fn test_parse_todo_filename_mixed_case_uuid() {
        let path = Path::new(
            "/home/user/.claude/todos/6e45A55c-3124-4Cc8-aD85-040a5C316009-agent-something.json",
        );
        assert_eq!(
            parse_todo_filename(path),
            Some("6e45A55c-3124-4Cc8-aD85-040a5C316009".to_string())
        );
    }

    #[test]
    fn test_parse_todo_filename_multiple_agent_separators() {
        // Should take the first part before the first "-agent-"
        let path = Path::new(
            "/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-middle-agent-end.json",
        );
        assert_eq!(
            parse_todo_filename(path),
            Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
        );
    }

    // ===== Tests for parse_file_history_path =====

    #[test]
    fn test_parse_file_history_path_valid() {
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/3a8f2c1b9e4d7a6f@v2",
        );
        let info = parse_file_history_path(path).unwrap();
        assert_eq!(info.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
        assert_eq!(info.file_hash, "3a8f2c1b9e4d7a6f");
        assert_eq!(info.version, 2);
    }

    #[test]
    fn test_parse_file_history_path_version_zero() {
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/deadbeef12345678@v0",
        );
        let info = parse_file_history_path(path).unwrap();
        assert_eq!(info.version, 0);
    }

    #[test]
    fn test_parse_file_history_path_large_version() {
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/deadbeef12345678@v999",
        );
        let info = parse_file_history_path(path).unwrap();
        assert_eq!(info.version, 999);
    }

    #[test]
    fn test_parse_file_history_path_no_version_suffix() {
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/3a8f2c1b9e4d7a6f",
        );
        assert!(parse_file_history_path(path).is_none());
    }

    #[test]
    fn test_parse_file_history_path_invalid_version() {
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/3a8f2c1b9e4d7a6f@vabc",
        );
        assert!(parse_file_history_path(path).is_none());
    }

    #[test]
    fn test_parse_file_history_path_negative_version() {
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/3a8f2c1b9e4d7a6f@v-1",
        );
        assert!(parse_file_history_path(path).is_none());
    }

    #[test]
    fn test_parse_file_history_path_invalid_session_id() {
        let path = Path::new("/home/.claude/file-history/invalid-session-id/3a8f2c1b9e4d7a6f@v1");
        assert!(parse_file_history_path(path).is_none());
    }

    #[test]
    fn test_parse_file_history_path_invalid_hash_length() {
        // Hash is too short (only 8 characters)
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/deadbeef@v1",
        );
        assert!(parse_file_history_path(path).is_none());
    }

    #[test]
    fn test_parse_file_history_path_invalid_hash_characters() {
        // Hash contains non-hex characters
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/ghijklmnopqrstuv@v1",
        );
        assert!(parse_file_history_path(path).is_none());
    }

    #[test]
    fn test_parse_file_history_path_no_parent() {
        let path = Path::new("3a8f2c1b9e4d7a6f@v1");
        assert!(parse_file_history_path(path).is_none());
    }

    #[test]
    fn test_parse_file_history_path_uppercase_hash() {
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/DEADBEEF12345678@v1",
        );
        let info = parse_file_history_path(path).unwrap();
        assert_eq!(info.file_hash, "DEADBEEF12345678");
    }

    #[test]
    fn test_parse_file_history_path_multiple_at_signs() {
        // Should only split on the first "@v"
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/3a8f2c1b9e4d7a6f@v2@v3",
        );
        // This should fail because "2@v3" is not a valid version number
        assert!(parse_file_history_path(path).is_none());
    }

    #[test]
    fn test_parse_file_history_path_empty_version() {
        let path = Path::new(
            "/home/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/3a8f2c1b9e4d7a6f@v",
        );
        assert!(parse_file_history_path(path).is_none());
    }

    // ===== Tests for parse_session_jsonl_path =====

    #[test]
    fn test_parse_session_jsonl_path_valid() {
        let path = Path::new(
            "/home/.claude/projects/-home-user-project/6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl",
        );
        let info = parse_session_jsonl_path(path).unwrap();
        assert_eq!(info.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
        assert_eq!(info.project_slug, "-home-user-project");
    }

    #[test]
    fn test_parse_session_jsonl_path_complex_slug() {
        let path = Path::new(
            "/home/.claude/projects/-home-user-deep-nested-project/a1b2c3d4-e5f6-7890-abcd-ef1234567890.jsonl",
        );
        let info = parse_session_jsonl_path(path).unwrap();
        assert_eq!(info.session_id, "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
        assert_eq!(info.project_slug, "-home-user-deep-nested-project");
    }

    #[test]
    fn test_parse_session_jsonl_path_wrong_extension() {
        let path = Path::new(
            "/home/.claude/projects/-home-user-project/6e45a55c-3124-4cc8-ad85-040a5c316009.json",
        );
        assert!(parse_session_jsonl_path(path).is_none());
    }

    #[test]
    fn test_parse_session_jsonl_path_no_extension() {
        let path = Path::new(
            "/home/.claude/projects/-home-user-project/6e45a55c-3124-4cc8-ad85-040a5c316009",
        );
        assert!(parse_session_jsonl_path(path).is_none());
    }

    #[test]
    fn test_parse_session_jsonl_path_invalid_uuid() {
        let path = Path::new("/home/.claude/projects/-home-user-project/not-a-valid-uuid.jsonl");
        assert!(parse_session_jsonl_path(path).is_none());
    }

    #[test]
    fn test_parse_session_jsonl_path_no_parent() {
        let path = Path::new("6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl");
        assert!(parse_session_jsonl_path(path).is_none());
    }

    #[test]
    fn test_parse_session_jsonl_path_root_parent() {
        let path = Path::new("/6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl");
        // Parent is "/" which has no file_name component
        assert!(parse_session_jsonl_path(path).is_none());
    }

    #[test]
    fn test_parse_session_jsonl_path_uppercase_uuid() {
        let path = Path::new(
            "/home/.claude/projects/myproject/6E45A55C-3124-4CC8-AD85-040A5C316009.jsonl",
        );
        let info = parse_session_jsonl_path(path).unwrap();
        assert_eq!(info.session_id, "6E45A55C-3124-4CC8-AD85-040A5C316009");
    }

    // ===== Tests for is_valid_uuid =====

    #[test]
    fn test_is_valid_uuid_valid() {
        assert!(is_valid_uuid("6e45a55c-3124-4cc8-ad85-040a5c316009"));
        assert!(is_valid_uuid("00000000-0000-0000-0000-000000000000"));
        assert!(is_valid_uuid("ffffffff-ffff-ffff-ffff-ffffffffffff"));
        assert!(is_valid_uuid("AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE"));
    }

    #[test]
    fn test_is_valid_uuid_invalid_length() {
        assert!(!is_valid_uuid("6e45a55c-3124-4cc8-ad85-040a5c31600")); // Too short
        assert!(!is_valid_uuid("6e45a55c-3124-4cc8-ad85-040a5c3160090")); // Too long
        assert!(!is_valid_uuid("")); // Empty
    }

    #[test]
    fn test_is_valid_uuid_invalid_format() {
        assert!(!is_valid_uuid("6e45a55c31244cc8ad85040a5c316009")); // No dashes
        assert!(!is_valid_uuid("6e45a55c-31244cc8-ad85-040a5c316009")); // Wrong dash positions
        assert!(!is_valid_uuid("6e45a55c-3124-4cc8-ad85040a5c316009")); // Missing dash
    }

    #[test]
    fn test_is_valid_uuid_invalid_characters() {
        assert!(!is_valid_uuid("6e45a55c-3124-4cc8-ad85-040a5c31600g")); // 'g' is not hex
        assert!(!is_valid_uuid("6e45a55c-3124-4cc8-ad85-040a5c31600!")); // Special char
        assert!(!is_valid_uuid("6e45a55c-3124-4cc8-ad85-040a5c31600 ")); // Space
    }

    // ===== Tests for is_valid_hex_hash =====

    #[test]
    fn test_is_valid_hex_hash_valid() {
        assert!(is_valid_hex_hash("3a8f2c1b9e4d7a6f"));
        assert!(is_valid_hex_hash("0000000000000000"));
        assert!(is_valid_hex_hash("ffffffffffffffff"));
        assert!(is_valid_hex_hash("ABCDEF1234567890"));
    }

    #[test]
    fn test_is_valid_hex_hash_invalid_length() {
        assert!(!is_valid_hex_hash("3a8f2c1b9e4d7a6")); // Too short (15)
        assert!(!is_valid_hex_hash("3a8f2c1b9e4d7a6f0")); // Too long (17)
        assert!(!is_valid_hex_hash("")); // Empty
    }

    #[test]
    fn test_is_valid_hex_hash_invalid_characters() {
        assert!(!is_valid_hex_hash("3a8f2c1b9e4d7a6g")); // 'g' is not hex
        assert!(!is_valid_hex_hash("3a8f2c1b9e4d7a6!")); // Special char
        assert!(!is_valid_hex_hash("3a8f2c1b9e4d7a6 ")); // Space
    }

    // ===== Edge cases and real-world scenarios =====

    #[test]
    fn test_windows_style_paths() {
        // Windows paths with backslashes won't work with Path on Unix,
        // but forward slashes work on both
        let path = Path::new(
            "C:/Users/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json",
        );
        assert_eq!(
            parse_todo_filename(path),
            Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
        );
    }

    #[test]
    fn test_relative_paths() {
        let path = Path::new(
            ".claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json",
        );
        assert_eq!(
            parse_todo_filename(path),
            Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
        );
    }

    #[test]
    fn test_tilde_paths() {
        // Note: Path doesn't expand ~, but the pattern matching still works on the filename
        let path = Path::new(
            "~/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json",
        );
        assert_eq!(
            parse_todo_filename(path),
            Some("6e45a55c-3124-4cc8-ad85-040a5c316009".to_string())
        );
    }

    #[test]
    fn test_file_history_deep_nested_path() {
        let path = Path::new(
            "/very/deep/nested/path/.claude/file-history/6e45a55c-3124-4cc8-ad85-040a5c316009/deadbeef12345678@v42",
        );
        let info = parse_file_history_path(path).unwrap();
        assert_eq!(info.session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
        assert_eq!(info.file_hash, "deadbeef12345678");
        assert_eq!(info.version, 42);
    }

    #[test]
    fn test_session_jsonl_simple_project_name() {
        let path = Path::new(
            "/home/.claude/projects/myproject/6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl",
        );
        let info = parse_session_jsonl_path(path).unwrap();
        assert_eq!(info.project_slug, "myproject");
    }
}
