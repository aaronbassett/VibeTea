# Coding Conventions

**Purpose**: Document code style, naming conventions, error handling, and common patterns.
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Code Style

### Formatting Tools

| Tool | Configuration | Command |
|------|---------------|---------|
| Prettier (TypeScript/Client) | `.prettierrc` | `npm run format` |
| ESLint (TypeScript/Client) | `.eslintrc.cjs` | `npm run lint` |
| rustfmt (Rust/Monitor) | Default settings | `cargo fmt` |
| clippy (Rust/Monitor) | Default lints | `cargo clippy` |

### Style Rules

#### TypeScript/Client

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 2 spaces | Enforced by Prettier |
| Quotes | Single quotes | `'string'` |
| Semicolons | Required | `const x = 1;` |
| Line length | No specific limit | Prettier handles wrapping |
| Trailing commas | ES5 style | Arrays/objects only (not function args) |
| JSX curly braces | Single-line only | `<Component />` or multi-line |

#### Rust/Monitor

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 4 spaces | rustfmt default |
| Strings | Double quotes | `"string"` |
| Line length | 100 chars (soft) | rustfmt respects natural breaks |
| Comments | `//` for lines, `///` for docs | Doc comments on public items |
| Naming | snake_case for functions, PascalCase for types | `fn get_config()`, `struct Config` |

## Naming Conventions

### TypeScript/Client

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Components | PascalCase | N/A (no components yet) |
| Hooks | camelCase with `use` prefix | `useEventStore.ts` |
| Types | PascalCase in `types/` folder | `types/events.ts` contains `VibeteaEvent` |
| Utilities | camelCase | N/A (no utils yet) |
| Constants | SCREAMING_SNAKE_CASE in const files | `MAX_EVENTS = 1000` |
| Test files | Same as source + `.test.ts` | `.test.tsx` for React components |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Variables | camelCase | `sessionId`, `eventCount` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_EVENTS`, `DEFAULT_BUFFER_SIZE` |
| Functions | camelCase, verb prefix | `selectEventsBySession()` |
| Classes | PascalCase (rare in modern React) | N/A |
| Interfaces | PascalCase, no `I` prefix | `EventStore`, `Session` |
| Types | PascalCase | `VibeteaEvent<T>`, `EventPayload` |
| Enums | PascalCase | N/A (use union types instead) |

### Rust/Monitor

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `config.rs`, `error.rs`, `types.rs` |
| Types | PascalCase | `Config`, `Event`, `MonitorError` |
| Constants | SCREAMING_SNAKE_CASE | `EVENT_ID_PREFIX`, `DEFAULT_BUFFER_SIZE` |
| Test modules | `#[cfg(test)] mod tests` | In same file as implementation |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Functions | snake_case | `from_env()`, `generate_event_id()` |
| Constants | SCREAMING_SNAKE_CASE | `EVENT_ID_SUFFIX_LEN = 20` |
| Structs | PascalCase | `Config`, `Event` |
| Enums | PascalCase | `EventType`, `SessionAction`, `MonitorError` |
| Methods | snake_case | `.new()`, `.to_string()` |
| Lifetimes | Single lowercase letter | `'a`, `'static` |

## Error Handling

### Error Patterns

#### TypeScript (Not yet implemented)

Client error handling will use:
- Try/catch for async operations
- Zod for schema validation with `.parse()` and `.safeParse()`
- Custom error classes for business logic errors

#### Rust/Monitor

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Configuration errors | Custom enum with `#[derive(Error)]` | `config.rs` defines `ConfigError` |
| I/O errors | Use `#[from]` for automatic conversion | `MonitorError::Io(#[from] std::io::Error)` |
| JSON errors | Automatic conversion via `serde_json` | `MonitorError::Json(#[from] serde_json::Error)` |
| HTTP errors | String-based variants | `MonitorError::Http(String)` |
| Cryptographic errors | String-based variants | `MonitorError::Crypto(String)` |

### Error Response Format

#### TypeScript (Standard for client responses)

```typescript
{
  error: {
    code: 'ERROR_CODE',
    message: 'Human readable message',
    details?: object
  }
}
```

#### Rust Error Messages

Errors use `thiserror::Error` with `#[error]` attributes for automatic `Display` impl:

```rust
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingEnvVar(String),

    #[error("invalid value for {key}: {message}")]
    InvalidValue { key: String, message: String },
}
```

### Logging Conventions

Not yet implemented in codebase. When logging is added:

| Level | When to Use | Example |
|-------|-------------|---------|
| error | Unrecoverable failures | Configuration load failure |
| warn | Recoverable issues | Retry attempt, missing optional config |
| info | Important events | Session start, monitor startup |
| debug | Development details | Event payload serialization |

## Common Patterns

### Event-Driven Architecture

The codebase uses discriminated unions for type-safe event handling:

```typescript
// Type-safe event handling in TypeScript
type VibeteaEvent<T extends EventType = EventType> = {
  id: string;
  source: string;
  timestamp: string;
  type: T;
  payload: EventPayloadMap[T]; // Auto-typed based on T
};

// Type guards for runtime checks
function isSessionEvent(event: VibeteaEvent): event is VibeteaEvent<'session'> {
  return event.type === 'session';
}
```

```rust
// Rust equivalent with tagged enums
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    Session { session_id: Uuid, action: SessionAction, project: String },
    Activity { session_id: Uuid, project: Option<String> },
    // ... other variants
}
```

### Zustand Store Pattern

Client state management uses Zustand with selector functions:

```typescript
export const useEventStore = create<EventStore>()((set) => ({
  // State
  status: 'disconnected',
  events: [],
  sessions: new Map(),

  // Actions (immutable updates)
  addEvent: (event: VibeteaEvent) => {
    set((state) => {
      // Calculate new state from current state
      const newEvents = [event, ...state.events].slice(0, MAX_EVENTS);
      // Return partial updates
      return { events: newEvents };
    });
  },
}));

// Selector utilities to extract derived state
export function selectActiveSessions(state: EventStore): Session[] {
  return Array.from(state.sessions.values()).filter(s => s.status !== 'ended');
}
```

### Configuration Pattern (Rust)

Config loads from environment variables with sensible defaults:

```rust
impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Required vars (no default)
        let server_url = env::var("VIBETEA_SERVER_URL")
            .map_err(|_| ConfigError::MissingEnvVar("VIBETEA_SERVER_URL".to_string()))?;

        // Optional with defaults
        let buffer_size = env::var("VIBETEA_BUFFER_SIZE")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(DEFAULT_BUFFER_SIZE);

        Ok(Self { server_url, buffer_size, /* ... */ })
    }
}
```

## Import Ordering

### TypeScript

Standard import order (enforced conceptually, no linter config):

1. React and external packages (`react`, `react-dom`, `zustand`)
2. Internal modules (`./types/`, `./hooks/`)
3. Relative imports (`./App`, `../sibling`)
4. Type imports (`import type { ... }`)

Example from `useEventStore.ts`:

```typescript
import { create } from 'zustand';

import type { Session, VibeteaEvent } from '../types/events';
```

### Rust

Standard ordering:

1. `use` statements for external crates
2. `use` statements for internal modules
3. `use` statements for types and traits

Example from `error.rs`:

```rust
use thiserror::Error;
use crate::config::ConfigError;
```

## Comments & Documentation

### TypeScript

| Type | When to Use | Format |
|------|-------------|--------|
| JSDoc | Public functions, hooks, interfaces | `/** ... */` |
| Inline | Complex logic or non-obvious code | `// Explanation` |
| Section dividers | Logically group related code | `// -------` comment blocks |
| TODO | Planned work | `// TODO: description` |
| FIXME | Known issues | `// FIXME: description` |

Example from `useEventStore.ts`:

```typescript
/**
 * Zustand store for managing WebSocket event state.
 *
 * Provides centralized state management for the VibeTea event stream,
 * with selective subscriptions to prevent unnecessary re-renders
 * during high-frequency event updates.
 */
```

### Rust

| Type | When to Use | Format |
|------|-------------|--------|
| Doc comments | All public items | `/// ...` or `//! ...` |
| Line comments | Internal logic | `// explanation` |
| Example blocks | Complex public APIs | `/// # Examples` section |
| Panics section | Functions that can panic | `/// # Panics` section |

Example from `config.rs`:

```rust
/// Configuration for the VibeTea Monitor.
#[derive(Debug, Clone)]
pub struct Config {
    /// Server URL for the VibeTea server (e.g., `https://vibetea.fly.dev`).
    pub server_url: String,
}

/// Creates a new `Config` by parsing environment variables.
///
/// # Errors
/// Returns a `ConfigError` if required variables are missing.
///
/// # Example
/// ```no_run
/// use vibetea_monitor::config::Config;
/// let config = Config::from_env().unwrap();
/// ```
pub fn from_env() -> Result<Self, ConfigError> { /* ... */ }
```

## Git Conventions

### Commit Messages

Format: `type(scope): description`

| Type | Usage | Example |
|------|-------|---------|
| feat | New feature | `feat(client): add event store` |
| fix | Bug fix | `fix(monitor): handle missing env var` |
| docs | Documentation | `docs: update conventions` |
| style | Formatting changes | `style: fix ESLint warnings` |
| refactor | Code restructure | `refactor(config): simplify validation` |
| test | Adding/updating tests | `test(config): add environment variable tests` |
| chore | Maintenance, dependencies | `chore: ignore TypeScript build artifacts` |

### Branch Naming

Format: `{type}/{ticket}-{description}`

Example: `feat/001-event-types`

---

## What Does NOT Belong Here

- Test strategies → TESTING.md
- Security practices → SECURITY.md
- Architecture patterns → ARCHITECTURE.md
- Technology choices → STACK.md

---

*This document defines HOW to write code. Update when conventions change.*
