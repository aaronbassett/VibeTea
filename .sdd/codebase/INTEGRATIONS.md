# External Integrations

**Status**: Phase 11 - Project activity tracking integration via file system watching
**Last Updated**: 2026-02-04

## Summary

VibeTea is designed as a distributed event system with three components:
- **Monitor**: Captures Claude Code session events from local JSONL files, applies privacy sanitization, signs with Ed25519, and transmits to server via HTTP. Supports `export-key` command for GitHub Actions integration (Phase 4). Can be deployed in GitHub Actions workflows (Phase 5). Integrated via reusable GitHub Actions composite action (Phase 6). Now tracks project-level activity via directory scanning (Phase 11).
- **Server**: Receives, validates, verifies Ed25519 signatures, and broadcasts events via WebSocket
- **Client**: Subscribes to server events via WebSocket for visualization with token-based authentication

All integrations use standard protocols (HTTPS, WebSocket) with cryptographic message authentication and privacy-by-design data handling.

## File System Integration

### Claude Code Session Files (JSONL)

**Source**: `~/.claude/projects/**/*.jsonl`
**Format**: JSON Lines (one JSON object per line, append-only)
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents)

**Parser Location**: `monitor/src/parser.rs` (SessionParser, ParsedEvent, ParsedEventKind)
**Watcher Location**: `monitor/src/watcher.rs` (FileWatcher, WatchEvent)
**Privacy Pipeline**: `monitor/src/privacy.rs` (PrivacyConfig, PrivacyPipeline)
**Agent Tracker**: `monitor/src/trackers/agent_tracker.rs` (Task tool agent spawn tracking)

**Privacy-First Approach**:
- Only metadata extracted: tool names, timestamps, file basenames, agent types
- Never processes code content, prompts, or responses
- File path parsing for project name extraction
- All event payloads sanitized through PrivacyPipeline

**Session File Structure**:
```
~/.claude/projects/<project-slug>/<session-uuid>.jsonl
```

**Supported Event Types** (from Claude Code JSONL):
| Claude Code Type | Parsed As | VibeTea Event | Fields |
|------------------|-----------|---------------|--------|
| `assistant` with `tool_use` (non-Task) | Tool invocation | ToolStarted | tool name, context |
| `assistant` with `tool_use` (Task tool) | Agent spawn | AgentSpawned | agent_type, description |
| `progress` with `PostToolUse` | Tool completion | ToolCompleted | tool name, success |
| `user` | User activity | Activity | timestamp only |
| `summary` | Session end marker | Summary | session metadata |
| File creation | Session start | SessionStarted | project from path |

**Watcher Behavior**:
- Monitors `~/.claude/projects/` directory recursively
- Detects file creation, modification, deletion events
- Maintains position map for efficient tailing (no re-reading)
- Emits WatchEvent::FileCreated, WatchEvent::LinesAdded, WatchEvent::FileRemoved

**Configuration** (`monitor/src/config.rs`):
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | Claude directory to monitor |
| `VIBETEA_BASENAME_ALLOWLIST` | No | - | Comma-separated file extensions to watch |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |

### Claude Code History File (Phase 5)

**Source**: `~/.claude/history.jsonl`
**Format**: JSON Lines (one JSON object per line, append-only)
**Update Mechanism**: File system watcher via `notify` crate

**Skill Tracker Location**: `monitor/src/trackers/skill_tracker.rs` (1837 lines)
**Tokenizer Location**: `monitor/src/utils/tokenize.rs`

**Purpose**: Real-time tracking of user skill/slash command invocations

**History.jsonl Structure**:
```json
{
  "display": "/commit -m \"fix: update docs\"",
  "timestamp": 1738567268363,
  "project": "/home/ubuntu/Projects/VibeTea",
  "sessionId": "6e45a55c-3124-4cc8-ad85-040a5c316009"
}
```

**Fields**:
- `display`: Command string with arguments (e.g., "/commit -m \"message\"")
- `timestamp`: Unix milliseconds
- `project`: Absolute path to project root
- `sessionId`: UUID of Claude Code session

**Privacy-First Approach**:
- Only skill name extracted (e.g., "commit" from "/commit -m \"fix\"")
- Command arguments never transmitted
- Project path included for context (identifies project, not code)

**Skill Tracker Module** (`monitor/src/trackers/skill_tracker.rs`):

1. **Core Types**:
   - `SkillInvocationEvent` - Emitted when user invokes a skill
   - `HistoryEntry` - Parsed entry from history.jsonl
   - `SkillTracker` - File watcher and parser
   - `SkillTrackerConfig` - Startup behavior configuration

2. **Parsing Functions**:
   - `parse_history_entry(line)` - Parses JSON with validation
   - `parse_history_entries(content)` - Parses multiple lines, lenient
   - `create_skill_invocation_event(entry)` - Constructs event

3. **File Watching**:
   - Watches parent directory of history.jsonl
   - Detects file creation, modification
   - Maintains atomic byte offset
   - Handles truncation gracefully
   - Emits SkillInvocationEvent via mpsc channel

4. **Skill Name Extraction** (`monitor/src/utils/tokenize.rs`):
   - `extract_skill_name(display)` - Parses from display string
   - Handles `/commit` → `commit`
   - Handles `/sdd:plan` → `sdd:plan`
   - Handles `/review-pr` → `review-pr`
   - Handles arguments: `/commit -m \"fix\"` → `commit`

**Configuration**:
- No specific environment variables (uses default ~/.claude)
- Optional: Extend to support custom history.jsonl paths

**Test Coverage**: 60+ comprehensive tests covering:
- Parsing (12 tests)
- Multiple entries (6 tests)
- Methods (5 tests)
- Skill extraction (10 tests)
- Event creation (5 tests)
- File operations (12+ async tests)

### Claude Code Todo Files (Phase 6)

**Source**: `~/.claude/todos/<session-uuid>-agent-<session-uuid>.json`
**Format**: JSON Array of todo objects
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents)

**Todo Tracker Location**: `monitor/src/trackers/todo_tracker.rs` (2345 lines)
**Utility Location**: `monitor/src/utils/debounce.rs`, `monitor/src/utils/session_filename.rs`

**Purpose**: Track todo list progress and detect abandoned tasks per session

**Todo File Structure**:
```json
[
  {
    "content": "Task description text",
    "status": "completed",
    "activeForm": "Completing task..."
  },
  {
    "content": "Another task",
    "status": "in_progress",
    "activeForm": "Working on task..."
  },
  {
    "content": "Pending task",
    "status": "pending",
    "activeForm": null
  }
]
```

**Fields**:
- `content`: Task description (never transmitted for privacy)
- `status`: One of `completed`, `in_progress`, `pending`
- `activeForm`: Optional active form text shown during task execution

**Privacy-First Approach**:
- Only status counts extracted: completed, in_progress, pending
- Task content (`content` field) never read or transmitted
- Abandonment detection for analysis (did tasks go incomplete?)
- Session context preserved for correlation

**Todo Tracker Module** (`monitor/src/trackers/todo_tracker.rs`):

1. **Core Types**:
   - `TodoProgressEvent` - Emitted when todo list changes
   - `TodoEntry` - Individual todo item
   - `TodoStatus` - Enum: Completed, InProgress, Pending
   - `TodoStatusCounts` - Aggregated counts by status
   - `TodoTracker` - File watcher for todos directory
   - `TodoTrackerConfig` - Configuration (debounce duration)
   - `TodoParseError` / `TodoTrackerError` - Comprehensive error types

2. **Parsing Functions**:
   - `parse_todo_file(content)` - Strict JSON array parsing
   - `parse_todo_file_lenient(content)` - Lenient parsing, skips invalid entries
   - `parse_todo_entry(value)` - Single entry validation
   - `count_todo_statuses(entries)` - Aggregate counts
   - `extract_session_id_from_filename(path)` - UUID extraction

3. **Abandonment Detection**:
   - `is_abandoned(counts, session_ended)` - True if session ended with incomplete tasks
   - `create_todo_progress_event(session_id, counts, abandoned)` - Event construction
   - Requires explicit session ended tracking via `mark_session_ended()`

4. **File Watching**:
   - Monitors `~/.claude/todos/` directory (non-recursive)
   - Detects .json file creation and modification
   - Validates filename format: `<uuid>-agent-<uuid>.json`
   - Debounces rapid changes (100ms default)
   - Uses notify crate for cross-platform compatibility
   - Maintains RwLock<HashSet> of ended sessions
   - Lenient parsing handles partially-written files

5. **Session Lifecycle Integration**:
   - `mark_session_ended(session_id)` - Call when summary event received
   - `is_session_ended(session_id)` - Query ended status
   - `clear_session_ended(session_id)` - Reset ended status
   - Abandonment flag set only if session ended AND incomplete tasks exist

**Configuration**:
- Default location: `~/.claude/todos/`
- Debounce interval: 100ms (coalesce rapid writes)
- No environment variables required (uses `directories` crate)

**Test Coverage**: 100+ comprehensive tests:
- Filename parsing (8 tests)
- Status counting (6 tests)
- Abandonment detection (6 tests)
- Entry parsing (8 tests)
- File parsing (8 tests)
- Lenient parsing (4 tests)
- Trait implementations (3 tests)
- Error messages (2 tests)
- Configuration (2 tests)
- File operations and async (12+ tests)

**Debouncing Implementation** (`monitor/src/utils/debounce.rs`):
- Generic `Debouncer<K, V>` for generic key-value coalescing
- Configurable duration (100ms for todos)
- mpsc channel based event emission
- Prevents duplicate processing of rapid file changes

**Filename Parsing** (`monitor/src/utils/session_filename.rs`):
- `parse_todo_filename(path)` - Extracts session UUID from filename
- Pattern: `<session-uuid>-agent-<session-uuid>.json`
- Returns Option<String> with first UUID

### Claude Code Stats Cache (Phase 8, Phase 10)

**Source**: `~/.claude/stats-cache.json`
**Format**: JSON object with model usage and session metrics
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents)

**Stats Tracker Location**: `monitor/src/trackers/stats_tracker.rs` (1400+ lines)

**Purpose**: Track token usage per model, global session statistics, hourly activity patterns, and model distribution

**Stats Cache File Structure**:
```json
{
  "totalSessions": 150,
  "totalMessages": 2500,
  "totalToolUsage": 8000,
  "longestSession": "00:45:30",
  "hourCounts": { "0": 10, "1": 5, ..., "23": 50 },
  "modelUsage": {
    "claude-sonnet-4-20250514": {
      "inputTokens": 1500000,
      "outputTokens": 300000,
      "cacheReadInputTokens": 800000,
      "cacheCreationInputTokens": 100000
    },
    "claude-opus-4-20250514": {
      "inputTokens": 500000,
      "outputTokens": 150000,
      "cacheReadInputTokens": 200000,
      "cacheCreationInputTokens": 50000
    }
  }
}
```

**Fields**:
- `totalSessions`: Total number of Claude Code sessions
- `totalMessages`: Total messages across all sessions
- `totalToolUsage`: Total tool invocations
- `longestSession`: Duration string (HH:MM:SS format)
- `hourCounts`: Activity distribution by hour of day (0-23)
- `modelUsage`: Per-model token consumption with cache metrics

**Privacy-First Approach**:
- Only aggregate statistics extracted (never session/prompt content)
- No model-specific information beyond model name
- Cache metrics only (no raw data)
- Session context derived from file name parsing only

**Stats Tracker Module** (`monitor/src/trackers/stats_tracker.rs`):

1. **Core Types**:
   - `StatsEvent` - Enum with variants: `SessionMetrics`, `TokenUsage`, `ActivityPattern` (Phase 10), `ModelDistribution` (Phase 10)
   - `SessionMetricsEvent` - Global session statistics
   - `TokenUsageEvent` - Per-model token consumption
   - `ActivityPatternEvent` (Phase 10) - Hourly activity distribution
   - `ModelDistributionEvent` (Phase 10) - Per-model usage breakdown
   - `TokenUsageSummary` (Phase 10) - Token counts for models
   - `StatsCache` - Deserialized stats-cache.json
   - `ModelTokens` - Per-model token counts
   - `StatsTracker` - File watcher for stats-cache.json
   - `StatsTrackerError` - Comprehensive error types

2. **Parsing Functions**:
   - `read_stats_with_retry()` - Reads with retry logic (up to 3 attempts)
   - `read_stats()` - Synchronous file read and parse
   - `parse_stats_cache()` - Public helper for testing
   - `emit_stats_events()` - Creates all event types

3. **Event Emission** (Phase 10):
   - Emits `SessionMetricsEvent` once per stats-cache.json read
   - Emits `ActivityPatternEvent` containing hourly breakdown (before token events)
   - Emits `TokenUsageEvent` for each model in modelUsage
   - Emits `ModelDistributionEvent` with all model aggregations (after token events)
   - Per-model events include: input tokens, output tokens, cache read tokens, cache creation tokens

4. **Phase 10 Event Details**:

   **ActivityPatternEvent**:
   - Field: `hour_counts: HashMap<String, u64>`
   - Source: Direct from stats-cache.json `hourCounts`
   - Keys: String keys "0" through "23" (for JSON deserialization reliability)
   - Values: Activity count per hour
   - Purpose: Real-time hourly distribution visualization

   **ModelDistributionEvent**:
   - Field: `model_usage: HashMap<String, TokenUsageSummary>`
   - Source: Aggregated from stats-cache.json `modelUsage`
   - Maps model names to their complete token breakdown
   - TokenUsageSummary contains:
     - `input_tokens: u64`
     - `output_tokens: u64`
     - `cache_read_tokens: u64`
     - `cache_creation_tokens: u64`
   - Purpose: Model-level usage distribution and cost analysis

5. **File Watching**:
   - Monitors `~/.claude/stats-cache.json` for changes
   - 200ms debounce interval to coalesce rapid writes
   - Handles initial read if file exists on startup
   - Retries JSON parse with 100ms delays (up to 3 attempts)
   - Uses notify crate for cross-platform FSEvents/inotify
   - Graceful degradation if file unavailable

6. **Main Event Loop Integration**:
   - StatsTracker initialization during startup (optional, warns on failure)
   - Dedicated channel: `mpsc::channel::<StatsEvent>`
   - Stats event processing in main select! loop
   - `process_stats_event()` handler in main.rs converts to Event

**Configuration**:
- Default location: `~/.claude/stats-cache.json`
- Debounce interval: 200ms
- Parse retry delay: 100ms
- Max retries: 3 attempts
- No environment variables required (uses `directories` crate)

**Test Coverage**: 60+ comprehensive tests covering:
- JSON parsing (7 tests)
- Model token parsing (6 tests)
- Empty/partial stats (3 tests)
- Malformed JSON handling (3 tests)
- Stats event emission (3 tests)
- Debounce timing (2 tests)
- Parse retry logic (2 tests)
- Missing/malformed files (3 tests)
- Tracker creation (2 tests)
- Initial read behavior (1 test)
- Refresh method (1 test)
- Phase 10: ActivityPatternEvent tests (3 tests)
- Phase 10: ModelDistributionEvent tests (3 tests)
- Enum/equality tests (10 tests)

### Claude Code Project Directory (Phase 11)

**Source**: `~/.claude/projects/`
**Format**: Directory structure with project slugs and session JSONL files
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents/ReadDirectoryChangesW)

**Project Tracker Location**: `monitor/src/trackers/project_tracker.rs` (500+ lines)

**Purpose**: Monitor which projects have active Claude Code sessions and track session completion state

**Directory Structure**:
```
~/.claude/projects/
+-- -home-ubuntu-Projects-VibeTea/
|   +-- 6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl  (active session)
|   +-- a1b2c3d4-5678-90ab-cdef-1234567890ab.jsonl  (completed session)
+-- -home-ubuntu-Projects-SMILE/
    +-- 60fc5b5e-a285-4a6d-b9cc-9a315eb90ea8.jsonl
```

**Project Slug Format**:
- Absolute paths have forward slashes replaced with dashes
- `/home/ubuntu/Projects/VibeTea` becomes `-home-ubuntu-Projects-VibeTea`
- Slug format is set by Claude Code; monitor only reads it

**Session Activity Detection**:
- A session is **active** if its JSONL file does NOT contain a `{"type": "summary", ...}` event
- A session is **completed** once a summary event is present
- Summary events indicate session end markers

**Privacy-First Approach**:
- Only project paths and session IDs tracked
- No code content, prompts, or responses
- No session details beyond activity status
- Session completion detected via file presence, not content analysis

**Project Tracker Module** (`monitor/src/trackers/project_tracker.rs`):

1. **Core Types**:
   - `ProjectActivityEvent` - Emitted when project session status changes
     - `project_path: String` - Absolute path to project
     - `session_id: String` - Session UUID
     - `is_active: bool` - True if session has no summary event
   - `ProjectTracker` - File system watcher for projects directory
   - `ProjectTrackerConfig` - Tracker configuration
   - `ProjectTrackerError` - Error handling

2. **Utility Functions**:
   - `parse_project_slug(slug)` - Converts slug back to absolute path
   - `has_summary_event(content)` - Detects session completion
   - `create_project_activity_event()` - Factory for event construction

3. **File Watching**:
   - Monitors `~/.claude/projects/` recursively
   - Detects .jsonl file creation and modification
   - Reads file content to check for summary events
   - Maintains session state
   - Emits ProjectActivityEvent via mpsc channel

4. **Features**:
   - No debouncing needed (project files change infrequently)
   - Async/await compatible with tokio runtime
   - Thread-safe via mpsc channels
   - Graceful error handling and logging
   - Initial scan option on startup (default: enabled)

5. **Configuration** (`ProjectTrackerConfig`):
   - `scan_on_init: bool` - Whether to scan all projects on startup
   - Default: true (scan existing projects)
   - Useful for initial dashboard population

6. **Error Handling**:
   - `WatcherInit` - File system watcher setup failures
   - `Io` - File read/write errors
   - `ClaudeDirectoryNotFound` - Missing project directory
   - `ChannelClosed` - Event sender channel unavailable

**Use Cases**:
- Multi-project activity overview
- Detect active sessions across projects
- Monitor completion state changes
- Project-based filtering in dashboards
- Correlate tool usage by project

**Test Coverage**: 50+ tests covering:
- Slug parsing (bidirectional)
- Summary event detection
- Event creation and serialization
- File watching and change detection
- Error handling and edge cases
- Async tracker initialization

## Privacy & Data Sanitization

### Privacy Pipeline Architecture

**Location**: `monitor/src/privacy.rs` (1039 lines)

**Core Components**:

1. **PrivacyConfig** - Configuration management
   - Optional extension allowlist (e.g., `.rs`, `.ts`)
   - Loaded from `VIBETEA_BASENAME_ALLOWLIST`
   - Supports comma-separated format

2. **PrivacyPipeline** - Event sanitization processor
   - Processes EventPayload before transmission
   - Strips sensitive contexts
   - Extracts basenames from paths
   - Applies extension filtering
   - Neutralizes summary text

3. **extract_basename()** - Path safety function
   - `/home/user/src/auth.ts` → `auth.ts`
   - Handles Unix, Windows, relative paths
   - Returns `None` for invalid paths

**Sensitive Tools** (context always stripped):
- `Bash` - Commands may contain secrets
- `Grep` - Patterns reveal search intent
- `Glob` - Patterns reveal project structure
- `WebSearch` - Queries reveal intent
- `WebFetch` - URLs may contain secrets

**Privacy Processing Rules**:
| Payload Type | Processing |
|--------------|-----------|
| Session | Pass through |
| Activity | Pass through |
| Tool (sensitive) | Context set to None |
| Tool (other) | Basename + allowlist filtering |
| Agent | Pass through unchanged |
| AgentSpawn | Pass through unchanged |
| SkillInvocation | Pass through unchanged |
| TodoProgress | Pass through unchanged (only counts) |
| SessionMetrics | Pass through unchanged (aggregate data) |
| TokenUsage | Pass through unchanged (aggregate data) |
| ActivityPattern | Pass through unchanged (hourly counts) |
| ModelDistribution | Pass through unchanged (model usage) |
| ProjectActivity | Pass through unchanged (paths & session IDs) |
| Summary | Text replaced with "Session ended" |
| Error | Pass through unchanged |

**Extension Allowlist Filtering**:
- Not set: All extensions allowed
- Set to `.rs,.ts`: Only those extensions transmitted
- Mismatch: Context filtered to `None`

**Todo Privacy**:
- TodoProgressEvent contains only counts and abandonment flag
- No task content or descriptions transmitted
- Counts are aggregate, non-sensitive metadata

**Stats Privacy** (Phase 10):
- SessionMetricsEvent contains only aggregate counts
- TokenUsageEvent contains per-model consumption metrics
- ActivityPatternEvent contains hourly distribution (no session data)
- ModelDistributionEvent contains aggregated usage by model (no session data)
- No per-session data or user information
- Cache metrics are transparent usage data

**Project Privacy** (Phase 11):
- ProjectActivityEvent contains only project path and session ID
- No project content or code
- Only session activity status (active/completed)
- Paths are absolute (user-identified) for context

### Privacy Test Suite

**Location**: `monitor/tests/privacy_test.rs` (951 lines)

**Coverage**: 18+ comprehensive privacy compliance tests
**Validates**: Constitution I (Privacy by Design)

**Test Categories**:
1. **Path Sanitization**
   - No full paths in output (Unix, Windows, relative)
   - Basenames correctly extracted
   - Hidden files handled

2. **Sensitive Tool Stripping**
   - Bash commands removed entirely
   - Grep patterns omitted
   - Glob patterns stripped
   - WebSearch queries removed
   - WebFetch URLs removed

3. **Content Stripping**
   - File contents never transmitted
   - Diffs excluded from payloads
   - Code excerpts removed

4. **Prompt/Response Stripping**
   - User prompts not included
   - Assistant responses excluded
   - Message content sanitized

5. **Command Argument Removal**
   - Arguments separated from descriptions
   - Descriptions allowed for Bash context
   - Actual commands never sent

6. **Summary Neutralization**
   - Summary text set to generic "Session ended"
   - Original text discarded
   - No content leakage

7. **Extension Allowlist Filtering**
   - Correct files allowed through
   - Disallowed extensions filtered
   - No-extension files handled properly

8. **Sensitive Pattern Detection**
   - Path patterns never appear (e.g., `/home/`, `/Users/`, `C:\`)
   - Command patterns removed (e.g., `rm -rf`, `sudo`, `curl -`, `Bearer`)
   - Credentials not transmitted

## Cryptographic Authentication & Key Management

### Phase 2: Enhanced Crypto Module with KeySource Tracking

**Module Location**: `monitor/src/crypto.rs` (438+ lines)

**KeySource Enum** (Phase 2 Addition):
- **Purpose**: Track where the private key was loaded from for audit/logging purposes
- **Variants**:
  - `EnvironmentVariable` - Key loaded from `VIBETEA_PRIVATE_KEY` environment variable
  - `File(PathBuf)` - Key loaded from file at specific path
- **Usage**: Enables reporting key source at startup for transparency
- **Logging**: Can be reported at INFO level to help users verify correct key usage

**Public Key Fingerprinting** (Phase 2 Addition):
- **public_key_fingerprint()**: New method returns first 8 characters of base64-encoded public key
  - Used for key verification in logs without exposing full key
  - Allows users to verify correct keypair with server registration
  - Always 8 characters long, guaranteed to be unique prefix of full key
  - Useful for quick visual verification in logs and documentation
  - Example: Full key `dGVzdHB1YmtleTExYWJjZGVmZ2hpams=` → Fingerprint `dGVzdHB1`

**Backward Compatibility**:
- KeySource and fingerprinting are tracking/logging features only
- Do not affect cryptographic operations (signing/verification)
- Existing code continues to work without modification
- New features are opt-in for enhanced observability

### Phase 3: Memory Safety & Environment Variable Key Loading

**Module Location**: `monitor/src/crypto.rs` (438+ lines)

**zeroize Crate Integration** (v1.8):
- Securely wipes sensitive memory (seed bytes, decoded buffers) after use
- Applied in key generation: seed zeroized after SigningKey construction
- Applied in load_from_env(): decoded buffer zeroized on both success and error paths
- Applied in load_with_fallback(): decoded buffer zeroized on error paths
- Prevents sensitive key material from remaining in memory dumps
- Complies with FR-020: Zero intermediate key material after key operations

**load_from_env() Method** (Phase 3 Addition):
- Loads Ed25519 private key from `VIBETEA_PRIVATE_KEY` environment variable
- Expects base64-encoded 32-byte seed (RFC 4648 standard)
- Trims whitespace (including newlines) before decoding
- Returns tuple: (Crypto instance, KeySource::EnvironmentVariable)
- Validates decoded length is exactly 32 bytes
- Error on missing/empty/invalid base64/wrong length
- Uses zeroize on both success and error paths
- Enables flexible key management without modifying code
- Example usage:
  ```bash
  export VIBETEA_PRIVATE_KEY=$(vibetea-monitor export-key)
  # Monitor loads from env var on next run
  ```

**load_with_fallback() Method** (Phase 3 Addition):
- Implements key precedence: environment variable first, then file
- If `VIBETEA_PRIVATE_KEY` is set, loads from it with NO fallback on error
- If env var not set, loads from `{dir}/key.priv` file
- Returns tuple: (Crypto instance, KeySource indicating source)
- Enables flexible key management without code changes
- Error handling: env var errors are terminal (no fallback)
- Useful for deployment workflows with different key sources

**seed_base64() Method** (Phase 3 Addition):
- Exports private key as base64-encoded string
- Inverse of load_from_env() for key migration workflows
- Suitable for storing in `VIBETEA_PRIVATE_KEY` environment variable
- Marked sensitive: handle with care, avoid logging
- Used for user-friendly key export workflows
- Example: `vibetea-monitor export-key` displays exportable key format

**CryptoError::EnvVar Variant** (Phase 3 Addition):
- New error variant for environment variable issues
- Returned when `VIBETEA_PRIVATE_KEY` is missing or empty
- Distinct from file-based key loading errors
- Enables precise error handling and logging

### Phase 4: Export-Key Command for GitHub Actions

**CLI Command Location**: `monitor/src/main.rs` (lines 101-109, 180-202)

**export-key Subcommand** (FR-003, FR-023, FR-026, FR-027, FR-028):
- **Command**: `vibetea-monitor export-key [--path <PATH>]`
- **Purpose**: Export private key for use in GitHub Actions secrets or other deployment systems
- **Implementation**: Loads key from disk via `Crypto::load()` (not environment variable)
- **Output**: Base64-encoded seed to stdout followed by exactly one newline
- **Diagnostics**: All error messages and logging go to stderr only
- **Exit Codes**:
  - 0 on success
  - 1 on configuration error (missing key, invalid path)
- **Features**:
  - Suitable for piping to clipboard tools (`pbpaste`, `xclip`)
  - Suitable for piping to secret management systems
  - No ANSI escape codes, no carriage returns
  - Clean output for automation and scripting
  - Optional `--path` argument for custom key directory

**run_export_key() Function** (lines 180-202):
- Accepts optional `path` parameter from `--path` flag
- Defaults to `get_key_directory()` if not provided
- Calls `Crypto::load()` to read from disk only
- Prints base64 seed to stdout with single trailing newline
- Errors printed to stderr with helpful context
- Exit code 1 if key file not found
- Example stderr message: "Error: No key found at /path/to/keys/key.priv"

**Usage Examples**:
```bash
# Export to environment variable for local testing
export VIBETEA_PRIVATE_KEY=$(vibetea-monitor export-key)

# Export to GitHub Actions secret
EXPORTED_KEY=$(vibetea-monitor export-key)
gh secret set VIBETEA_PRIVATE_KEY --body "$EXPORTED_KEY"

# Export from custom key directory
vibetea-monitor export-key --path ~/.keys/vibetea

# Pipe directly to file
vibetea-monitor export-key > private_key.txt
```

**Integration Test Suite** (`monitor/tests/key_export_test.rs` - 699 lines):

**Framework**:
- Uses `serial_test` crate with `#[serial]` attribute
- Ensures tests run with `--test-threads=1` to prevent env var interference
- **EnvGuard RAII pattern**: Saves/restores environment variables on drop for isolation

**Test Coverage** (13 tests total):

1. **Round-trip Tests** (FR-027, FR-028):
   - `roundtrip_generate_export_command_import_sign_verify`
     - Generate new key → Save → Export via command → Load from env → Sign message → Verify signature
     - Validates exported key can be loaded and used for cryptography
   - `roundtrip_export_command_signatures_are_identical`
     - Verifies Ed25519 determinism: same key produces identical signatures
     - Tests that exported key produces same signatures as original

2. **Output Format Tests** (FR-003):
   - `export_key_output_format_base64_with_single_newline`
     - Validates exact format: base64 seed + exactly one newline
     - No leading/trailing whitespace other than final newline
   - `export_key_output_is_valid_base64_32_bytes`
     - Decodes output as base64 and verifies 32-byte length
     - Ensures cryptographic validity of exported data

3. **Diagnostic Output Tests** (FR-023):
   - `export_key_diagnostics_go_to_stderr`
     - Confirms stdout contains only base64 characters
     - No diagnostic patterns in stdout (no labels, no prose)
   - `export_key_error_messages_go_to_stderr`
     - Verifies errors written to stderr, not stdout
     - Stdout empty on error, stderr contains error message

4. **Exit Code Tests** (FR-026):
   - `export_key_exit_code_success` - Returns 0 on success
   - `export_key_exit_code_missing_key_file` - Returns 1 for missing key.priv
   - `export_key_exit_code_nonexistent_path` - Returns 1 for non-existent directory

5. **Edge Case Tests**:
   - `export_key_handles_path_with_spaces` - Paths with spaces handled correctly
   - `export_key_suitable_for_piping` - No ANSI codes, no carriage returns for clean piping
   - `export_key_reads_from_key_priv_file` - Verifies correct file is read (key.priv)

**Test Infrastructure**:
- Uses `tempfile` crate for isolated test directories (no interference)
- Uses `Command::new()` to invoke vibetea-monitor binary
- Tests find compiled binary via `get_monitor_binary_path()`
- Uses `base64` crate for decoding verification
- Uses `ed25519_dalek::Verifier` for signature validation
- All tests marked with `#[test]` and `#[serial]` attributes
- Comprehensive error message assertions with stderr capture

**Requirements Addressed**:
- **FR-003**: Export-key command outputs base64 key with single newline (piping-friendly)
- **FR-023**: Diagnostics on stderr, key only on stdout (machine-readable)
- **FR-026**: Exit codes 0 (success), 1 (config/missing key error), 2 (runtime error)
- **FR-027**: Exported key can be loaded via `VIBETEA_PRIVATE_KEY` environment variable
- **FR-028**: Round-trip verified: generate → export → load → sign → verify

### Phase 6: Monitor Cryptographic Operations

**Module Location**: `monitor/src/crypto.rs` (438 lines)

**Crypto Module Features**:

1. **Keypair Generation**
   - `Crypto::generate()` creates new Ed25519 keypair
   - Uses OS cryptographically secure RNG via `rand` crate
   - Returns Crypto struct managing SigningKey

2. **Key Persistence**
   - `save(dir)` writes keypair to files
   - Private key: `key.priv` (raw 32-byte seed, permissions 0600)
   - Public key: `key.pub` (base64-encoded, permissions 0644)
   - Creates directory if not present
   - Error on invalid file permissions (Unix)

3. **Key Loading**
   - `load(dir)` reads existing keypair
   - Validates private key is exactly 32 bytes
   - Returns CryptoError if format invalid
   - Reconstructs SigningKey from seed bytes

4. **Key Existence Check**
   - `exists(dir)` checks if private key file present
   - Used to prevent accidental overwrite

5. **Public Key Export**
   - `public_key_base64()` returns base64-encoded public key
   - Format suitable for `VIBETEA_PUBLIC_KEYS` environment variable
   - Derived from SigningKey via VerifyingKey

6. **Event Signing**
   - `sign(message)` returns base64-encoded Ed25519 signature
   - Message is JSON-encoded event payload (bytes)
   - Signature verifiable by server with public key
   - Uses RFC 8032 compliant signing via ed25519-dalek

**CryptoError Types**:
- `Io` - File system errors
- `InvalidKey` - Seed not 32 bytes or malformed
- `Base64` - Public key decoding error
- `KeyExists` - Files already present (can be overwritten)
- `EnvVar` - Environment variable missing or empty (Phase 3)

**File Locations** (configurable):
- Default key directory: `~/.vibetea/`
- Override with `VIBETEA_KEY_PATH` environment variable
- Private key: `{key_dir}/key.priv`
- Public key: `{key_dir}/key.pub`

**Key Loading Workflow** (Phase 3):
```
Priority 1: Check VIBETEA_PRIVATE_KEY env var
  - If set and valid: Use it
  - If set but invalid: Error (no fallback)
Priority 2: Load from {VIBETEA_KEY_PATH}/key.priv
  - If exists and valid: Use it
  - If missing or invalid: Error
```

### Monitor → Server Authentication

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Ed25519 digital signatures | Rust `ed25519-dalek` crate |
| **Protocol** | HTTPS POST with signed payload | Event signatures in X-Signature header |
| **Key Management** | Source-specific public key registration | `VIBETEA_PUBLIC_KEYS` env var |
| **Key Format** | Base64-encoded Ed25519 public keys | `source1:pubkey1,source2:pubkey2` |
| **Verification** | Constant-time comparison using `subtle` crate | `server/src/auth.rs` |
| **Flow** | Monitor signs event → Server validates signature | `server/src/auth.rs`, `server/src/routes.rs` |
| **Fallback** | Unsafe no-auth mode (dev only) | `VIBETEA_UNSAFE_NO_AUTH=true` |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_PUBLIC_KEYS` (required unless unsafe mode enabled)
- Parses `VIBETEA_UNSAFE_NO_AUTH` (dev-only authentication bypass)
- Validates on every server startup with comprehensive error messages
- Supports multiple comma-separated source:key pairs

**Example Key Format**:
```
VIBETEA_PUBLIC_KEYS=monitor-prod:dGVzdHB1YmtleTEx,monitor-dev:dGVzdHB1YmtleTIy
```

**Implementation Details**:
- Uses `HashMap<String, String>` to map source_id to base64-encoded keys
- Public keys stored in plain text (no decryption needed)
- Empty public_keys map allowed if unsafe_no_auth is enabled
- Error handling with ConfigError enum for missing/invalid formats
- Constant-time comparison prevents timing attacks on signature verification

**Signature Verification Process** (`server/src/auth.rs`):
- Decode base64 signature from X-Signature header
- Decode base64 public key from configuration
- Extract Ed25519 VerifyingKey from public key bytes
- Use `ed25519_dalek::Signature::verify()` for verification
- Apply `subtle::ConstantTimeEq` to compare results

### Client Authentication (Server → Client)

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Bearer token in WebSocket headers | Static token per deployment |
| **Protocol** | WebSocket upgrade with `Authorization: Bearer <token>` | Client sends on connect |
| **Token Type** | Opaque string (no expiration in Phase 4) | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| **Scope** | All clients use the same token | No per-user differentiation |
| **Validation** | Server-side validation only | In-memory, no persistence |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_SUBSCRIBER_TOKEN` (required unless unsafe mode enabled)
- Token required for all WebSocket connections
- No token refresh mechanism in Phase 5
- Stored as `Option<String>` in Config struct

**Future Enhancements**: Per-user tokens, token expiration, refresh tokens

## HTTP Sender & Event Transmission

### Phase 6: Event Sender Module

**Module Location**: `monitor/src/sender.rs` (544 lines)

**Sender Features**:

1. **HTTP Client Configuration**
   - Built with `reqwest` Client
   - Connection pooling: 10 max idle connections per host
   - Request timeout: 30 seconds
   - Automatic redirect handling

2. **Event Buffering**
   - VecDeque-based buffer with FIFO eviction
   - Default capacity: 1000 events
   - Configurable via `buffer_size` parameter
   - Tracks buffer overflow events with warnings
   - Supports queuing before sending

3. **Exponential Backoff Retry**
   - Initial delay: 1 second
   - Maximum delay: 60 seconds
   - Jitter: ±25% per attempt
   - Max retry attempts: 10 per batch
   - Resets on successful send

4. **Rate Limit Handling**
   - Recognizes HTTP 429 (Too Many Requests)
   - Reads `Retry-After` header from server
   - Respects server-provided delay
   - Falls back to exponential backoff if no header

5. **Event Signing**
   - Signs JSON event payload with Ed25519
   - X-Signature header contains base64-encoded signature
   - X-Source-ID header contains monitor source identifier
   - Compatible with server `auth.rs` verification

6. **Batch Sending**
   - `send_batch()` for efficient transmission
   - Single HTTP request with event array or single event
   - JSON request body with event(s)
   - 202 Accepted response expected

7. **Buffer Management**
   - `queue(event)` - Add to buffer
   - `flush()` - Send all buffered events
   - `send(event)` - Send single event immediately
   - `buffer_len()` - Current buffer size
   - `is_empty()` - Check if buffer empty

8. **Graceful Shutdown**
   - `shutdown(timeout)` - Flushes remaining events
   - Returns count of unflushed events
   - Waits for timeout before giving up
   - Allows time for final retry attempts

**SenderConfig**:
```rust
pub struct SenderConfig {
    pub server_url: String,     // e.g., https://vibetea.fly.dev
    pub source_id: String,      // e.g., hostname
    pub buffer_size: usize,     // e.g., 1000
}
```

**SenderError Types**:
- `Http` - HTTP client error (network, TLS, etc.)
- `ServerError { status, message }` - Non-202 response
- `AuthFailed` - 401 Unauthorized (invalid signature)
- `RateLimited { retry_after_secs }` - 429 with delay
- `BufferOverflow { evicted_count }` - Events evicted
- `MaxRetriesExceeded { attempts }` - All retries failed
- `Json` - Event serialization error

**Connection Details**:
- Server URL from `VIBETEA_SERVER_URL` env var
- POST to `{server_url}/events` endpoint
- HTTPS recommended for production
- HTTP allowed for local development

## GitHub Actions Integration

### Phase 5: Workflow File

**Location**: `.github/workflows/ci-with-monitor.yml` (114 lines)

### Phase 6: Composite Action File

**Location**: `.github/actions/vibetea-monitor/action.yml` (167 lines)

### Features

**Monitor Binary Download**:
- Fetches pre-built monitor from GitHub releases
- Target: x86_64-unknown-linux-gnu (Linux x86_64)
- URL pattern: `https://github.com/aaronbassett/VibeTea/releases/latest/download/vibetea-monitor-x86_64-unknown-linux-gnu`
- Graceful fallback: Continues if download fails (with warning)
- Exit code validation: Checks for successful execution
- Version control: Supports pinning specific versions

**Background Execution**:
- Starts monitor daemon: `./vibetea-monitor run &`
- Executes before main CI jobs (formatting, linting, tests, builds)
- Captures PID: `MONITOR_PID=$!` for later termination
- Non-blocking: Doesn't halt workflow on start failures

**Environment Setup**:
- **VIBETEA_PRIVATE_KEY**: From GitHub Actions secret
  - Base64-encoded 32-byte Ed25519 seed
  - Generated via `vibetea-monitor export-key`
  - Securely stored in repository secrets
- **VIBETEA_SERVER_URL**: From GitHub Actions secret
  - Server endpoint (e.g., `https://vibetea.fly.dev`)
  - Must be running and accessible
- **VIBETEA_SOURCE_ID**: Custom format for traceability
  - Format: `github-{owner}/{repo}-{run_id}`
  - Example: `github-aaronbassett/VibeTea-12345678`
  - Enables filtering events by workflow run in dashboards

**Graceful Shutdown**:
- Condition: `if: always()` (runs even if previous steps fail)
- Signal: `kill -TERM $MONITOR_PID`
- Grace period: 2-second flush window (`sleep 2`)
- Flushes buffered events before termination
- Prevents event loss on workflow completion

**Non-Blocking Behavior**:
- Network failures: Don't fail workflow
- Monitor startup failures: Don't fail workflow
- HTTP errors: Monitor retries with exponential backoff
- Rate limiting: Monitor respects Retry-After header

**CI Integration**:
- Runs alongside standard Rust/TypeScript checks
- Monitors active during: formatting, linting, tests, builds
- Events captured: All Claude Code activity during CI
- Example use cases: Track code generation, tool usage, agent decisions

**Binary Caching**:
- Uses GitHub Actions cache for cargo registry and dependencies
- Cache keys: `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
- Fallback keys: `${{ runner.os }}-cargo-`
- Reduces build times on subsequent runs

### Composite Action Inputs

- `server-url` (required): VibeTea server URL
- `private-key` (required): Base64-encoded Ed25519 private key
- `source-id` (optional): Custom source identifier (defaults to `github-<repo>-<run_id>`)
- `version` (optional): Monitor version to download (default: `latest`)
- `shutdown-timeout` (optional): Seconds to wait for graceful shutdown (default: `5`)

### Composite Action Outputs

- `monitor-pid`: Process ID of running monitor
- `monitor-started`: Boolean indicating successful startup

### Configuration

**Required Secrets** (set in repository settings):

1. **VIBETEA_PRIVATE_KEY**
   ```bash
   # Generate on local machine
   vibetea-monitor init         # If needed
   vibetea-monitor export-key   # Outputs base64-encoded key

   # Store in GitHub:
   # Settings → Secrets and variables → Actions → New repository secret
   # Name: VIBETEA_PRIVATE_KEY
   # Value: <paste output from export-key>
   ```

2. **VIBETEA_SERVER_URL**
   ```bash
   # Set to your running VibeTea server
   # Examples:
   # - https://vibetea.fly.dev
   # - https://your-domain.example.com
   # - http://localhost:3000 (not recommended for public workflows)
   ```

**Monitor Configuration**:
- Public key registration: Server must have corresponding public key registered
  - Formula: Export public key from local machine (`cat ~/.vibetea/key.pub`)
  - Register with server: `VIBETEA_PUBLIC_KEYS="github-{owner}/{repo}:{public_key}"`
  - Pattern: Source ID prefix matches monitor source ID in workflow

**Source ID Tracking**:
- Custom format enables filtering events by workflow run
- Dashboard can filter: `WHERE source = "github-aaronbassett/VibeTea-12345678"`
- Useful for correlating events with specific CI runs
- Prevents mixing events from multiple workflows

### Workflow Steps

1. **Checkout code**
   - Action: `actions/checkout@v4`
   - Prepares repository for CI jobs

2. **Download monitor binary**
   - Downloads pre-built monitor from releases
   - Sets execute permission if successful
   - Logs warning if download fails (graceful degradation)

3. **Start VibeTea Monitor**
   - Checks for binary and secrets
   - Starts background daemon
   - Captures PID for later shutdown
   - Logs source ID for tracking

4. **Install Rust toolchain**
   - Action: `dtolnay/rust-toolchain@stable`
   - Includes rustfmt and clippy

5. **Setup cache**
   - Caches cargo registry, git, and build artifacts
   - Improves subsequent build times

6. **Check formatting**
   - Command: `cargo fmt --all -- --check`
   - Validates code formatting

7. **Run clippy**
   - Command: `cargo clippy --all-targets -- -D warnings`
   - Lints code with clippy

8. **Run tests**
   - Command: `cargo test --workspace -- --test-threads=1`
   - Executes all tests sequentially (for env var isolation)

9. **Build release**
   - Command: `cargo build --workspace --release`
   - Produces optimized binaries

10. **Stop VibeTea Monitor**
    - Sends SIGTERM to background process
    - Allows 2-second grace period for event flushing
    - Runs even if previous steps failed

### Event Tracking During CI

**What Gets Captured**:
- Claude Code session events during workflow execution
- Tool usage: Read, Write, Grep, Bash, etc.
- Activity events: User interactions
- Agent state changes
- Session start/end markers

**What Doesn't Get Sent**:
- Code content (privacy filtered)
- Prompts or responses (privacy filtered)
- Full file paths (reduced to basenames)
- Sensitive command arguments (stripped)

**Dashboard Integration**:
- Filter by source: `github-{owner}/{repo}-{run_id}`
- Correlate with GitHub Actions run
- Track tool usage across CI jobs
- Analyze patterns in automated sessions

### Example Workflow Configuration

```yaml
# In GitHub repository settings:
# Secrets → VIBETEA_PRIVATE_KEY
# Value: base64-encoded key from vibetea-monitor export-key

# Secrets → VIBETEA_SERVER_URL
# Value: https://your-vibetea-server.example.com

# On server, register public key:
# export VIBETEA_PUBLIC_KEYS="github-owner/repo:$(cat ~/.vibetea/key.pub)"
```

## CLI & Key Management

### Phase 6: Monitor CLI

**Module Location**: `monitor/src/main.rs` (301 lines, expanded to 566 lines in Phase 4)

**Command Structure**:

1. **init Command**: Generate Ed25519 keypair
   ```bash
   vibetea-monitor init [--force]
   ```
   - Generates new keypair using `Crypto::generate()`
   - Saves to `~/.vibetea/` or `VIBETEA_KEY_PATH`
   - Displays public key for server registration
   - Prompts for overwrite confirmation (unless --force)
   - Provides copy-paste ready export command

2. **run Command**: Start monitor daemon
   ```bash
   vibetea-monitor run
   ```
   - Loads configuration from environment variables
   - Loads cryptographic keys from disk or env var (Phase 3)
   - Creates sender with buffering and retry
   - Initializes file watcher (future: Phase 7)
   - Waits for shutdown signal
   - Graceful shutdown with event flushing

3. **export-key Command**: Export private key (Phase 4)
   ```bash
   vibetea-monitor export-key [--path <PATH>]
   ```
   - Loads private key from disk
   - Outputs base64-encoded seed to stdout (+ single newline)
   - All diagnostics to stderr
   - Exit code 0 on success, 1 on error
   - Suitable for piping to clipboard or secret management tools

4. **help Command**: Show documentation
   ```bash
   vibetea-monitor help
   vibetea-monitor --help
   vibetea-monitor -h
   ```
   - Displays usage information
   - Lists all available commands
   - Shows environment variables
   - Provides example commands

5. **version Command**: Show version
   ```bash
   vibetea-monitor version
   vibetea-monitor --version
   vibetea-monitor -V
   ```
   - Prints binary version from Cargo.toml

**CLI Framework** (Phase 4):
- Uses `clap` crate with Subcommand and Parser derive macros
- Type-safe command parsing with automatic help generation
- Replaces manual argument parsing from Phase 6
- Command enum variants: Init, ExportKey, Run
- Flag support: `--force/-f` for init, `--path` for export-key

**Environment Variables Used**:

| Variable | Required | Default | Command |
|----------|----------|---------|---------|
| `VIBETEA_SERVER_URL` | Yes | - | run |
| `VIBETEA_SOURCE_ID` | No | hostname | run |
| `VIBETEA_PRIVATE_KEY` | No* | - | run (Phase 3 - loads from env) |
| `VIBETEA_KEY_PATH` | No | ~/.vibetea | init, run, export-key |
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | run |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | run |
| `VIBETEA_BASENAME_ALLOWLIST` | No | - | run |
| `RUST_LOG` | No | info | all |

*Either VIBETEA_PRIVATE_KEY (env) or VIBETEA_KEY_PATH/key.priv (file) required

**Logging**:
- Structured logging via `tracing` crate
- Environment-based filtering (`RUST_LOG`)
- JSON output support
- Logs configuration, key loading, shutdown events
- Info level by default

**Signal Handling**:
- Listens for SIGINT (Ctrl+C)
- Listens for SIGTERM on Unix
- Cross-platform support via `tokio::signal`
- Graceful shutdown sequence on signal

**Key Registration Workflow**:
1. User runs: `vibetea-monitor init`
2. Binary displays public key
3. User copies to: `export VIBETEA_PUBLIC_KEYS="...:<public_key>"`
4. User adds to server configuration
5. User runs: `vibetea-monitor run`

**Phase 3 Key Loading Workflow**:
```bash
# Option 1: Use environment variable (new in Phase 3)
export VIBETEA_PRIVATE_KEY=$(vibetea-monitor export-key)
vibetea-monitor run

# Option 2: Use file (Phase 2)
vibetea-monitor init
vibetea-monitor run

# Option 3: Fallback behavior (both checked in order)
export VIBETEA_PRIVATE_KEY=...  # Checked first
# If not set, falls back to ~/.vibetea/key.priv
vibetea-monitor run
```

**Phase 4 GitHub Actions Workflow**:
```bash
# Export key from development machine
exported_key=$(vibetea-monitor export-key)

# Register in GitHub Actions secret
gh secret set VIBETEA_PRIVATE_KEY --body "$exported_key"

# Use in workflow
- name: Export monitor key
  env:
    VIBETEA_PRIVATE_KEY: ${{ secrets.VIBETEA_PRIVATE_KEY }}
  run: vibetea-monitor run
```

## Client-Side Integrations (Phase 7-10)

### Browser WebSocket Connection

**Module Location**: `client/src/hooks/useWebSocket.ts` (321 lines)

**WebSocket Hook Features**:

1. **Connection Management**
   - Establishes WebSocket connection to server
   - Validates token from localStorage before connecting
   - Tracks connection state: connecting, connected, reconnecting, disconnected
   - Provides manual `connect()` and `disconnect()` methods

2. **Auto-Reconnection**
   - Exponential backoff: 1s initial, 60s maximum
   - Jitter: ±25% randomization per attempt
   - Resets attempt counter on successful connection
   - Respects user's disconnect intent (no auto-reconnect after manual disconnect)

3. **Token Management**
   - Reads token from `localStorage` key: `vibetea_token`
   - Token set via TokenForm component
   - Returns error if token missing, prevents connection
   - Token passed as query parameter in WebSocket URL

4. **Event Processing**
   - Receives JSON-encoded VibeteaEvent messages
   - Validates message structure (id, source, timestamp, type, payload)
   - Dispatches valid events to Zustand store via `addEvent()`
   - Silently discards invalid/unparseable messages

5. **Integration with Event Store**
   - `useEventStore` for state management
   - `addEvent(event)` - Add event to store
   - `setStatus(status)` - Update connection status
   - Status field synced with component state

6. **Error Handling**
   - Logs connection errors to console
   - Logs message parsing failures
   - Graceful handling of malformed messages
   - No crashes on connection errors

7. **Cleanup & Lifecycle**
   - Proper cleanup on component unmount
   - Clears pending reconnection timeouts
   - Closes WebSocket connection
   - Prevents memory leaks

**Hook Return Type**:
```typescript
export interface UseWebSocketReturn {
  readonly connect: () => void;          // Manually initiate connection
  readonly disconnect: () => void;        // Manually disconnect
  readonly isConnected: boolean;          // Connection state
}
```

**Constants**:
- `TOKEN_STORAGE_KEY`: `"vibetea_token"` (matches TokenForm)
- `INITIAL_BACKOFF_MS`: 1000ms
- `MAX_BACKOFF_MS`: 60000ms
- `JITTER_FACTOR`: 0.25 (25%)

**Default WebSocket URL**:
- Protocol: `ws://` (HTTP) or `wss://` (HTTPS) based on location protocol
- Host: Current browser location host
- Path: `/ws`
- Query param: `token=<token_from_localStorage>`

### Connection Status Component

**Module Location**: `client/src/components/ConnectionStatus.tsx` (106 lines)

**Features**:

1. **Visual Indicator**
   - Colored dot (2.5x2.5 rem) showing connection state
   - Green (#22c55e) for connected
   - Yellow (#eab308) for connecting/reconnecting
   - Red (#ef4444) for disconnected
   - Uses Tailwind CSS classes

2. **Optional Status Label**
   - Shows text status if `showLabel` prop is true
   - Labels: "Connected", "Connecting", "Reconnecting", "Disconnected"
   - Styled as small gray text
   - Dark mode support

3. **Performance Optimization**
   - Selective Zustand subscription: only re-renders when status changes
   - Uses selector to extract only status field
   - Prevents re-renders on other store updates

4. **Accessibility**
   - `role="status"` for semantic meaning
   - `aria-label` with full status description
   - Visual indicator marked as `aria-hidden="true"`
   - Screen reader friendly

5. **Component Props**:
```typescript
interface ConnectionStatusProps {
  readonly showLabel?: boolean;    // Show status text (default: false)
  readonly className?: string;     // Additional CSS classes
}
```

6. **Styling**
   - Flexbox layout with gap-2
   - Responsive and composable
   - Integrates seamlessly with other UI elements
   - Dark mode aware styling

### Token Form Component

**Module Location**: `client/src/components/TokenForm.tsx` (201 lines)

**Features**:

1. **Token Input & Storage**
   - Password input field for secure token entry
   - Persists token to `localStorage` via `TOKEN_STORAGE_KEY`
   - Matches key used by `useWebSocket` hook
   - Non-empty validation before saving

2. **Button Controls**
   - **Save Token** button
     - Disabled when input is empty
     - Saves trimmed token to localStorage
     - Resets input field after save
     - Invokes optional callback
   - **Clear Token** button
     - Disabled when no token saved
     - Removes token from localStorage
     - Resets input and status
     - Invokes optional callback

3. **Status Indicator**
   - Green dot when token saved
   - Gray dot when no token saved
   - Text shows "Token saved" or "No token saved"
   - Updates in real-time as user changes

4. **Cross-Window Synchronization**
   - Listens to `storage` events
   - Detects token changes from other tabs/windows
   - Updates status accordingly
   - Handles multi-tab scenarios

5. **Component Props**:
```typescript
interface TokenFormProps {
  readonly onTokenChange?: () => void;  // Called when token saved/cleared
  readonly className?: string;          // Additional CSS classes
}
```

6. **Callback Support**
   - `onTokenChange()` invoked on save or clear
   - Allows parent to reconnect WebSocket
   - Enables form submission handlers

7. **Accessibility**
   - Label element linked to input
   - `aria-describedby` for status association
   - Status region with `aria-live="polite"`
   - Semantic form structure
   - Proper button states for disabled

8. **Styling**
   - Tailwind CSS dark mode (bg-gray-800, text-white)
   - Responsive layout
   - Visual feedback on focus (blue ring)
   - Disabled state styling (gray background, cursor not-allowed)
   - Button hover effects

9. **Behavior**
   - Stores token under key `vibetea_token` (matches useWebSocket)
   - Input placeholder changes based on save state
   - Form submission on button click or Enter key
   - Input cleared after successful save
   - Token masked as password field

## Network Communication

### Monitor → Server (Event Publishing)

**Method**: Ed25519 digital signatures
**Protocol**: HTTPS POST with signed payload
**Key Management**: Source-specific public key registration
**Verification**: Constant-time comparison using `subtle` crate

**Configuration Location**: `server/src/config.rs`
- `VIBETEA_PUBLIC_KEYS` - Format: `source1:pubkey1,source2:pubkey2`
- `VIBETEA_UNSAFE_NO_AUTH` - Dev-only authentication bypass

**Signature Verification Process** (`server/src/auth.rs`):
1. Decode base64 signature from X-Signature header
2. Decode base64 public key from configuration
3. Extract Ed25519 VerifyingKey from public key bytes
4. Verify signature with RFC 8032 compliance
5. Apply constant-time comparison to prevent timing attacks

**Cryptographic Details**:
- Algorithm: Ed25519 (ECDSA variant)
- Library: `ed25519-dalek` crate (version 2.1)
- Key generation: 32-byte seed via OS RNG
- File permissions: 0600 (private key only)
- Public key format: Base64-encoded

### Server → Client (Event Streaming)

**Method**: Bearer token in WebSocket headers
**Protocol**: WebSocket upgrade with `Authorization: Bearer <token>`
**Token Type**: Opaque string (no expiration in Phase 5)
**Scope**: All clients use the same token (global scope)

**Configuration Location**: `server/src/config.rs`
- `VIBETEA_SUBSCRIBER_TOKEN` - Required unless unsafe mode
- Validated on WebSocket upgrade
- No token refresh mechanism in Phase 5

**Validation**: Server-side validation only (in-memory)

**Future Enhancements**: Per-user tokens, token expiration, refresh tokens

## External APIs

There are no external third-party API integrations in Phase 11. The system is self-contained:
- All data sources are local files
- All services are internal (Monitor, Server, Client)
- No SaaS dependencies or external service calls

**Future Integration Points** (Not Yet Implemented):
- Cloud storage (S3, GCS) for event archive
- Monitoring services (Datadog, New Relic)
- Message queues (Redis, RabbitMQ)
- Webhooks for external notifications
- Database persistence (PostgreSQL, etc.)

## HTTP API Endpoints

### POST /events

**Purpose**: Ingest events from monitors

**Request Headers**:
| Header | Required | Value |
|--------|----------|-------|
| X-Source-ID | Yes | Monitor identifier (non-empty) |
| X-Signature | No* | Base64-encoded Ed25519 signature |
| Content-Type | Yes | application/json |

*Required unless `VIBETEA_UNSAFE_NO_AUTH=true`

**Request Body**: Single Event or array of Events (JSON)

**Response Codes**:
- 202 Accepted - Events accepted and broadcasted
- 400 Bad Request - Invalid JSON or malformed events
- 401 Unauthorized - Invalid X-Source-ID or signature mismatch
- 429 Too Many Requests - Rate limit exceeded (includes Retry-After header)

**Rate Limiting** (`server/src/rate_limit.rs`):
- Token bucket algorithm per source
- Default: 100 events/second per source
- Capacity: 100 tokens
- Exceeds limit: Returns 429 with Retry-After header
- Cleanup: Stale sources removed after 60 seconds idle

### GET /ws

**Purpose**: WebSocket subscription for event streaming

**Query Parameters**:
| Parameter | Required | Example |
|-----------|----------|---------|
| token | No* | my-secret-token |
| source | No | monitor-1 |
| type | No | session |
| project | No | my-project |

*Required unless `VIBETEA_UNSAFE_NO_AUTH=true`

**WebSocket Messages**: JSON-encoded Event objects (one per message)

**Response Codes**:
- 101 Switching Protocols - Upgrade successful
- 401 Unauthorized - Token validation failed

**Filtering** (`server/src/broadcast.rs`):
- Optional SubscriberFilter based on query parameters
- Matches event type, source, project
- Enables selective delivery

### GET /health

**Purpose**: Health check and uptime reporting

**Response**:
```json
{
  "status": "ok",
  "uptime_secs": 3600
}
```

**Response Code**: 200 OK (always succeeds, no auth)

## Network Communication

### Monitor → Server (Event Publishing)

**Endpoint**: `https://<server-url>/events`
**Method**: POST
**Content-Type**: application/json

**Flow**:
1. Monitor watches local JSONL files via file watcher
2. Parser extracts metadata from new/modified lines
3. Events processed through PrivacyPipeline
4. Monitor signs event payload with Ed25519 private key
5. Monitor POSTs signed event with X-Source-ID and X-Signature headers
6. Server validates signature against registered public key
7. Server rate limits based on source ID
8. Server broadcasts to all connected clients via WebSocket

**Client Library**: `reqwest` (HTTP client)

**Configuration** (`monitor/src/config.rs`):
- `VIBETEA_SERVER_URL` - Server endpoint (required)
- `VIBETEA_SOURCE_ID` - Monitor identifier (default: hostname)

**Sender Module** (`monitor/src/sender.rs`):
- HTTP client with connection pooling (10 max idle)
- Event buffering with FIFO eviction (1000 events default)
- Exponential backoff: 1s → 60s with ±25% jitter
- Rate limit handling: Respects 429 with Retry-After
- Timeout: 30 seconds per request

### Server → Client (Event Broadcasting)

**Protocol**: WebSocket (upgraded from HTTP)
**URL**: `ws://<server-url>/ws` (or `wss://` for HTTPS)
**Authentication**: Bearer token in upgrade request
**Message Format**: JSON (Event)

**Flow**:
1. Client initiates WebSocket with Bearer token
2. Server validates token and establishes connection
3. Server broadcasts events as they arrive
4. Optional filtering based on query parameters
5. Client processes events via Zustand store
6. Client UI renders session information

**Broadcasting** (`server/src/broadcast.rs`):
- EventBroadcaster wraps tokio broadcast channel
- 1000-event capacity for burst handling
- Thread-safe, cloneable across handlers
- SubscriberFilter enables selective delivery

**Client-Side Handling**:
- WebSocket proxy configured in `client/vite.config.ts`
- State management via `useEventStore` hook (Zustand)
- Event type guards in `client/src/types/events.ts`
- ConnectionStatus component for visual feedback
- useWebSocket hook with auto-reconnect

### Monitor → File System (JSONL, Todo, Stats, & Projects Watching)

**Targets**:
- `~/.claude/projects/**/*.jsonl` - Session events
- `~/.claude/projects/` - Project directory scanning (Phase 11)
- `~/.claude/history.jsonl` - Skill invocations
- `~/.claude/todos/*.json` - Todo lists
- `~/.claude/stats-cache.json` - Token/session statistics

**Mechanism**: `notify` crate file system events
**Update Strategy**: Incremental line reading with position tracking (sessions/history), file debouncing (todos/stats), directory scanning with summary detection (projects)

**Session File Flow**:
1. FileWatcher initialized with watch directory
2. Recursive file system monitoring begins
3. File creation detected → WatchEvent::FileCreated
4. File modification detected → Read new lines from position
5. Lines accumulated → WatchEvent::LinesAdded
6. Position marker updated
7. File deletion detected → WatchEvent::FileRemoved

**Project Monitoring Flow** (Phase 11):
1. ProjectTracker initialized with projects directory
2. Recursive file system monitoring begins
3. File creation/modification detected
4. JSONL file read to check for summary events
5. Session activity status determined
6. ProjectActivityEvent emitted via mpsc channel
7. Event sent to server for broadcast

**Skill File Monitoring** (Phase 5):
1. SkillTracker initialized with history.jsonl path
2. Watcher monitors parent directory
3. Modification detected (data changes only)
4. New entries read from byte offset
5. Entries parsed → SkillInvocationEvent created
6. Event emitted via mpsc channel
7. Byte offset updated

**Todo File Monitoring** (Phase 6):
1. TodoTracker initialized with todos directory
2. Watcher monitors `~/.claude/todos/` non-recursively
3. File creation/modification detected
4. Filename validated: `<uuid>-agent-<uuid>.json`
5. File content read as JSON array
6. Entries parsed and counted (lenient)
7. Abandonment flag set based on session ended status
8. TodoProgressEvent emitted via mpsc channel
9. Changes debounced at 100ms to coalesce rapid writes

**Stats File Monitoring** (Phase 8, Phase 10):
1. StatsTracker initialized with stats-cache.json path
2. Watcher monitors `~/.claude/` directory
3. File creation/modification detected
4. File content read as JSON object
5. SessionMetricsEvent created and emitted
6. ActivityPatternEvent created from hourCounts and emitted (Phase 10)
7. TokenUsageEvent created for each model
8. ModelDistributionEvent created from all models and emitted (Phase 10)
9. Events emitted via mpsc channel
10. Changes debounced at 200ms to coalesce rapid writes

**Efficiency Features**:
- Position tracking prevents re-reading (sessions/history)
- Only new lines since last position extracted
- BufReader with Seek for efficient iteration
- Arc<RwLock<>> for thread-safe concurrent access
- Atomic offset for lock-free reads in skill tracker
- Debouncing prevents duplicate processing (todos/stats)
- Summary event detection enables efficient completion tracking (projects)

## Development & Local Configuration

### Local Server Setup

**Environment Variables**:
```bash
PORT=8080                                       # Server port
VIBETEA_PUBLIC_KEYS=localhost:cHVia2V5MQ==     # Monitor public key
VIBETEA_SUBSCRIBER_TOKEN=dev-token-secret      # Client token
VIBETEA_UNSAFE_NO_AUTH=false                   # Auth mode
RUST_LOG=debug                                 # Logging level
```

**Unsafe Development Mode**:
When `VIBETEA_UNSAFE_NO_AUTH=true`:
- All monitor authentication bypassed
- All client authentication bypassed
- Suitable for local development only
- Never use in production
- Warning logged on startup

### Local Monitor Setup

**Environment Variables**:
```bash
VIBETEA_SERVER_URL=http://localhost:8080         # Server endpoint
VIBETEA_SOURCE_ID=my-monitor                     # Custom source identifier
VIBETEA_KEY_PATH=~/.vibetea                      # Directory with private/public keys
VIBETEA_PRIVATE_KEY=<base64-seed>                # Env var key loading (Phase 3)
VIBETEA_CLAUDE_DIR=~/.claude                     # Claude Code directory to watch
VIBETEA_BUFFER_SIZE=1000                         # Event buffer capacity
VIBETEA_BASENAME_ALLOWLIST=.ts,.tsx,.rs          # Optional file extension filter (Phase 5)
RUST_LOG=debug                                   # Logging level
```

**Configuration Loading**: `monitor/src/config.rs`
- Required: VIBETEA_SERVER_URL (no default)
- Optional defaults use directories crate for platform-specific paths
- Home directory determined via BaseDirs::new()
- Hostname fallback when VIBETEA_SOURCE_ID not set
- Buffer size parsed as usize, validated for positive integers
- Allowlist split by comma, whitespace trimmed, empty entries filtered

**Key Management** (Phase 3):
- `vibetea-monitor init` generates Ed25519 keypair
- `vibetea-monitor export-key` exports private key as base64 (Phase 4 feature)
- Keys stored in ~/.vibetea/ or VIBETEA_KEY_PATH
- Private key: key.priv (0600 permissions)
- Public key: key.pub (0644 permissions)
- Public key must be registered with server via VIBETEA_PUBLIC_KEYS
- Private key can be loaded from VIBETEA_PRIVATE_KEY env var (Phase 3)

**Privacy Configuration** (Phase 5):
- `VIBETEA_BASENAME_ALLOWLIST` loads into PrivacyConfig via `from_env()`
- Format: `.rs,.ts,.md` or `rs,ts,md` (dots auto-added)
- Whitespace tolerance: ` .rs , .ts ` → `[".rs", ".ts"]`
- Empty entries filtered: `.rs,,.ts,,,` → `[".rs", ".ts"]`
- When not set: All extensions allowed (default behavior)
- Applied during PrivacyPipeline event processing

**File System Monitoring**:
- Watches directory: VIBETEA_CLAUDE_DIR
- Monitors for file creation, modification, deletion, and directory changes
- Uses `notify` crate (version 8.0) for cross-platform inotify/FSEvents
- Optional extension filtering via VIBETEA_BASENAME_ALLOWLIST
- Phase 4: FileWatcher tracks position to efficiently tail JSONL files
- Phase 11: ProjectTracker scans project directory for session activity

**JSONL Parsing**:
- Phase 4: SessionParser extracts metadata from Claude Code JSONL
- Privacy-first: Never processes code content or prompts
- Tool tracking: Extracts tool name and context from assistant tool_use events
- Progress tracking: Detects tool completion from progress PostToolUse events

**Privacy Pipeline** (Phase 5):
- PrivacyPipeline processes all events before transmission
- PrivacyConfig loaded from `VIBETEA_BASENAME_ALLOWLIST`
- Sensitive tools stripped: Bash, Grep, Glob, WebSearch, WebFetch
- Paths reduced to basenames with extension allowlist filtering
- Summary text neutralized to "Session ended"

**Cryptographic Signing** (Phase 6):
- Crypto module signs all events with Ed25519 private key
- Signature sent in X-Signature header (base64-encoded)
- Monitor must be initialized before first run: `vibetea-monitor init`

**HTTP Transmission** (Phase 6):
- Sender module handles event buffering (1000 events default)
- Exponential backoff retry: 1s → 60s with ±25% jitter
- Rate limit handling: Respects 429 with Retry-After header
- Connection pooling: 10 max idle connections per host
- 30-second request timeout

### GitHub Actions Setup (Phase 5)

**Prerequisites**:
1. A running VibeTea server with your public key registered
2. An existing keypair on your local machine (run `vibetea-monitor init` if needed)

**Step 1: Export Your Private Key**
```bash
# Export to clipboard (macOS)
vibetea-monitor export-key | pbcopy

# Export to stdout (Linux/Windows)
vibetea-monitor export-key
```

**Step 2: Register GitHub Actions Secret**
```bash
# Using GitHub CLI
gh secret set VIBETEA_PRIVATE_KEY --body "$(vibetea-monitor export-key)"

# Or manually in GitHub web interface:
# Settings → Secrets and variables → Actions → New repository secret
# Name: VIBETEA_PRIVATE_KEY
# Value: <paste output from export-key command>
```

**Step 3: Register Server URL Secret**
```bash
# Using GitHub CLI
gh secret set VIBETEA_SERVER_URL --body "https://your-vibetea-server.example.com"

# Or manually in GitHub web interface:
# Settings → Secrets and variables → Actions → New repository secret
# Name: VIBETEA_SERVER_URL
# Value: https://your-vibetea-server.example.com
```

**Step 4: Register Public Key with Server**
```bash
# Export public key from local machine
cat ~/.vibetea/key.pub

# On server, register with source ID pattern:
export VIBETEA_PUBLIC_KEYS="github-{owner}/{repo}:$(cat ~/.vibetea/key.pub)"

# Example:
export VIBETEA_PUBLIC_KEYS="github-aaronbassett/VibeTea:dGVzdHB1YmtleTExYWJjZGVmZ2hpams="
```

**Step 5: Add Workflow File**
- Copy `.github/workflows/ci-with-monitor.yml` example
- Customize for your repository and CI needs
- Commit and push to main branch
- Workflow will run on next push/PR

## Configuration Quick Reference

### Server Environment Variables

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `PORT` | number | 8080 | No | HTTP server listening port |
| `VIBETEA_PUBLIC_KEYS` | string | - | Yes* | Source public keys (source:key,source:key) |
| `VIBETEA_SUBSCRIBER_TOKEN` | string | - | Yes* | Bearer token for clients |
| `VIBETEA_UNSAFE_NO_AUTH` | boolean | false | No | Disable all authentication (dev only) |
| `RUST_LOG` | string | info | No | Logging level (debug, info, warn, error) |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true`

### Monitor Environment Variables

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `VIBETEA_SERVER_URL` | string | - | Yes | Server endpoint (e.g., https://vibetea.fly.dev) |
| `VIBETEA_SOURCE_ID` | string | hostname | No | Monitor identifier |
| `VIBETEA_PRIVATE_KEY` | string | - | No* | Base64-encoded private key (Phase 3) |
| `VIBETEA_KEY_PATH` | string | ~/.vibetea | No | Directory with key.priv/key.pub |
| `VIBETEA_CLAUDE_DIR` | string | ~/.claude | No | Claude Code directory to watch |
| `VIBETEA_BUFFER_SIZE` | number | 1000 | No | Event buffer capacity |
| `VIBETEA_BASENAME_ALLOWLIST` | string | - | No | Comma-separated file extensions to watch (Phase 5) |
| `RUST_LOG` | string | info | No | Logging level (debug, info, warn, error) |

*Either VIBETEA_PRIVATE_KEY (env) or VIBETEA_KEY_PATH/key.priv (file) required

### Client localStorage Keys (Phase 7)

| Key | Purpose | Format |
|-----|---------|--------|
| `vibetea_token` | WebSocket authentication token | String |

## Future Integration Points

### Planned (Not Yet Integrated)

- **Main event loop**: Integrate file watcher, parser, privacy pipeline, and HTTP sender (Phase 6 in progress)
- **Database/Persistence**: Store events beyond memory (Phase 5+)
- **Authentication Providers**: OAuth2, API key rotation (Phase 5+)
- **Monitoring Services**: Datadog, New Relic, CloudWatch (Phase 5+)
- **Message Queues**: Redis, RabbitMQ for event buffering (Phase 5+)
- **Webhooks**: External service notifications (Phase 6+)
- **Background Task Spawning**: Async watcher and sender pipeline (Phase 6+)
- **Session Persistence**: Store events in database for replay (Phase 7+)
- **Advanced Authentication**: Per-user tokens, OAuth2 flows (Phase 7+)
- **Event Search/Filtering**: Full-text search and advanced filtering UI (Phase 7+)
- **Performance Monitoring**: Client-side performance metrics (Phase 8+)
