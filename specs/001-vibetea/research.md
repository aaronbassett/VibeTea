# Research: VibeTea

**Feature**: 001-vibetea | **Date**: 2026-02-02

This document consolidates research findings for VibeTea implementation decisions.

---

## 1. Claude Code JSONL Format

**Decision**: Parse Claude Code session files from `~/.claude/projects/<slugified-path>/<uuid>.jsonl`

**Format**: One JSON object per line with `type` field indicating event type.

**Event Types to Extract**:

| Claude Code Type | VibeTea Mapping | Fields to Extract |
|------------------|-----------------|-------------------|
| `assistant` with `tool_use` content | `tool` (started) | tool name from content block |
| `progress` with `PostToolUse` | `tool` (completed) | tool name, success status |
| `user` | `activity` | timestamp only |
| `summary` | `summary` | marks session end |
| First event in file | `session` (started) | project name from path |

**Privacy Pipeline Rules**:
- Strip all `content` fields (prompts, responses, code)
- Extract only tool names from `tool_use` blocks
- Convert full paths to basenames only
- Never transmit `command` from Bash tool use (only `description` if present)
- Strip `pattern` from Grep/Glob tool use

**Rationale**: Claude Code writes structured JSONL with clear event boundaries. The privacy pipeline operates on the parsed JSON before any network transmission.

---

## 2. Ed25519 Signing (Rust)

**Decision**: Use `ed25519-dalek` v2.1 with `zeroize` for key material protection

**Crates**:
```toml
ed25519-dalek = { version = "2.1", features = ["rand_core"] }
rand = "0.8"
zeroize = { version = "1.7", features = ["derive"] }
base64 = "0.22"
```

**Signing Pattern**:
```rust
// Sign request body, return base64 signature
let signature: Signature = signing_key.sign(body);
let signature_b64 = BASE64.encode(signature.to_bytes());
```

**Request Headers**:
- `X-Source-ID`: Monitor identifier (hostname or custom)
- `X-Signature`: Base64-encoded Ed25519 signature of request body

**Key Storage**:
- Path: `~/.vibetea/key.priv` and `~/.vibetea/key.pub`
- Permissions: 0600 on Unix
- Use `directories` crate for cross-platform paths

**Alternatives Considered**:
- HMAC-SHA256: Simpler but requires shared secrets (rejected - harder to manage)
- RSA: Larger keys, slower (rejected - Ed25519 is faster and sufficient)

---

## 3. axum WebSocket Broadcast

**Decision**: Use `tokio::sync::broadcast` channel for server-side event distribution

**Rationale**: All clients receive the same events (with optional filtering). Broadcast channel is the natural fit.

**Pattern**:
```rust
let (event_tx, _) = broadcast::channel::<Arc<Event>>(1000);

// On POST /events
let _ = state.event_tx.send(Arc::new(event));

// On WebSocket connection
let mut event_rx = state.event_tx.subscribe();
while let Ok(event) = event_rx.recv().await {
    if matches_filters(&event, &client_filters) {
        sender.send(Message::Text(serde_json::to_string(&*event)?)).await?;
    }
}
```

**Slow Client Handling**:
- `broadcast::RecvError::Lagged(n)` indicates client is too slow
- Log warning, continue (events are non-critical to catch up)
- Optionally send "lagged" notification to client

**Alternatives Considered**:
- Individual mpsc per client: More memory, complex management (rejected)
- Redis pub/sub: External dependency, overkill for single-instance (rejected)

---

## 4. File Watching (notify crate)

**Decision**: Use `notify` v6.1 with position tracking for JSONL tailing

**Crates**:
```toml
notify = "6.1"
```

**Pattern**:
1. Watch `~/.claude/projects/` recursively
2. Track file positions in `HashMap<PathBuf, u64>`
3. On `ModifyKind::Data` event, seek to last position, read new lines
4. Handle file truncation (size < last position → reset to 0)

**Session Detection**:
- New `.jsonl` file created → emit `session.started` event
- `summary` event in file → emit `session.ended`, stop watching file

**Alternatives Considered**:
- notify-debouncer-mini: Adds complexity without much benefit (rejected - handle duplicates in parsing)
- Polling: Higher CPU usage, not event-driven (rejected)

---

## 5. HTTP Client with Retry (reqwest)

**Decision**: Use `reqwest` with connection pooling and custom exponential backoff

**Crates**:
```toml
reqwest = { version = "0.12", features = ["json"] }
```

**Client Configuration**:
```rust
ClientBuilder::new()
    .pool_max_idle_per_host(2)
    .pool_idle_timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .timeout(Duration::from_secs(30))
```

**Backoff Parameters** (from spec):
- Initial: 1s
- Max: 60s
- Multiplier: 2x
- Jitter: ±25%

**Retry Conditions**:
- Network errors (timeout, connect failure)
- 429 Too Many Requests (respect `Retry-After` header)
- 5xx Server errors
- NOT 4xx client errors (except 429)

---

## 6. React State Management

**Decision**: Use **Zustand** for WebSocket state

**Rationale**:
- Context API causes re-renders for ALL consumers on any state change
- Zustand (~1KB) provides selective subscriptions - components only re-render when their slice changes
- Critical for high-frequency WebSocket events (up to 100/sec)

**Package**:
```json
"zustand": "^4.5"
```

**Store Shape**:
```typescript
interface WebSocketState {
  status: 'connecting' | 'connected' | 'disconnected' | 'reconnecting';
  events: VibeteaEvent[];  // Last 1000
  sessions: Map<string, Session>;
  addEvent: (event: VibeteaEvent) => void;
  setStatus: (status: ConnectionStatus) => void;
}
```

**Alternatives Considered**:
- React Context: Re-render issues at high event rates (rejected)
- Redux: Overkill for this use case, larger bundle (rejected)
- Jotai: Good but less ecosystem adoption than Zustand (rejected)

---

## 7. Virtualized Event List

**Decision**: Use **@tanstack/react-virtual** for event stream

**Rationale**:
- Hooks-based API, excellent TypeScript support
- Most actively maintained (TanStack ecosystem)
- Works well with auto-scroll behavior needed for event stream

**Package**:
```json
"@tanstack/react-virtual": "^3.0"
```

**Auto-scroll Logic**:
- Track if user has scrolled away from top
- If user scrolled → show "Jump to latest" button
- If user at top → auto-scroll on new events

**Alternatives Considered**:
- react-window: Less active maintenance, older API (rejected)
- react-virtuoso: Good for variable heights, but TanStack is more flexible (rejected)

---

## 8. Heatmap Visualization

**Decision**: **CSS Grid with Tailwind** (no charting library)

**Rationale**:
- 7 days × 24 hours = 168 cells (trivial for DOM)
- 30 days × 24 hours = 720 cells (still fine)
- Native hover states, click handlers, accessibility
- Zero external dependencies
- Perfect for Tailwind theming

**Color Scale** (from spec):
```css
--heatmap-0: #1a1a2e;   /* 0 events */
--heatmap-1: #2d4a3e;   /* 1-10 events */
--heatmap-2: #3d6b4f;   /* 11-25 events */
--heatmap-3: #4d8c5f;   /* 26-50 events */
--heatmap-4: #5dad6f;   /* 51+ events */
```

**Alternatives Considered**:
- D3.js: Overkill for simple grid (rejected)
- Canvas: Loses accessibility benefits (rejected)
- recharts/visx: External dependency for simple use case (rejected)

---

## 9. Vite Production Optimization

**Decision**: Vite with Brotli compression and manual chunk splitting

**Key Configuration**:
```typescript
// vite.config.ts
build: {
  target: 'es2020',
  rollupOptions: {
    output: {
      manualChunks: {
        'react-vendor': ['react', 'react-dom'],
        'state': ['zustand'],
        'virtual': ['@tanstack/react-virtual'],
      }
    }
  }
}
```

**Plugins**:
```json
"vite-plugin-compression2": "^1.0"
```

**Bundle Targets for 3G < 2s TTI**:
- Critical JS: < 50KB gzipped
- Total JS: < 150KB gzipped
- CSS: < 20KB gzipped

**Code Splitting Strategy**:
- Lazy load Statistics panel (P3 feature, non-critical)
- Lazy load Session overview (P2 feature)
- Keep Event stream and Heatmap in main bundle (core experience)

---

## 10. Shared Types

**Decision**: Define TypeScript interfaces in client, document for Rust parity

**Event Envelope** (shared contract):
```typescript
interface VibeteaEvent {
  id: string;           // "evt_" + 20 alphanumeric chars
  source: string;       // Monitor identifier
  timestamp: string;    // RFC 3339 UTC
  type: 'session' | 'activity' | 'tool' | 'agent' | 'summary' | 'error';
  payload: EventPayload;
}

interface EventPayload {
  sessionId: string;
  project?: string;
  tool?: string;
  status?: 'started' | 'completed';
  context?: string;     // File basename
  action?: 'started' | 'ended';
  summary?: string;
  state?: string;
  category?: string;
}
```

**Rationale**: TypeScript interfaces serve as documentation. Rust uses `serde` with matching field names.

---

## Summary: Technology Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Rust async | tokio | Standard, well-integrated with axum |
| HTTP server | axum | Best tokio integration, clean API |
| File watching | notify | Cross-platform, event-driven |
| Crypto | ed25519-dalek | Fast, secure, small signatures |
| HTTP client | reqwest | Full-featured, connection pooling |
| React state | Zustand | Selective re-renders, tiny bundle |
| Virtualization | @tanstack/react-virtual | Modern hooks API, active maintenance |
| Heatmap | CSS Grid | Zero dependencies, accessible |
| Build | Vite + compression | Fast builds, optimal output |

---

*Phase 0 complete. Proceed to Phase 1: Design & Contracts.*
