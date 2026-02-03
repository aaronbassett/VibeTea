# Research Log: vibetea

**Updated**: 2026-02-02 12:15 UTC

---

## R1: Claude Code JSONL Session File Format

**Date**: 2026-02-02
**Questions Addressed**: Q3, Q4, Q5, Q14
**Source**: Direct examination of `~/.claude/projects/**/*.jsonl` files

### Findings

#### File Location
Session files are stored at: `~/.claude/projects/<project-path-slug>/<session-uuid>.jsonl`

- Project path is slugified (e.g., `/home/ubuntu/Projects/VibeTea` → `-home-ubuntu-Projects-VibeTea`)
- Each session has a UUID-named file
- Subagent sessions stored in `<session-uuid>/subagents/` subdirectory

#### Event Types (from Claude Code)

| Type | Description | Key Fields |
|------|-------------|------------|
| `system` | System events (commands, hooks) | `subtype`, `content`, `level` |
| `user` | User input messages | `message.role`, `message.content` |
| `assistant` | Claude responses | `message.model`, `message.content`, `message.usage` |
| `progress` | Tool/hook progress | `data.type`, `toolUseID`, `parentToolUseID` |
| `file-history-snapshot` | File history state | `snapshot.trackedFileBackups` |
| `summary` | Session summary | `summary` (text description) |

#### Common Event Fields

All events share these fields:
```json
{
  "type": "string",           // Event type (required)
  "sessionId": "uuid",        // Session identifier
  "uuid": "uuid",             // Unique event ID
  "parentUuid": "uuid|null",  // Parent event for threading
  "timestamp": "ISO-8601",    // Event timestamp
  "cwd": "/path",             // Working directory
  "version": "2.1.29",        // Claude Code version
  "gitBranch": "string",      // Current git branch
  "isSidechain": "boolean",   // Whether this is a sidechain event
  "userType": "external"      // User type
}
```

#### Assistant Event with Tool Use

```json
{
  "type": "assistant",
  "message": {
    "model": "claude-opus-4-5-20251101",
    "id": "msg_xxx",
    "role": "assistant",
    "content": [
      {"type": "text", "text": "..."},
      {"type": "tool_use", "id": "toolu_xxx", "name": "Read", "input": {...}}
    ],
    "usage": {
      "input_tokens": 1000,
      "output_tokens": 500,
      "cache_read_input_tokens": 5000,
      "cache_creation_input_tokens": 2000
    }
  }
}
```

#### Progress Event (Hook/Tool Progress)

```json
{
  "type": "progress",
  "data": {
    "type": "hook_progress",
    "hookEvent": "PostToolUse",
    "hookName": "PostToolUse:Read",
    "command": "callback"
  },
  "toolUseID": "toolu_xxx",
  "parentToolUseID": "toolu_xxx"
}
```

### Mapping to VibeTea Events

| Claude Code Type | VibeTea Type | Extracted Fields |
|------------------|--------------|------------------|
| `assistant` with tool_use | `tool` | tool name, status="started" |
| `progress` (PostToolUse) | `tool` | tool name, status="completed" |
| `user` | `activity` | timestamp only |
| `summary` | `summary` | summary text |
| `system` (session start) | `session` | action="started" |

### Privacy Extraction Rules

From `assistant.message.content`:
- Tool name: Extract from `content[].name` where `type == "tool_use"`
- File context: Extract basename from tool input parameters (e.g., `input.file_path`)
- Token counts: Extract from `message.usage`

From tool inputs (strip sensitive data):
- `Read.file_path` → basename only (e.g., `/home/user/project/src/auth.ts` → `auth.ts`)
- `Bash.command` → use `description` field only, never the command itself
- `Grep.pattern` → omit entirely (may contain sensitive search terms)
- `Write/Edit` → basename only, no content

---

## R2: History File Format

**Date**: 2026-02-02
**Source**: `~/.claude/history.jsonl`

### Findings

The history file contains user prompts across all sessions:

```json
{
  "display": "/spec-writer review the @PRD.md",  // User-visible prompt
  "pastedContents": {},                          // Any pasted content (sensitive!)
  "timestamp": 1770033067795,                    // Unix timestamp (ms)
  "project": "/home/ubuntu/Projects/VibeTea",    // Project path
  "sessionId": "uuid"                            // Session ID
}
```

**Privacy Note**: History file contains raw user prompts which are PROHIBITED from transmission. The Monitor should ONLY read session JSONL files, not history.jsonl.

---

## R3: Session Lifecycle

**Date**: 2026-02-02
**Source**: File system observation

### Findings

1. **Session Creation**: A new `.jsonl` file is created when Claude Code starts a session
2. **Session Continuation**: Events are appended to the same file throughout the session
3. **Session End**: The file remains; a `summary` event is written at session end
4. **Detection**: Watch for new files (session start) and `summary` events (session end)
5. **Multiple Sessions**: Each session has its own file; multiple can be active simultaneously

---
