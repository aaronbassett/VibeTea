# Implementation Plan: Supabase Persistence Layer

**Branch**: `001-supabase-persistence` | **Date**: 2026-02-03 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-supabase-persistence/spec.md`

## Summary

Add optional persistence via Supabase to enable historic activity heatmaps spanning 7-30 days. The monitor batches and forwards events to Supabase edge functions using existing Ed25519 authentication. The client queries pre-aggregated hourly event counts for heatmap display. All database access occurs through authenticated edge functions with the database locked down via RLS policies.

## Technical Context

**Phase 0 Research**: See [research.md](./research.md) for detailed decisions on:
- Ed25519 library choice (`@noble/ed25519` for Deno compatibility)
- RLS configuration pattern (enable with no policies = implicit deny-all)
- Batch insert approach (`jsonb_populate_recordset` via RPC for atomic transactions)
- Deno testing strategy (built-in test framework with mocked Supabase client)
- Cache invalidation (5-minute staleness window with refresh on mount)

**Language/Version**:
- Rust 2021 edition (1.85+) for monitor batch submission
- TypeScript 5.9 for client queries
- TypeScript (Deno 1.40+) for Supabase edge functions

**Primary Dependencies**:
- Monitor: `reqwest` (HTTP client, already used), `serde_json` (serialization)
- Client: `@supabase/supabase-js` or direct fetch API for edge function calls
- Edge Functions: `@noble/ed25519` v2.0.0+ for signature verification, Supabase service role for DB access

**Storage**: Supabase PostgreSQL with single `events` table

**Testing**:
- Rust: `cargo test` with mocked HTTP responses
- TypeScript: `vitest` with MSW for mocking edge function responses
- Edge Functions: Deno test framework

**Target Platform**:
- Monitor: Linux/macOS/Windows (existing)
- Client: Modern browsers (existing)
- Edge Functions: Deno Deploy (Supabase)

**Project Type**: Multi-component (monitor, client, edge functions)

**Performance Goals**:
- Historic data visible within 5 seconds of page load
- 95%+ event persistence under normal network conditions
- Zero degradation to real-time WebSocket streaming

**Constraints**:
- Maximum batch size: 1000 events
- Batch interval: 60 seconds (configurable)
- Edge function timeout: 30 seconds (Supabase limit)

**Scale/Scope**:
- Single tenant (all events in one table)
- Up to 30 days of historic data

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Privacy by Design (I) - NON-NEGOTIABLE
- **Status**: ✅ COMPLIANT
- Events persisted to Supabase are already privacy-filtered by the monitor's `PrivacyPipeline`
- No additional sensitive data introduced - same event structure as real-time
- Edge functions only return aggregated counts, not raw event payloads to clients
- **Enforcement**: Verify edge function query returns only `{ date, hour, count }` aggregates

### Unix Philosophy (II)
- **Status**: ✅ COMPLIANT
- Persistence is optional and decoupled - system works without it
- Monitor monitors, server serves, edge functions handle persistence
- Clear separation: Monitor → Edge Function → Database; Client → Edge Function → Aggregates

### Keep It Simple (III)
- **Status**: ✅ COMPLIANT
- Single `events` table, no complex schema
- Pre-aggregated hourly counts avoid complex client-side aggregation
- No ORM or query builder - direct SQL in edge functions
- **Risk**: Adding Supabase adds complexity, but it's isolated and optional

### Event-Driven Communication (IV)
- **Status**: ✅ COMPLIANT
- Monitor batches events locally, sends asynchronously
- Persistence failures don't block real-time event flow
- Edge functions are request/response but events are still the source of truth

### Test What Matters (V)
- **Status**: ✅ COMPLIANT
- Test persistence paths (batch submission, query aggregation)
- Test authentication (signature verification, token validation)
- Mock edge function responses for unit tests

### Fail Fast & Loud (VI)
- **Status**: ✅ COMPLIANT
- Clear error messages when Supabase is unavailable
- Heatmap error states:
  - **Loading**: "Fetching historic data..." (5-second timeout)
  - **Error**: "Unable to load historic data. Showing real-time events only." with Retry button
  - **Timeout**: After 5s, shows error state (non-blocking, falls back to real-time)
- Monitor logs persistence failures at WARN level with: event count, source ID, retry attempt, error message
- Error format: JSON via tracing crate (consistent with SECURITY.md audit logging)

### Modularity & Clear Boundaries (VII)
- **Status**: ✅ COMPLIANT
- Persistence is feature-flagged via environment variables
- New code isolated: `monitor/src/persistence.rs`, `client/src/hooks/useHistoricData.ts`
- Edge functions are separate deployable units

## Project Structure

### Documentation (this feature)

```text
specs/001-supabase-persistence/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (OpenAPI/edge function schemas)
│   ├── ingest.yaml      # POST /ingest endpoint schema
│   └── query.yaml       # GET /query endpoint schema
├── checklists/          # Feature-specific checklists
└── tasks.md             # Phase 2 output (not created by /sdd:plan)
```

### Source Code (repository root)

```text
# Monitor additions
monitor/
├── src/
│   ├── persistence.rs   # NEW: Batch submission to Supabase edge function
│   ├── lib.rs           # ADD: pub mod persistence
│   └── main.rs          # MODIFY: Initialize persistence if configured

# Client additions
client/
├── src/
│   ├── hooks/
│   │   └── useHistoricData.ts  # NEW: Fetch and cache historic aggregates
│   ├── components/
│   │   └── Heatmap.tsx         # MODIFY: Merge historic + real-time data
│   └── types/
│       └── events.ts           # ADD: HistoricAggregate type

# Supabase additions (new directory)
supabase/
├── functions/
│   ├── ingest/
│   │   └── index.ts     # Edge function: validate signature, insert events
│   └── query/
│       └── index.ts     # Edge function: validate token, return aggregates
├── migrations/
│   └── 20260203_create_events_table.sql  # Initial schema
└── config.toml          # Supabase project configuration

# Infrastructure
fly.toml                 # Already exists - no changes needed
```

**Structure Decision**: Extends existing multi-component structure (monitor, client) with new Supabase directory for edge functions and migrations. Monitor handles batch submission; client handles historic data queries; edge functions mediate all database access.

## Complexity Tracking

No constitution violations requiring justification. The feature adds complexity but:
1. It's opt-in (disabled by default)
2. Isolated to new modules
3. Uses existing authentication patterns

## Learnings from Previous Features

From retrospectives (P8, P9, P10):

### Patterns to Reuse
- **Memoization with useMemo**: Use for event counting and aggregation (P9)
- **Selective Zustand subscriptions**: Prevent re-renders during high-frequency updates (P8)
- **Pure function calculations**: Avoid Date.now() in useMemo; use reference timestamps (P10)
- **Graceful loading states**: Show loading indicator while fetching, fallback to real-time only on error (P8)

### Packages Worth Using
- No additional packages needed for client - use native fetch API
- Edge functions use standard Deno APIs

### Known Issues to Avoid
- **TypeScript discriminated union narrowing**: May need explicit type assertions for aggregate payloads
- Don't use `Date.now()` in render functions - pass time as parameter or use event timestamps

---

## Phase 0: Research

### Research Tasks

1. **Supabase Edge Function Auth**: How to verify Ed25519 signatures in Deno
   - Library: `@noble/ed25519` or native Web Crypto API
   - Pattern for extracting headers and verifying

2. **Supabase RLS Configuration**: How to configure "deny all" policies
   - Ensure service role bypasses RLS in edge functions
   - Verify anonymous/anon key cannot access data

3. **Batch Insert Performance**: Best practice for bulk event insertion
   - PostgreSQL `INSERT ... ON CONFLICT` vs batch insert
   - Transaction handling for atomicity

4. **Deno Testing**: How to unit test edge functions locally
   - Mocking Supabase client
   - Running functions with `deno test`

5. **Client Query Caching**: Strategy for caching historic data
   - Cache in Zustand store vs separate cache
   - Invalidation strategy (time-based, refresh on mount)

### Output
Generate `research.md` with decisions and rationale for each area.

---

## Phase 1: Design & Contracts

### Data Model

**events table** (PostgreSQL):
```sql
CREATE TABLE events (
  id TEXT PRIMARY KEY,           -- Event ID (evt_xxx format)
  source TEXT NOT NULL,          -- Monitor source ID
  timestamp TIMESTAMPTZ NOT NULL, -- Event timestamp
  event_type TEXT NOT NULL,      -- session, activity, tool, agent, summary, error
  payload JSONB NOT NULL,        -- Full event payload (already privacy-filtered)
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for time-range queries
CREATE INDEX idx_events_timestamp ON events (timestamp DESC);

-- Index for source filtering (if needed in future)
CREATE INDEX idx_events_source ON events (source);

-- RLS: Deny all direct access
ALTER TABLE events ENABLE ROW LEVEL SECURITY;

-- No policies = service role only access
```

**Hourly Aggregate** (returned by query endpoint, not stored):
```typescript
interface HourlyAggregate {
  source: string;
  date: string;      // YYYY-MM-DD
  hour: number;      // 0-23
  eventCount: number;
}
```

### API Contracts

**POST /ingest** (Edge Function):
- Headers: `X-Source-ID`, `X-Signature`, `Content-Type: application/json`
- Body: Array of Event objects (max 1000)
- Response: `{ inserted: number }` or error

**GET /query** (Edge Function):
- Headers: `Authorization: Bearer <token>`
- Query params: `days=7|30`, `source=optional`
- Response: Array of HourlyAggregate objects

### Artifacts to Generate
- `data-model.md`: Full schema documentation
- `contracts/ingest.yaml`: OpenAPI spec for ingest endpoint
- `contracts/query.yaml`: OpenAPI spec for query endpoint
- `quickstart.md`: Local development setup for edge functions

---

## Phase 2: Local Development Environment

### Supabase Local Development

1. **Supabase CLI Installation**:
   ```bash
   npm install -g supabase
   # or via Homebrew: brew install supabase/tap/supabase
   ```

2. **Project Initialization**:
   ```bash
   cd /home/ubuntu/Projects/VibeTea
   supabase init
   supabase start  # Starts local PostgreSQL + edge functions runtime
   ```

3. **Edge Function Development**:
   ```bash
   supabase functions new ingest
   supabase functions new query
   supabase functions serve  # Hot reload for local development
   ```

4. **Migration Management**:
   ```bash
   supabase migration new create_events_table
   supabase db push  # Apply migrations to local DB
   ```

### Testing Setup

1. **Rust/Monitor Tests**:
   - Use `mockito` or `wiremock` to mock edge function responses
   - Test batch queueing, retry logic, signature inclusion

2. **TypeScript/Client Tests**:
   - Use MSW to mock fetch calls to edge functions
   - Test loading states, error handling, data merging

3. **Edge Function Tests**:
   - Deno test framework with mocked Supabase client
   - Test signature verification, token validation, query logic

### CI Integration

Add to GitHub Actions workflow:
```yaml
# New job for edge function tests
test-edge-functions:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: denoland/setup-deno@v2
    - run: deno test supabase/functions/

# Existing jobs remain unchanged
```

---

## Stop Point

This plan completes through Phase 2 (development environment setup). The `/sdd:tasks` command will generate implementation tasks based on this plan.

**Summary of artifacts to be generated:**
1. `research.md` - Decisions on Ed25519 in Deno, RLS config, batch inserts, testing
2. `data-model.md` - PostgreSQL schema, TypeScript types
3. `contracts/ingest.yaml` - OpenAPI for batch ingestion
4. `contracts/query.yaml` - OpenAPI for historic queries
5. `quickstart.md` - Local Supabase setup instructions

**Key implementation phases (for tasks.md):**
1. Database schema and migrations
2. Edge function: ingest (signature verification, batch insert)
3. Edge function: query (token validation, aggregation)
4. Monitor: persistence module (batching, submission)
5. Client: useHistoricData hook and Heatmap integration
6. Testing across all components
7. Documentation and deployment guide
