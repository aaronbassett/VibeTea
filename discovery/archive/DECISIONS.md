# Decision Log: vibetea

**Updated**: 2026-02-02 12:45 UTC

---

## D1: Rate Limiting Parameters

**Date**: 2026-02-02
**Question**: Q1 (Server Rate Limiting Specifics)
**Stories Affected**: Server: Event Hub Core

### Context
The PRD requires rate limiting but doesn't specify parameters. Need to define limits that prevent abuse while allowing legitimate high-activity usage.

### Decision
Implement tiered rate limiting:

| Limit | Value | Scope |
|-------|-------|-------|
| Per-source event rate | 100 events/second | Per Monitor connection |
| Burst allowance | 200 events | 1-second burst window |
| Global throughput | 1000 events/second | Entire server |

**On limit exceeded**: Return `429 Too Many Requests` with `Retry-After` header. Do not drop events silently—let the Monitor buffer and retry.

### Rationale
- 100 events/sec per source is generous (Claude Code rarely exceeds 10 events/sec even during heavy tool use)
- Burst allowance handles reconnection scenarios where buffered events are sent quickly
- Global limit prevents server overload regardless of source count
- Explicit 429 response allows Monitor to handle gracefully (buffer + backoff)

---

## D2: Authentication Model

**Date**: 2026-02-02 (Revised: 2026-02-02)
**Question**: Q2 (Server Authentication Token Lifecycle)
**Stories Affected**: Server: Event Hub Core, Monitor: Server Connection, Client: WebSocket Connection

### Context
Need to define how auth works. Options considered:
1. Single shared token (simple but no revocation granularity)
2. Per-user tokens (requires user management—out of scope for v1)
3. Per-role tokens (publisher vs subscriber)
4. **Ed25519 public/private key signing** (per-monitor identity, secure)

### Decision
**v1: Hybrid approach** with Ed25519 signing for Monitors (default) and static token for Clients:

#### Monitor Authentication (Ed25519 Signing)
- Each Monitor generates a keypair on first run (`vibetea init`)
- Monitor signs each event batch with its private key
- Server verifies signature against registered public keys
- **Overhead is negligible**: Ed25519 signs ~50,000 msgs/sec, adds 64 bytes per request

| Component | Location |
|-----------|----------|
| Private key | `~/.vibetea/key.priv` (never leaves machine) |
| Public key | `~/.vibetea/key.pub` (uploaded to server) |
| Server authorized keys | `VIBETEA_PUBLIC_KEYS` env var |

**Request format:**
```
POST /events
X-Signature: <ed25519 signature of body>
X-Source-ID: <monitor source id>
Content-Type: application/json

[events...]
```

**Server public key format:**
```
VIBETEA_PUBLIC_KEYS="source1:base64pubkey1,source2:base64pubkey2"
```

#### Client Authentication (Static Token)
- Clients use a static subscriber token (unchanged from original design)
- Lower security requirement (read-only access)

| Token | Purpose | Env Var |
|-------|---------|---------|
| Subscriber token | For Clients to receive events | `VIBETEA_SUBSCRIBER_TOKEN` |

#### Unsafe Mode (Development Only)
For local development or trusted networks, auth can be disabled entirely:

```
VIBETEA_UNSAFE_NO_AUTH=true
```

When enabled:
- Server accepts all events without signature verification
- Server accepts all WebSocket connections without token
- Logs warning on startup: "⚠️ Running in unsafe mode - authentication disabled"

### Rationale
- **Ed25519 over shared tokens**: Per-monitor identity enables revocation without affecting other monitors
- **Negligible overhead**: ~0.02ms to sign, ~0.07ms to verify, 64 bytes per request
- **Private key never transmitted**: Only public key is shared with server
- **Unsafe mode for dev**: Eliminates auth friction during local development
- **Client keeps simple token**: Read-only access doesn't need same security level
- **All config in env vars**: Consistent with 12-factor app principles

---

## D3: Monitor Reconnection Parameters

**Date**: 2026-02-02
**Question**: Q7 (Monitor Reconnection Behavior)
**Stories Affected**: Monitor: Server Connection

### Context
Need to define exponential backoff parameters for server reconnection.

### Decision

| Parameter | Value |
|-----------|-------|
| Initial delay | 1 second |
| Max delay | 60 seconds |
| Backoff multiplier | 2x |
| Jitter | ±25% randomization |
| Max retries | Unlimited (never give up) |

**Buffer overflow behavior**: When the 1000-event buffer fills during extended outage:
- Drop oldest events first (FIFO eviction)
- Log warning to stderr when buffer reaches 80% capacity
- Continue accepting new events (never block the file watcher)

### Rationale
- 1s initial delay balances quick recovery with avoiding thundering herd
- 60s max prevents excessive delay after prolonged outage
- Jitter prevents synchronized reconnection from multiple monitors
- Unlimited retries—Monitor should always try to reconnect
- FIFO eviction ensures most recent events are preserved (more valuable)

---

## D4: Sensitive Filename Handling

**Date**: 2026-02-02
**Question**: Q6 (Privacy Pipeline Edge Cases)
**Stories Affected**: Monitor: Privacy Pipeline

### Context
File basenames could contain sensitive information (e.g., `api-keys-backup.json`, `password-list.txt`).

### Decision
**Transmit basenames as-is** with the following rationale:
1. Basenames rarely contain actual secrets (the contents do)
2. Filtering basenames would reduce utility significantly
3. Users who name files sensitively accept some metadata exposure
4. Project names are already transmitted (same privacy model)

**However**, add an optional allowlist mode:
- `VIBETEA_BASENAME_ALLOWLIST` env var (comma-separated extensions)
- If set, only transmit basenames with listed extensions
- Default: empty (all basenames transmitted)
- Example: `VIBETEA_BASENAME_ALLOWLIST=".ts,.js,.rs,.py,.md"`

### Rationale
- Default behavior is useful and matches user expectations
- Allowlist provides escape hatch for privacy-conscious users
- Consistent with "project name is transmitted" precedent

---

## D5: Client Authentication UX

**Date**: 2026-02-02
**Question**: Q8 (Client Authentication UX)
**Stories Affected**: Client: WebSocket Connection

### Context
Need to define how users authenticate to the dashboard.

### Decision
1. **Primary**: Token in URL query parameter (as PRD specifies): `https://dashboard.vibetea.dev/?token=xxx`
2. **Persistence**: Store token in localStorage after first successful connection
3. **UI**: Show token input form if no token in URL and none in localStorage
4. **Invalid token**: Show clear error message, clear localStorage, show input form

### Rationale
- URL token allows easy bookmarking/sharing of authenticated dashboard
- localStorage persistence improves UX for returning users
- Simple form fallback for users who arrive without token

---

## D6: Session "Active" Definition

**Date**: 2026-02-02
**Question**: Q12 (Session "Active" Definition)
**Stories Affected**: Client: Session Overview

### Context
Need to define when a session is considered "active" for the session cards.

### Decision
A session is **active** if:
- Has received at least one event in the last 5 minutes
- OR has not received a `summary` event (explicit session end)

A session becomes **inactive** when:
- No events for 5 minutes AND no explicit session end
- Display with "Last active: X minutes ago"

**Display rules**:
- Show all active sessions
- Show inactive sessions for 30 minutes after last event
- After 30 minutes, remove from display entirely

### Rationale
- 5 minutes covers typical "thinking" pauses during coding
- 30-minute retention allows seeing recently ended sessions
- `summary` event provides explicit end signal when available

---

## D7: Heatmap Specifications

**Date**: 2026-02-02
**Questions**: Q10 (Time Granularity), Q11 (Color Scale)
**Stories Affected**: Client: Activity Heatmap

### Context
Need to specify heatmap visual parameters.

### Decision

**Time handling**:
- Display in user's local timezone (browser's `Intl` API)
- Each cell = 1 hour (fixed, not configurable in v1)
- Default view: 7 days
- Expandable to: 30 days

**Color scale** (5 levels, like GitHub):
| Level | Event Count | Color |
|-------|-------------|-------|
| 0 | 0 events | `#1a1a2e` (empty) |
| 1 | 1-10 events | `#2d4a3e` |
| 2 | 11-25 events | `#3d6b4f` |
| 3 | 26-50 events | `#4d8c5f` |
| 4 | 51+ events | `#5dad6f` |

**Thresholds are absolute**, not relative to visible range.

### Rationale
- Local timezone matches user mental model
- Fixed hour granularity keeps implementation simple
- Absolute thresholds provide consistent visual language across sessions
- GitHub-style 5 levels is familiar and effective

---

## D8: Statistics Time Periods

**Date**: 2026-02-02
**Question**: Q13 (Statistics Time Period)
**Stories Affected**: Client: Statistics Panel

### Context
Need to define what time periods are available for statistics.

### Decision
**Preset periods** (no custom range in v1):
- Last hour
- Last 24 hours (default)
- Last 7 days
- Last 30 days

**Behavior**:
- Statistics are calculated client-side from received events
- Since v1 has no persistence, only events received since page load are included
- Display clear disclaimer: "Statistics based on events since [page load time]"

### Rationale
- Preset periods cover common use cases
- Client-side calculation aligns with "no persistence" constraint
- Clear disclaimer sets correct expectations

---

## D9: Event Stream Auto-Scroll Behavior

**Date**: 2026-02-02
**Question**: Q9 (Event Stream Auto-Scroll Behavior)
**Stories Affected**: Client: Live Event Stream

### Context
Need to define when auto-scroll pauses and resumes for the live event stream.

### Decision

| Behavior | Trigger |
|----------|---------|
| Pause auto-scroll | User scrolls more than 50px from bottom |
| Show "Jump to latest" button | When auto-scroll is paused |
| Resume auto-scroll (button) | User clicks "Jump to latest" |
| Resume auto-scroll (manual) | User scrolls to within 50px of bottom |

**No timeout-based resume**—user must explicitly return to bottom.

### Rationale
- 50px threshold prevents accidental pause from minor scroll adjustments
- Explicit resume (button or scroll) gives user full control
- No timeout avoids jarring UX where stream suddenly jumps while user is reading
- "Jump to latest" button provides clear affordance for returning to live view

---
