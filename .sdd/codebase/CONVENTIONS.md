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
| rustfmt (Rust/Server/Monitor) | Default settings | `cargo fmt` |
| clippy (Rust/Server/Monitor) | Default lints | `cargo clippy` |

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

#### Rust/Server/Monitor

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

### Rust/Server/Monitor

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `config.rs`, `error.rs`, `types.rs` |
| Types | PascalCase | `Config`, `Event`, `ServerError`, `MonitorError` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT`, `DEFAULT_BUFFER_SIZE` |
| Test modules | `#[cfg(test)] mod tests` | In same file as implementation |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Functions | snake_case | `from_env()`, `generate_event_id()` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT = 8080` |
| Structs | PascalCase | `Config`, `Event` |
| Enums | PascalCase | `EventType`, `SessionAction`, `ServerError` |
| Methods | snake_case | `.new()`, `.to_string()`, `.from_env()` |
| Lifetimes | Single lowercase letter | `'a`, `'static` |

## Error Handling

### Error Patterns

#### TypeScript (Not yet implemented)

Client error handling will use:
- Try/catch for async operations
- Zod for schema validation with `.parse()` and `.safeParse()`
- Custom error classes for business logic errors

#### Rust/Server

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Configuration errors | Custom enum with `#[derive(Error)]` | `server/src/config.rs` defines `ConfigError` |
| Authentication errors | String-based variant in `ServerError` | `ServerError::Auth(String)` |
| Validation errors | String-based variant in `ServerError` | `ServerError::Validation(String)` |
| Rate limiting | Struct variant with fields | `ServerError::RateLimit { source, retry_after }` |
| WebSocket errors | String-based variant in `ServerError` | `ServerError::WebSocket(String)` |
| Internal errors | String-based variant in `ServerError` | `ServerError::Internal(String)` |

#### Rust/Monitor

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Configuration errors | Custom enum with `#[derive(Error)]` | `monitor/src/config.rs` defines `ConfigError` |
| I/O errors | Use `#[from]` for automatic conversion | `MonitorError::Io(#[from] std::io::Error)` |
| JSON errors | Automatic conversion via `serde_json` | `MonitorError::Json(#[from] serde_json::Error)` |
| HTTP errors | String-based variants | `MonitorError::Http(String)` |
| Cryptographic errors | String-based variants | `MonitorError::Crypto(String)` |
| File watching errors | String-based variants | `MonitorError::Watch(String)` |

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

**Server Example** (`server/src/error.rs`):

```rust
#[derive(Debug)]
pub enum ServerError {
    Config(ConfigError),
    Auth(String),
    Validation(String),
    RateLimit { source: String, retry_after: u64 },
    WebSocket(String),
    Internal(String),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(err) => write!(f, "configuration error: {err}"),
            Self::Auth(msg) => write!(f, "authentication failed: {msg}"),
            // ... other variants
        }
    }
}
```

**Monitor Example** (`monitor/src/error.rs`):

```rust
#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Logging Conventions

Not yet implemented in codebase. When logging is added:

| Level | When to Use | Example |
|-------|-------------|---------|
| error | Unrecoverable failures | Configuration load failure, server startup error |
| warn | Recoverable issues | Retry attempt, unsafe auth mode enabled |
| info | Important events | Server started, session created |
| debug | Development details | Event payload serialization, connection established |

## Common Patterns

### Event-Driven Architecture

The codebase uses discriminated unions for type-safe event handling:

#### TypeScript

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

#### Rust

```rust
// Rust equivalent with untagged enums
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    Tool { session_id: Uuid, tool: String, status: ToolStatus, ... },
    Session { session_id: Uuid, action: SessionAction, project: String },
    Activity { session_id: Uuid, project: Option<String> },
    // ... other variants, ordered from most specific to least specific
}

// Tagged wrapper for the full event
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub payload: EventPayload,
}
```

### Zustand Store Pattern (TypeScript)

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

**Server Example** (`server/src/config.rs`):

```rust
pub struct Config {
    pub public_keys: HashMap<String, String>,
    pub subscriber_token: Option<String>,
    pub port: u16,
    pub unsafe_no_auth: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = parse_port()?;
        let public_keys = parse_public_keys()?;
        let subscriber_token = env::var("VIBETEA_SUBSCRIBER_TOKEN").ok();
        let unsafe_no_auth = parse_bool_env("VIBETEA_UNSAFE_NO_AUTH");

        let config = Self { public_keys, subscriber_token, port, unsafe_no_auth };
        config.validate()?;
        Ok(config)
    }
}
```

**Monitor Example** (`monitor/src/config.rs`):

```rust
pub struct Config {
    pub server_url: String,
    pub source_id: String,
    pub key_path: PathBuf,
    pub claude_dir: PathBuf,
    pub buffer_size: usize,
    pub basename_allowlist: Option<Vec<String>>,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_url = env::var("VIBETEA_SERVER_URL")
            .map_err(|_| ConfigError::MissingEnvVar("VIBETEA_SERVER_URL".into()))?;
        // ... parse other vars with defaults
    }
}
```

### Error Handling Pattern (Rust)

Create typed error enums with helper constructors:

```rust
impl ServerError {
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn rate_limit(source: impl Into<String>, retry_after: u64) -> Self {
        Self::RateLimit {
            source: source.into(),
            retry_after,
        }
    }

    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::Auth(_) | Self::Validation(_) | Self::RateLimit { .. }
        )
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

Example from `server/src/error.rs`:

```rust
use std::error::Error;
use std::fmt;

use thiserror::Error as ThisError;
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
| Errors section | Fallible functions | `/// # Errors` section |

Example from `server/src/config.rs`:

```rust
/// Server configuration parsed from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Map of source_id to base64-encoded Ed25519 public key.
    pub public_keys: HashMap<String, String>,

    /// Authentication token for subscriber clients.
    pub subscriber_token: Option<String>,

    /// HTTP server port.
    pub port: u16,
}

impl Config {
    /// Parse configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if:
    /// - Required environment variables are missing
    /// - Environment variables have invalid format
    /// - Port number is not a valid u16
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_server::config::Config;
    ///
    /// let config = Config::from_env().expect("Failed to load config");
    /// println!("Server will listen on port {}", config.port);
    /// ```
    pub fn from_env() -> Result<Self, ConfigError> {
        // ...
    }
}
```

## Git Conventions

### Commit Messages

Format: `type(scope): description`

| Type | Usage | Example |
|------|-------|---------|
| feat | New feature | `feat(client): add event store` |
| fix | Bug fix | `fix(server): handle missing env var` |
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
