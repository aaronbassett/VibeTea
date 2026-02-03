# Research: Monitor Enhanced Data Tracking

**Date**: 2026-02-03
**Branch**: `005-monitor-enhanced-tracking`
**Status**: Complete

## Research Questions & Findings

### 1. stats-cache.json Structure

**Location**: `~/.claude/stats-cache.json`

**Schema**:
```json
{
  "totalSessions": 150,
  "totalMessages": 2500,
  "totalToolUsage": 8000,
  "longestSession": "00:45:30",
  "hourCounts": {
    "0": 10, "1": 5, "2": 2, ..., "23": 50
  },
  "modelUsage": {
    "claude-sonnet-4-20250514": {
      "inputTokens": 1500000,
      "outputTokens": 300000,
      "cacheReadInputTokens": 800000,
      "cacheCreationInputTokens": 100000
    },
    "claude-opus-4-5-20251101": {
      "inputTokens": 500000,
      "outputTokens": 100000,
      "cacheReadInputTokens": 200000,
      "cacheCreationInputTokens": 50000
    }
  },
  "dailyActivity": {
    "2026-02-01": { "messageCount": 100, "sessionCount": 5 },
    "2026-02-02": { "messageCount": 150, "sessionCount": 7 }
  }
}
```

**Key Fields for Tracking**:
- `modelUsage.<model-id>`: Per-model token counts (FR-007)
- `totalSessions`, `totalMessages`, `totalToolUsage`: Global metrics (FR-008, FR-009)
- `hourCounts`: Activity pattern by hour 0-23 (FR-012)
- `longestSession`: Duration tracking
- `dailyActivity`: Daily message/session aggregates

**Parsing Notes**:
- File is overwritten atomically on each Claude Code session change
- No per-session breakdown in this file - global aggregates only
- Model IDs include full version suffix (e.g., `claude-opus-4-5-20251101`)

---

### 2. history.jsonl Format

**Location**: `~/.claude/history.jsonl`

**Schema** (one JSON object per line):
```json
{"display": "/commit", "timestamp": 1738567268363, "project": "/home/ubuntu/Projects/VibeTea", "sessionId": "6e45a55c-3124-4cc8-ad85-040a5c316009"}
```

**Fields**:
- `display`: The skill/command invoked (always starts with `/`)
- `timestamp`: Unix milliseconds
- `project`: Absolute path to project root
- `sessionId`: UUID of the session that invoked the skill

**Parsing Notes**:
- Append-only file (tail for new entries)
- Only skill invocations are logged (not tool usage)
- `display` field captures user-visible command name
- Can correlate to sessions via `sessionId` field

---

### 3. Todo File Naming Pattern

**Location**: `~/.claude/todos/<session-uuid>-agent-<session-uuid>.json`

**Pattern**: `{session-uuid}-agent-{session-uuid}.json`

The session UUID appears twice, separated by `-agent-`.

**Content Schema**:
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

**Status Values**:
- `completed`: Task finished successfully
- `in_progress`: Task currently being worked on
- `pending`: Task waiting to start

**Parsing Notes**:
- Session UUID extraction: split filename on `-agent-`, take first part
- File is overwritten on each todo update (not appended)
- Track file modification events for progress updates
- Abandonment detection: correlate with session summary event

---

### 4. file-history Versioning

**Location**: `~/.claude/file-history/<session-uuid>/<hash>@vN`

**Pattern**: `{16-char-hex-hash}@v{version-number}`

**Examples**:
```
3a8f2c1b9e4d7a6f@v1
3a8f2c1b9e4d7a6f@v2
3a8f2c1b9e4d7a6f@v3
7b2e4f8c1d9a3e5b@v1
```

**Versioning**:
- Hash appears to be derived from file path (consistent across versions)
- Versions increment: v1, v2, v3, etc.
- Each version contains the full file content at that point

**Diff Strategy**:
- For v1: Skip (no previous version to diff against) - per spec FR-025
- For vN (N >= 2): Diff against v(N-1)
- Use Rust `similar` or `diff` crate for generating diffs
- Extract only metadata (lines added/removed/modified count)

**Parsing Notes**:
- Files contain raw file content, not diffs
- Must read both vN-1 and vN to compute diff
- Async diff operations to avoid blocking (NFR-003)

---

### 5. Session JSONL Task Tool Structure

**Location**: `~/.claude/projects/<path-slug>/<session-uuid>.jsonl`

**Task Tool Invocation** (within assistant message):
```json
{
  "type": "assistant",
  "message": {
    "content": [
      {
        "type": "tool_use",
        "id": "toolu_01Fw1HCjXzYHNtWX7jXWzgBj",
        "name": "Task",
        "input": {
          "description": "Create SmileError enum",
          "prompt": "Create the SmileError enum...",
          "subagent_type": "devs:rust-dev"
        }
      }
    ]
  },
  "timestamp": "2026-02-03T05:01:57.678Z"
}
```

**Key Fields**:
- `message.content[].type`: Must be `"tool_use"`
- `message.content[].name`: Must be `"Task"`
- `message.content[].input.subagent_type`: The agent type being spawned (FR-001)
- `message.content[].input.description`: Short description of agent task

**Agent Types Observed**:
- `devs:rust-dev`
- `devs:typescript-dev`
- `devs:react-dev`
- `devs:python-expert`
- `Explore`
- `Plan`
- `general-purpose`

**Parsing Notes**:
- Reuse existing JSONL parser from `parser.rs`
- Filter for `type: "assistant"` entries
- Search `message.content[]` for `tool_use` with `name: "Task"`
- Extract `subagent_type` from `input` object

---

### 6. Projects Directory Structure

**Location**: `~/.claude/projects/<path-slug>/`

**Structure**:
```
~/.claude/projects/
├── -home-ubuntu-Projects-VibeTea/
│   ├── 6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl  # Active session
│   ├── a1b2c3d4-5678-90ab-cdef-1234567890ab.jsonl  # Completed session
│   └── CLAUDE.local.md
├── -home-ubuntu-Projects-SMILE/
│   ├── 60fc5b5e-a285-4a6d-b9cc-9a315eb90ea8.jsonl
│   └── 568bfa37-5e30-42a0-8853-eb51c55d54c3.jsonl
```

**Path Slug**: Absolute path with `/` replaced by `-`
Example: `/home/ubuntu/Projects/VibeTea` → `-home-ubuntu-Projects-VibeTea`

**Session Activity Detection**:
- **Active Session**: Last line does NOT have `{"type": "summary"}`
- **Completed Session**: Last line HAS `{"type": "summary"}`

**Summary Event Structure**:
```json
{
  "type": "summary",
  "summary": "Session summary text...",
  "leafUuid": "uuid-of-last-message",
  "timestamp": "2026-02-03T05:30:00.000Z"
}
```

**Parsing Notes**:
- Watch for new `.jsonl` files (new sessions)
- Watch for file modifications (session activity)
- Read last line to determine session state
- Correlate with todo files for abandonment tracking

---

### 7. inotify Limits

**Current System Limit**: `max_user_watches = 495440`

**Watches Required**:
- 1 for `~/.claude/stats-cache.json`
- 1 for `~/.claude/history.jsonl`
- 1 for `~/.claude/todos/` directory
- 1 for `~/.claude/file-history/` directory
- 1 per project directory in `~/.claude/projects/`
- Existing session JSONL watches (already implemented)

**Estimate**: ~50-100 watches for typical usage (well under limit)

**Detection Strategy**:
- Check `/proc/sys/fs/inotify/max_user_watches` at startup
- Log warning if current watch count exceeds 50% of limit
- Graceful degradation: skip non-critical watches if limit approached

---

## Implementation Recommendations

### Tracker Module Structure

```rust
// trackers/mod.rs
pub mod agent_tracker;      // Task tool parsing from session JSONL
pub mod skill_tracker;      // history.jsonl watching
pub mod stats_tracker;      // stats-cache.json watching
pub mod todo_tracker;       // todos/*.json watching
pub mod file_history_tracker; // file-history diff tracking
pub mod project_tracker;    // projects/ activity tracking
```

### Event Type Additions

```rust
pub enum EventPayload {
    // Existing
    ToolUsage { ... },

    // New (FR-001-003)
    AgentSpawn {
        session_id: String,
        agent_type: String,
        description: String,
        timestamp: DateTime<Utc>,
    },

    // New (FR-004-006)
    SkillInvocation {
        session_id: String,
        skill_name: String,
        project: String,
        timestamp: DateTime<Utc>,
    },

    // New (FR-007-015)
    TokenUsage {
        model: String,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_creation_tokens: u64,
    },

    SessionMetrics {
        total_sessions: u64,
        total_messages: u64,
        total_tool_usage: u64,
        longest_session: String,
    },

    ActivityPattern {
        hour_counts: HashMap<u8, u64>,
    },

    ModelDistribution {
        model_usage: HashMap<String, TokenUsage>,
    },

    // New (FR-016-021)
    TodoProgress {
        session_id: String,
        completed: u32,
        in_progress: u32,
        pending: u32,
        abandoned: bool,
    },

    // New (FR-022-027)
    FileChange {
        session_id: String,
        file_hash: String,
        version: u32,
        lines_added: u32,
        lines_removed: u32,
        lines_modified: u32,
    },

    // New (FR-028-030)
    ProjectActivity {
        project_path: String,
        session_id: String,
        is_active: bool,
    },
}
```

### Debouncing Strategy

| Source | Debounce | Rationale |
|--------|----------|-----------|
| stats-cache.json | 200ms | Updates frequently during sessions |
| history.jsonl | 0ms | Append-only, process immediately |
| todos/*.json | 100ms | Moderate update frequency |
| file-history/* | 100ms | Batch multiple version saves |
| projects/*.jsonl | 0ms | Existing behavior, no change |

### Privacy Considerations

All new events contain only metadata:
- Session UUIDs (already exposed)
- Tool/skill names (not content)
- Counts and timestamps
- Model names (public info)
- File hashes (not content)

No code, prompts, or file contents are transmitted. Privacy pipeline remains unchanged.

---

## Questions Resolved

| Question | Resolution |
|----------|------------|
| stats-cache.json schema | Documented above - global aggregates with modelUsage map |
| history.jsonl format | JSONL with display, timestamp, sessionId, project |
| todo file naming | `{session-uuid}-agent-{session-uuid}.json` |
| file-history versioning | `{hash}@vN` with sequential versions |
| Task tool structure | `tool_use` with `name: "Task"`, `input.subagent_type` |
| Session activity detection | Check for `type: "summary"` in last line |
| inotify limits | 495440 watches available, ~100 needed |

---

**Next Phase**: Phase 1 Design - Create data-model.md, contracts/, and quickstart.md
