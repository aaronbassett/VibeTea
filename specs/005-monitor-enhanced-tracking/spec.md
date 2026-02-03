# Feature Specification: Monitor Enhanced Data Tracking

**Feature Branch**: `005-monitor-enhanced-tracking`
**Created**: 2026-02-03
**Status**: Draft
**Input**: User description: "Enhanced monitor data tracking: Agent spawns from session JSONLs (subagent_type from Task tool input), Skill invocations from history.jsonl (lines starting with /), Token usage from stats-cache.json (by model), Session metrics from stats-cache.json (message/tool counts, duration), Activity patterns from stats-cache.json (hourCounts), Model distribution from stats-cache.json (opus/sonnet/haiku split), Todo progress from todos/*.json with abandoned tracking on session end via summary event correlation, File edit history from file-history/ directory (diff vN-1 to vN for N>=2, skip v1), Project activity from projects/ directory structure. All tracked per-session, client aggregates as needed. Existing tool usage tracking remains unchanged."

**Codebase Documentation**: See [.sdd/codebase/](.sdd/codebase/) for technical details

## Clarifications

### Session 2026-02-03

- Q: What do SessionMetricsEvent fields represent (per-session snapshots, global aggregates, or deltas)? → A: Global aggregates - fields are totals across all sessions from stats-cache.json
- Q: How should the monitor extract the command name from a skill invocation in the display field? → A: Capture everything after `/` until first unquoted whitespace (shell-like tokenization that respects quotes). E.g., `/commit -m "msg"` → `commit`, `/"foo bar" arg1` → `"foo bar"`
- Q: What is the primary signal for detecting session end to trigger todo abandonment? → A: Only summary event in JSONL triggers abandonment. File deletion just cleans up tracking state without marking tasks as abandoned.
- Q: Where should the monitor find Task tool invocations to extract subagent_type? → A: Session JSONL files - parse `tool_use` events where tool name is "Task" and extract `subagent_type` from the parameters object.
- Q: How should token usage be tracked given stats-cache.json is global? → A: Global aggregates per model - TokenUsageEvent has no session_id, emits totals per model from stats-cache.json (consistent with SessionMetricsEvent approach).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Token Usage by Model (Priority: P1)

As a developer using Claude Code, I want to see how many tokens I'm consuming broken down by model so that I can understand my usage patterns and optimize my workflows.

**Why this priority**: Token usage is the primary cost driver. Understanding consumption by model enables developers to identify expensive patterns and make informed decisions about their Claude Code usage.

**Independent Test**: Can be fully tested by starting a Claude Code session, performing some actions, and verifying the monitor displays global token counts (input, output, cache) broken down by model from stats-cache.json.

**Acceptance Scenarios**:

1. **Given** a Claude Code session is active, **When** the monitor receives stats-cache.json updates, **Then** it emits token usage events containing global totals for input tokens, output tokens, cache read tokens, and cache creation tokens per model.
2. **Given** multiple models are used (opus, sonnet, haiku), **When** viewing token data, **Then** the breakdown shows cumulative usage per model separately.
3. **Given** stats-cache.json is updated, **When** the monitor emits token events, **Then** each model's totals are available for client-side aggregation and display.

---

### User Story 2 - Track Agent Spawns (Priority: P1)

As a developer, I want to see when and which types of agents Claude spawns during my sessions so that I can understand how Claude is approaching my tasks and which specialized agents are being used.

**Why this priority**: Agent spawns indicate how Claude decomposes complex tasks. This visibility helps developers understand Claude's problem-solving approach and identify which agent types are most useful for their workflows.

**Independent Test**: Can be fully tested by asking Claude to perform a task that triggers agent spawns (e.g., "explore the codebase"), and verifying the monitor captures the subagent_type for each Task tool invocation.

**Acceptance Scenarios**:

1. **Given** Claude invokes the Task tool with a subagent_type, **When** the monitor parses the session JSONL, **Then** it emits an agent spawn event containing the session ID and subagent_type.
2. **Given** multiple agents are spawned in a session, **When** viewing agent data, **Then** all spawned agent types are listed with their counts.
3. **Given** an agent spawn event, **When** it is transmitted to the server, **Then** the event includes timestamp, session ID, and agent type.

---

### User Story 3 - Monitor Skill Invocations (Priority: P1)

As a developer, I want to see which slash commands (skills) I invoke so that I can track my usage patterns and identify my most-used workflows.

**Why this priority**: Skill invocations represent deliberate user actions and workflow choices. Tracking them provides insight into how developers interact with Claude Code and which features they rely on most.

**Independent Test**: Can be fully tested by invoking several slash commands (e.g., /commit, /sdd:plan), and verifying the monitor captures each invocation with timestamp and command name.

**Acceptance Scenarios**:

1. **Given** a user types a slash command in Claude Code, **When** the history.jsonl file is updated, **Then** the monitor emits a skill invocation event containing the command name and timestamp.
2. **Given** a skill invocation includes arguments, **When** the event is emitted, **Then** only the command name (not arguments) is captured for privacy.
3. **Given** multiple skill invocations across sessions, **When** viewing skill data, **Then** invocations are attributed to their respective sessions.

---

### User Story 4 - Track Todo Progress with Abandonment (Priority: P2)

As a developer, I want to see the progress of my task lists and know when tasks are abandoned due to session termination so that I can track my completion rates and identify unfinished work.

**Why this priority**: Todo tracking provides visibility into task completion. Abandoned task tracking helps identify patterns of incomplete work and sessions that ended prematurely.

**Independent Test**: Can be fully tested by creating tasks during a session, completing some, leaving others pending, then ending the session and verifying the monitor correctly categorizes tasks as completed, pending, or abandoned.

**Acceptance Scenarios**:

1. **Given** a todo file is created or modified, **When** the monitor detects the change, **Then** it emits a todo event with counts for completed, pending, and in_progress tasks for that session.
2. **Given** a session ends (summary event in JSONL), **When** the session has a matching todo file with incomplete tasks, **Then** those tasks are reclassified as abandoned.
3. **Given** abandoned tasks are tracked, **When** the todo event is updated, **Then** the abandoned count reflects the number of incomplete tasks from ended sessions.
4. **Given** a todo file exists but contains an empty array, **When** the monitor processes it, **Then** no event is emitted (or event shows zero counts).

---

### User Story 5 - Track File Edit Line Changes (Priority: P2)

As a developer, I want to see how many lines of code are being added and removed during my sessions so that I can understand the scope of changes Claude is making.

**Why this priority**: Line change metrics provide a quantitative measure of code modification activity. This helps developers understand session productivity and the magnitude of changes being made.

**Independent Test**: Can be fully tested by having Claude edit files during a session, and verifying the monitor calculates lines added/removed by diffing consecutive versions in file-history.

**Acceptance Scenarios**:

1. **Given** a file version v2 or higher appears in file-history, **When** the monitor detects it, **Then** it diffs the new version against the previous version (vN vs vN-1).
2. **Given** a diff is computed, **When** the results are processed, **Then** the monitor emits a file_change event with lines_added and lines_removed counts.
3. **Given** a v1 file appears (initial version), **When** the monitor processes it, **Then** no diff is performed and no line change event is emitted.
4. **Given** multiple files are edited in a session, **When** viewing line change data, **Then** totals are aggregated per session.

---

### User Story 6 - View Session Metrics (Priority: P2)

As a developer, I want to see aggregated global session metrics including message counts, tool call counts, and session duration so that I can understand my overall usage patterns.

**Why this priority**: Session metrics provide high-level visibility into Claude Code usage. This helps developers understand session characteristics and compare productivity across sessions.

**Independent Test**: Can be fully tested by running Claude Code sessions and verifying the monitor captures and displays global message counts, tool call counts, and session duration from stats-cache.json.

**Acceptance Scenarios**:

1. **Given** stats-cache.json is updated, **When** the monitor reads it, **Then** it extracts global totals for message count, tool call count, and session count across all sessions.
2. **Given** session duration is tracked, **When** a session metric event is emitted, **Then** it includes the longest session duration in milliseconds.
3. **Given** the longest session data is available, **When** viewing metrics, **Then** the longest session details (ID, duration, message count) are accessible.

---

### User Story 7 - View Activity Patterns by Hour (Priority: P3)

As a developer, I want to see which hours of the day I'm most active with Claude Code so that I can understand my productivity patterns.

**Why this priority**: Activity patterns are useful for self-reflection but not critical for core functionality. This is supplementary data that enhances the overall analytics experience.

**Independent Test**: Can be fully tested by using Claude Code at different times and verifying the monitor captures hourCounts from stats-cache.json and makes them available.

**Acceptance Scenarios**:

1. **Given** stats-cache.json contains hourCounts data, **When** the monitor reads it, **Then** it emits an activity pattern event with counts per hour (0-23).
2. **Given** activity data spans multiple days, **When** viewing patterns, **Then** the data reflects cumulative counts across all tracked days.

---

### User Story 8 - View Model Distribution (Priority: P3)

As a developer, I want to see the distribution of my usage across different Claude models (opus, sonnet, haiku) so that I can understand which models I'm relying on most.

**Why this priority**: Model distribution helps developers understand their usage patterns but is supplementary to the core token tracking functionality.

**Independent Test**: Can be fully tested by using different models during sessions and verifying the monitor captures per-model usage from stats-cache.json.

**Acceptance Scenarios**:

1. **Given** stats-cache.json contains modelUsage data, **When** the monitor reads it, **Then** it extracts usage metrics for each model separately.
2. **Given** model distribution is tracked, **When** viewing the data, **Then** percentage or ratio of usage per model can be derived.

---

### User Story 9 - Track Active Projects (Priority: P3)

As a developer, I want to see which projects have active Claude Code sessions so that I can understand where my development activity is focused.

**Why this priority**: Project activity tracking is supplementary metadata that enhances the overall analytics experience but is not critical for core monitoring.

**Independent Test**: Can be fully tested by running Claude Code in different project directories and verifying the monitor identifies active projects from the projects/ directory structure.

**Acceptance Scenarios**:

1. **Given** a session JSONL exists in projects/<path>/, **When** the monitor scans the directory, **Then** it identifies the project path from the directory name.
2. **Given** a session has no summary event, **When** checking project activity, **Then** the project is considered to have an active session.
3. **Given** all sessions for a project have summary events, **When** checking project activity, **Then** the project has no active sessions.

---

### Edge Cases

- What happens when stats-cache.json is corrupted or malformed? The monitor logs a warning, waits briefly (100ms), and retries. If still failing, continues with previously known values.
- What happens when a todo file is deleted while being watched? The monitor removes tracking for that session's todos and cleans up associated state.
- What happens when file-history versions are non-sequential (e.g., v1, v3, missing v2)? The monitor diffs against the highest available previous version.
- What happens when history.jsonl grows very large? The monitor only watches for new appended lines (tail-like behavior), not reprocessing existing content.
- What happens when multiple sessions write to stats-cache.json simultaneously? The monitor reads the latest state; stats-cache.json is an aggregate file so this is expected behavior.
- What happens when a session JSONL is deleted before summary event is detected? The monitor cleans up tracking state for that session but does NOT mark todos as abandoned (only summary events trigger abandonment).
- What happens when the monitor restarts? It re-scans all monitored directories from scratch, re-establishing watches and reprocessing current state (no position persistence).
- What happens when inotify watch limits are approached? The monitor logs a warning when approaching system limits (80% of max_user_watches).
- What happens when events from different sources arrive simultaneously? Events have no ordering guarantees across event types; consumers must handle interleaved events.

## Requirements *(mandatory)*

### Functional Requirements

#### Agent Spawn Tracking
- **FR-001**: System MUST parse session JSONL files for `tool_use` events where the tool name is "Task", extracting `subagent_type` from the parameters object.
- **FR-002**: System MUST emit an agent event for each detected Task tool invocation containing session ID and agent type.
- **FR-003**: System MUST track agent spawn counts per session.

#### Skill Invocation Tracking
- **FR-004**: System MUST watch ~/.claude/history.jsonl for new entries.
- **FR-005**: System MUST identify skill invocations as lines where the display field starts with "/".
- **FR-006**: System MUST emit a skill event containing the command name (without arguments), timestamp, and session ID. Command name extraction uses shell-like tokenization: capture everything after `/` until first unquoted whitespace.

#### Token Usage Tracking (Global Aggregates per Model)
- **FR-007**: System MUST watch ~/.claude/stats-cache.json for changes.
- **FR-008**: System MUST extract global token usage per model including input tokens, output tokens, cache read tokens, and cache creation tokens.
- **FR-009**: System MUST emit token usage events containing global aggregates when stats-cache.json is updated. These events do not include a session_id as they represent cumulative totals per model.

#### Session Metrics Tracking (Global Aggregates)
- **FR-010**: System MUST extract global session metrics from stats-cache.json including total messages (across all sessions), total tool calls (across all sessions), total session count, and longest session data (ID, duration, message count).
- **FR-011**: System MUST emit session metrics events containing global aggregates when stats-cache.json is updated. These events do not include a session_id as they represent cumulative data.

#### Activity Pattern Tracking
- **FR-012**: System MUST extract hourCounts data from stats-cache.json.
- **FR-013**: System MUST emit activity pattern events containing usage counts per hour.

#### Model Distribution Tracking
- **FR-014**: System MUST extract per-model usage data from stats-cache.json modelUsage field.
- **FR-015**: System MUST emit model distribution events showing usage breakdown by model.

#### Todo Progress Tracking
- **FR-016**: System MUST watch ~/.claude/todos/*.json files for changes.
- **FR-017**: System MUST parse todo files to count tasks by status (completed, pending, in_progress).
- **FR-018**: System MUST correlate todo files with sessions via the session ID in the filename.
- **FR-019**: System MUST detect session end by monitoring for summary events in session JSONL files. A summary event is a JSON line with `{"type": "summary", ...}`. Summary events are the ONLY trigger for todo abandonment. The summary event may appear at any position in the JSONL (not necessarily last line).
- **FR-020**: System MUST reclassify incomplete tasks as abandoned ONLY when a summary event is detected for their associated session. File deletion or other cleanup does NOT trigger abandonment—only summary events do.
- **FR-021**: System MUST emit todo events containing per-session counts for completed, pending, in_progress, and abandoned tasks.

#### File Edit History Tracking
- **FR-022**: System MUST watch ~/.claude/file-history/<session-id>/ directories for new files.
- **FR-023**: System MUST identify file versions by the @vN suffix pattern.
- **FR-024**: System MUST perform diffs only for versions N >= 2, comparing vN against v(N-1).
- **FR-025**: System MUST NOT process v1 files (initial state, no diff possible).
- **FR-026**: System MUST calculate lines added (lines starting with >) and lines removed (lines starting with <) from diff output.
- **FR-027**: System MUST emit file_change events containing session ID, lines_added, and lines_removed.

#### Project Activity Tracking
- **FR-028**: System MUST scan ~/.claude/projects/ directory structure to identify projects.
- **FR-029**: System MUST determine project activity status by checking for sessions without summary events.
- **FR-030**: System MUST emit project activity events indicating which projects have active sessions.

#### General Requirements
- **FR-031**: Per-session events (AgentSpawn, SkillInvocation, TodoProgress, FileChange) MUST include session ID. Global aggregate events (TokenUsage, SessionMetrics, ActivityPattern, ModelDistribution, ProjectActivity) do not include session ID.
- **FR-032**: All events MUST include timestamps.
- **FR-033**: Existing tool usage tracking MUST remain unchanged.
- **FR-034**: System MUST handle missing or inaccessible files gracefully without crashing.

### Non-Functional Requirements

- **NFR-001**: File watching MUST use efficient mechanisms (inotify on Linux) to minimize CPU usage.
- **NFR-002**: Stats-cache.json parsing MUST complete within 100ms to avoid blocking the event loop.
- **NFR-003**: File diff operations MUST be performed asynchronously to avoid blocking other event processing.
- **NFR-004**: The monitor MUST not read or transmit actual file contents, prompts, or code for privacy.
- **NFR-005**: File change events MUST be debounced with a configurable interval (default 200ms) to coalesce rapid updates.
- **NFR-006**: System MUST bound memory usage for tracking state, with configurable limits for maximum tracked sessions (default: 1000).
- **NFR-007**: System MUST handle JSON parse failures gracefully by retrying after a brief delay, not emitting events based on corrupted reads.
- **NFR-008**: System MUST gracefully handle watch removal when monitored directories are deleted or permissions change.
- **NFR-009**: When event processing cannot keep pace with file changes, the system MUST coalesce pending events to prevent unbounded memory growth.
- **NFR-010**: System MUST recover gracefully from crashes, re-scanning monitored directories on startup rather than persisting position state.
- **NFR-011**: On shutdown signal, system MUST complete processing of in-flight events and flush any buffered data before terminating.

### Development Standards

- **DS-001**: Project MUST include Justfile with common development commands (build, test, run, lint, format).
- **DS-002**: Project MUST include pre-commit hooks (lefthook) for linting and formatting checks.
- **DS-003**: All commits MUST follow conventional commit message format.
- **DS-004**: Implementation MUST include unit tests for each new parser and integration tests for file watching.
- **DS-005**: All new code MUST pass existing linting rules (clippy for Rust, ESLint for TypeScript).
- **DS-006**: All new code MUST be formatted according to project standards (rustfmt, Prettier).

### Key Entities

- **AgentSpawnEvent**: session_id, agent_type, description, timestamp
- **SkillInvocationEvent**: session_id, skill_name, project, timestamp
- **TokenUsageEvent**: model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens (global aggregates per model from stats-cache.json; no session_id as this represents cumulative totals; one event emitted per model per stats-cache.json update)
- **SessionMetricsEvent**: total_sessions, total_messages, total_tool_usage, longest_session (global aggregates from stats-cache.json; no session_id as this represents all sessions)
- **ActivityPatternEvent**: hour_counts (map of hour -> count)
- **ModelDistributionEvent**: model_usage (map of model -> TokenUsageSummary)
- **TodoProgressEvent**: session_id, completed, pending, in_progress, abandoned (boolean: true if session ended with incomplete tasks)
- **FileChangeEvent**: session_id, file_hash, version, lines_added, lines_removed, lines_modified, timestamp
- **ProjectActivityEvent**: project_path, session_id, is_active

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Token usage events are emitted within 2 seconds (P95 latency) of stats-cache.json filesystem modification timestamp, measured at event emission time.
- **SC-002**: Agent spawn events are emitted within 1 second (P95 latency) of Task tool invocation appearing in session JSONL, measured from JSONL write to event emission.
- **SC-003**: Skill invocation events capture 100% of slash commands written to history.jsonl.
- **SC-004**: Todo abandonment is correctly detected for 100% of sessions that end with incomplete tasks.
- **SC-005**: File change line counts match manual diff calculations for all processed file versions.
- **SC-006**: The monitor processes all new data sources without increasing idle CPU usage above the existing 5% threshold.
- **SC-007**: All per-session data can be aggregated by the client to produce accurate global totals.
- **SC-008**: No sensitive content (code, prompts, file contents) is included in any emitted events.

## File Format Specifications

### Todo Filename Format
Todo files follow the pattern: `<session-uuid>-agent-<session-uuid>.json`
- Example: `13e7f90a-4818-445a-9463-4f1cc52364df-agent-13e7f90a-4818-445a-9463-4f1cc52364df.json`
- The session UUID appears twice in the filename; extract from the first segment before `-agent-`
- Files not matching this pattern are ignored

### File-History Version Format
File versions follow the pattern: `<hash>@v<N>` where N is a positive integer
- Example: `3f79c7095dc57fea@v2`
- The hash is a 16-character hexadecimal identifier
- Version numbers start at 1 and increment
- Extract version number using regex: `@v(\d+)$`
- Leading zeros are valid (e.g., `@v02` parses as version 2)
- When version gaps exist (e.g., v1, v3, missing v2), diff against highest available previous version and log a warning

### History.jsonl Entry Format
Each line is a JSON object with at minimum:
- `display`: string - The user's input text
- `timestamp`: number - Unix timestamp in milliseconds
- `sessionId`: string - The session UUID
- `project`: string - The project path

Skill invocations are identified when `display` starts with `/`. Command name extraction:
- Parse using shell-like tokenization (respects quoted strings)
- Capture everything after `/` until first unquoted whitespace
- Examples: `/commit -m "msg"` → `commit`, `/"my skill" arg` → `"my skill"`, `/sdd:plan` → `sdd:plan`

## Assumptions

- The ~/.claude directory structure and file formats remain stable across Claude Code versions.
- stats-cache.json is updated atomically or frequently enough that partial reads are not a concern.
- Session IDs in todo filenames reliably match session IDs in project JSONL files.
- The file-history directory uses consistent @vN versioning where N is a positive integer.
- history.jsonl is append-only and new entries appear at the end of the file.
- The monitor has read access to all files in ~/.claude directory.
- Line diff calculation uses simple line-by-line comparison (no external diff tool required).
