# Data Model: Monitor Enhanced Data Tracking

**Date**: 2026-02-03
**Branch**: `005-monitor-enhanced-tracking`
**Status**: Phase 1 Design

## Entity Definitions

### AgentSpawnEvent

Tracks when the Task tool spawns a specialized agent.

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `session_id` | `String` | UUID of parent session | Session JSONL filename |
| `agent_type` | `String` | Agent type (e.g., `devs:rust-dev`, `Explore`) | `tool_use.input.subagent_type` |
| `description` | `String` | Short description of agent task | `tool_use.input.description` |
| `timestamp` | `DateTime<Utc>` | When agent was spawned | JSONL entry timestamp |

**Rust Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpawnEvent {
    pub session_id: String,
    pub agent_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}
```

**Privacy**: No sensitive data. Agent types and descriptions are metadata.

---

### SkillInvocationEvent

Tracks skill/command invocations from history.jsonl.

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `session_id` | `String` | UUID of invoking session | `sessionId` field |
| `skill_name` | `String` | Skill invoked (e.g., `/commit`) | `display` field |
| `project` | `String` | Project path | `project` field |
| `timestamp` | `DateTime<Utc>` | When skill was invoked | `timestamp` field (ms) |

**Rust Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInvocationEvent {
    pub session_id: String,
    pub skill_name: String,
    pub project: String,
    pub timestamp: DateTime<Utc>,
}
```

**Privacy**: Skill names are not sensitive. Project paths are already exposed via existing events.

---

### TokenUsageEvent

Tracks per-model token consumption from stats-cache.json.

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `model` | `String` | Model identifier (e.g., `claude-opus-4-5-20251101`) | `modelUsage` key |
| `input_tokens` | `u64` | Total input tokens | `inputTokens` |
| `output_tokens` | `u64` | Total output tokens | `outputTokens` |
| `cache_read_tokens` | `u64` | Tokens read from cache | `cacheReadInputTokens` |
| `cache_creation_tokens` | `u64` | Tokens used to create cache | `cacheCreationInputTokens` |

**Rust Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageEvent {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
}
```

**Privacy**: Token counts are aggregate metrics, not sensitive.

---

### SessionMetricsEvent

Tracks global session metrics from stats-cache.json.

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `total_sessions` | `u64` | Total session count | `totalSessions` |
| `total_messages` | `u64` | Total message count | `totalMessages` |
| `total_tool_usage` | `u64` | Total tool invocations | `totalToolUsage` |
| `longest_session` | `String` | Duration of longest session (HH:MM:SS) | `longestSession` |

**Rust Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetricsEvent {
    pub total_sessions: u64,
    pub total_messages: u64,
    pub total_tool_usage: u64,
    pub longest_session: String,
}
```

**Privacy**: Aggregate counts only.

---

### ActivityPatternEvent

Tracks hourly activity distribution from stats-cache.json.

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `hour_counts` | `HashMap<u8, u64>` | Messages per hour (0-23) | `hourCounts` |

**Rust Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPatternEvent {
    pub hour_counts: HashMap<u8, u64>,
}
```

**Privacy**: Aggregate hourly distribution only.

---

### ModelDistributionEvent

Tracks usage distribution across models from stats-cache.json.

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `model_usage` | `HashMap<String, TokenUsageSummary>` | Per-model token breakdown | `modelUsage` |

**TokenUsageSummary**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageSummary {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
}
```

**Rust Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDistributionEvent {
    pub model_usage: HashMap<String, TokenUsageSummary>,
}
```

**Privacy**: Model names and aggregate token counts only.

---

### TodoProgressEvent

Tracks todo list progress per session from todos/*.json.

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `session_id` | `String` | Session UUID | Filename parsing |
| `completed` | `u32` | Count of completed tasks | Status count |
| `in_progress` | `u32` | Count of in-progress tasks | Status count |
| `pending` | `u32` | Count of pending tasks | Status count |
| `abandoned` | `bool` | True if session ended with incomplete tasks | Summary event correlation |

**Rust Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoProgressEvent {
    pub session_id: String,
    pub completed: u32,
    pub in_progress: u32,
    pub pending: u32,
    pub abandoned: bool,
}
```

**Privacy**: Task counts only, no task content.

---

### FileChangeEvent

Tracks file edit history from file-history/ directory.

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `session_id` | `String` | Session UUID | Directory name |
| `file_hash` | `String` | 16-char hex hash of file | Filename prefix |
| `version` | `u32` | Version number | Filename suffix |
| `lines_added` | `u32` | Lines added in this version | Diff computation |
| `lines_removed` | `u32` | Lines removed in this version | Diff computation |
| `lines_modified` | `u32` | Lines modified in this version | Diff computation |
| `timestamp` | `DateTime<Utc>` | When file change was detected | File modification time |

**Rust Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    pub session_id: String,
    pub file_hash: String,
    pub version: u32,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub lines_modified: u32,
    pub timestamp: DateTime<Utc>,
}
```

**Privacy**: Line counts only. No file paths or content transmitted. The file_hash is a content-addressable identifier derived from the file's original path using a one-way hash function; it cannot be reversed to reveal the actual file path.

---

### ProjectActivityEvent

Tracks project activity from projects/ directory.

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `project_path` | `String` | Project path (from slug) | Directory name |
| `session_id` | `String` | Session UUID | JSONL filename |
| `is_active` | `bool` | True if session is ongoing | Last line check |

**Rust Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectActivityEvent {
    pub project_path: String,
    pub session_id: String,
    pub is_active: bool,
}
```

**Privacy**: Project paths already exposed. Activity status is metadata.

---

## Extended EventPayload Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    // Existing variants
    ToolUsage(ToolUsageEvent),

    // New variants (FR-001-030)
    AgentSpawn(AgentSpawnEvent),
    SkillInvocation(SkillInvocationEvent),
    TokenUsage(TokenUsageEvent),
    SessionMetrics(SessionMetricsEvent),
    ActivityPattern(ActivityPatternEvent),
    ModelDistribution(ModelDistributionEvent),
    TodoProgress(TodoProgressEvent),
    FileChange(FileChangeEvent),
    ProjectActivity(ProjectActivityEvent),
}
```

---

## TypeScript Types (Client)

```typescript
// New event types
export interface AgentSpawnEvent {
  type: 'agent_spawn';
  sessionId: string;
  agentType: string;
  description: string;
  timestamp: string;
}

export interface SkillInvocationEvent {
  type: 'skill_invocation';
  sessionId: string;
  skillName: string;
  project: string;
  timestamp: string;
}

export interface TokenUsageEvent {
  type: 'token_usage';
  model: string;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheCreationTokens: number;
}

export interface SessionMetricsEvent {
  type: 'session_metrics';
  totalSessions: number;
  totalMessages: number;
  totalToolUsage: number;
  longestSession: string;
}

export interface ActivityPatternEvent {
  type: 'activity_pattern';
  hourCounts: Record<number, number>;
}

export interface ModelDistributionEvent {
  type: 'model_distribution';
  modelUsage: Record<string, TokenUsageSummary>;
}

export interface TokenUsageSummary {
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheCreationTokens: number;
}

export interface TodoProgressEvent {
  type: 'todo_progress';
  sessionId: string;
  completed: number;
  inProgress: number;
  pending: number;
  abandoned: boolean;
}

export interface FileChangeEvent {
  type: 'file_change';
  sessionId: string;
  fileHash: string;
  version: number;
  linesAdded: number;
  linesRemoved: number;
  linesModified: number;
  timestamp: string;
}

export interface ProjectActivityEvent {
  type: 'project_activity';
  projectPath: string;
  sessionId: string;
  isActive: boolean;
}

// Extended EventPayload union
export type EventPayload =
  | ToolUsageEvent
  | AgentSpawnEvent
  | SkillInvocationEvent
  | TokenUsageEvent
  | SessionMetricsEvent
  | ActivityPatternEvent
  | ModelDistributionEvent
  | TodoProgressEvent
  | FileChangeEvent
  | ProjectActivityEvent;
```

---

## Entity Relationships

```
stats-cache.json
    ├── TokenUsageEvent (per model)
    ├── SessionMetricsEvent (global)
    ├── ActivityPatternEvent (hourly)
    └── ModelDistributionEvent (summary)

history.jsonl
    └── SkillInvocationEvent (per entry)
            │
            └── correlates via sessionId

projects/<slug>/<session>.jsonl
    ├── AgentSpawnEvent (per Task tool_use)
    └── ProjectActivityEvent (per session)
            │
            └── correlates with todos/

todos/<session>-agent-<session>.json
    └── TodoProgressEvent (per session)
            │
            └── abandoned flag set when
                session summary received

file-history/<session>/<hash>@vN
    └── FileChangeEvent (per version >= 2)
```

---

## Aggregation Strategy

### Client-Side Aggregation

| Event Type | Aggregation | Scope |
|------------|-------------|-------|
| AgentSpawn | Count by agent_type | Per session, global |
| SkillInvocation | Count by skill_name | Per session, per project |
| TokenUsage | Sum all token counts | Per model |
| SessionMetrics | Display latest | Global |
| ActivityPattern | Display heatmap | Global |
| ModelDistribution | Pie chart | Global |
| TodoProgress | Sum counts | Per session |
| FileChange | Sum line counts | Per session |
| ProjectActivity | List active sessions | Per project |

### Store Selectors (Zustand)

```typescript
// Recommended selectors for memoized aggregation
const useAgentSpawnCounts = () =>
  useEventStore((state) =>
    state.events
      .filter((e): e is AgentSpawnEvent => e.type === 'agent_spawn')
      .reduce((acc, e) => {
        acc[e.agentType] = (acc[e.agentType] || 0) + 1;
        return acc;
      }, {} as Record<string, number>)
  );

const useTokenUsageByModel = () =>
  useEventStore((state) =>
    state.events.filter((e): e is TokenUsageEvent => e.type === 'token_usage')
  );
```

---

## Privacy Compliance Matrix

| Event Type | Contains | Privacy Status |
|------------|----------|----------------|
| AgentSpawn | Agent type, description, timestamp | Safe - metadata only |
| SkillInvocation | Skill name, project path, timestamp | Safe - command names only |
| TokenUsage | Token counts by model | Safe - aggregate metrics |
| SessionMetrics | Counts and duration | Safe - aggregate metrics |
| ActivityPattern | Hourly counts | Safe - time patterns only |
| ModelDistribution | Model names and counts | Safe - public model info |
| TodoProgress | Task counts, abandoned flag | Safe - no task content |
| FileChange | Line counts, file hash, timestamp | Safe - no file content (see note below) |
| ProjectActivity | Project path, activity status | Safe - already exposed |

All events pass Constitution Principle I (Privacy by Design).

### File Hash Privacy Note

The `file_hash` field in FileChangeEvent is a **content-addressable identifier** derived from the original file path using a one-way hash function. It:
- Cannot be reversed to reveal the actual file path
- Is consistent within a session (same file = same hash)
- Does not reveal file names, directory structure, or file contents
- Is used only for deduplication and tracking edit counts per logical file

This approach ensures compliance with Constitution Principle I while enabling meaningful file edit aggregation.
