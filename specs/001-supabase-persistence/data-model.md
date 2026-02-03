# Data Model: Supabase Persistence Layer

**Feature**: 001-supabase-persistence
**Date**: 2026-02-03

## Overview

The persistence layer stores privacy-filtered events from monitors and provides hourly aggregates for historic heatmap visualization. All access is mediated through edge functions using the service role key.

## Database Schema

### events Table

```sql
-- Main events table for storing privacy-filtered events
CREATE TABLE public.events (
  -- Event identifier (format: evt_<20-char-suffix>)
  id TEXT PRIMARY KEY,

  -- Monitor source identifier
  source TEXT NOT NULL,

  -- Event timestamp (when the event occurred)
  timestamp TIMESTAMPTZ NOT NULL,

  -- Event type discriminator
  event_type TEXT NOT NULL CHECK (event_type IN (
    'session', 'activity', 'tool', 'agent', 'summary', 'error'
  )),

  -- Full event payload (already privacy-filtered by monitor)
  payload JSONB NOT NULL,

  -- When the event was persisted (for debugging/auditing)
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient time-range queries (DESC for newest-first)
CREATE INDEX idx_events_timestamp ON public.events (timestamp DESC);

-- Index for source filtering (optional, for future multi-tenant queries)
CREATE INDEX idx_events_source ON public.events (source);

-- Composite index for common query pattern (source + time range)
CREATE INDEX idx_events_source_timestamp ON public.events (source, timestamp DESC);

-- Enable Row Level Security (implicit deny-all without policies)
ALTER TABLE public.events ENABLE ROW LEVEL SECURITY;

-- Force RLS for table owners (defense in depth)
ALTER TABLE public.events FORCE ROW LEVEL SECURITY;

-- No policies = service_role only access
```

### Bulk Insert Function

```sql
-- Function for atomic batch insertion of events
-- Called via Supabase RPC from the ingest edge function
CREATE OR REPLACE FUNCTION public.bulk_insert_events(events_json JSONB)
RETURNS TABLE(inserted_count BIGINT) AS $$
BEGIN
  RETURN QUERY
  WITH inserted AS (
    INSERT INTO public.events (id, source, timestamp, event_type, payload)
    SELECT
      (e->>'id')::TEXT,
      (e->>'source')::TEXT,
      (e->>'timestamp')::TIMESTAMPTZ,
      (e->>'eventType')::TEXT,
      e->'payload'
    FROM jsonb_array_elements(events_json) AS e
    ON CONFLICT (id) DO NOTHING
    RETURNING 1
  )
  SELECT COUNT(*)::BIGINT FROM inserted;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Grant execute permission to service role
GRANT EXECUTE ON FUNCTION public.bulk_insert_events(JSONB) TO service_role;
```

### Aggregation Query Function

```sql
-- Function for retrieving hourly event aggregates
-- Called via Supabase RPC from the query edge function
CREATE OR REPLACE FUNCTION public.get_hourly_aggregates(
  days_back INTEGER DEFAULT 7,
  source_filter TEXT DEFAULT NULL
)
RETURNS TABLE(
  source TEXT,
  date DATE,
  hour INTEGER,
  event_count BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT
    e.source,
    DATE(e.timestamp AT TIME ZONE 'UTC') AS date,
    EXTRACT(HOUR FROM e.timestamp AT TIME ZONE 'UTC')::INTEGER AS hour,
    COUNT(*)::BIGINT AS event_count
  FROM public.events e
  WHERE
    e.timestamp >= NOW() - (days_back || ' days')::INTERVAL
    AND (source_filter IS NULL OR e.source = source_filter)
  GROUP BY e.source, DATE(e.timestamp AT TIME ZONE 'UTC'), EXTRACT(HOUR FROM e.timestamp AT TIME ZONE 'UTC')
  ORDER BY date DESC, hour DESC;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Grant execute permission to service role
GRANT EXECUTE ON FUNCTION public.get_hourly_aggregates(INTEGER, TEXT) TO service_role;
```

## TypeScript Types

### Event (for batch submission)

```typescript
// Matches existing Event type from monitor/client
// Already defined in client/src/types/events.ts
interface Event {
  id: string;           // evt_<20-char-suffix>
  source: string;       // Monitor identifier
  timestamp: string;    // RFC 3339 format
  type: EventType;      // 'session' | 'activity' | 'tool' | 'agent' | 'summary' | 'error'
  payload: EventPayload;
}
```

### HourlyAggregate (for query response)

```typescript
// Add to client/src/types/events.ts
interface HourlyAggregate {
  source: string;       // Monitor identifier
  date: string;         // YYYY-MM-DD format
  hour: number;         // 0-23 (UTC)
  eventCount: number;   // Count of events in this hour
}
```

### Persistence Configuration (Monitor)

```typescript
// For monitor/src/persistence.rs (Rust types)
struct PersistenceConfig {
  supabase_url: String,         // Edge function base URL
  batch_interval_secs: u64,     // Default: 60
  max_batch_size: usize,        // Default: 1000
}
```

### Store Extensions (Client)

```typescript
// Extensions to useEventStore for historic data
interface EventStoreHistoricExtension {
  // Historic data state
  historicData: HourlyAggregate[];
  historicDataStatus: 'idle' | 'loading' | 'error' | 'success';
  historicDataFetchedAt: number | null;
  historicDataError: string | null;

  // Actions
  fetchHistoricData: (days: 7 | 30) => Promise<void>;
  clearHistoricData: () => void;
}
```

## Data Flow

### Ingest Flow (Monitor → Supabase)

```
Monitor
  │
  ├─ Events generated by watcher
  │
  ├─ Privacy pipeline filters sensitive data
  │
  ├─ Events queued in persistence buffer
  │
  ├─ Every 60s (or max 1000 events):
  │     │
  │     ├─ Serialize events to JSON array
  │     │
  │     ├─ Sign JSON body with Ed25519 private key
  │     │
  │     ├─ POST to /ingest edge function
  │     │     Headers: X-Source-ID, X-Signature
  │     │     Body: JSON array of events
  │     │
  │     └─ Handle response (retry on failure)
  │
  └─ Continue real-time streaming (independent)
```

### Query Flow (Client → Supabase)

```
Client (Heatmap component)
  │
  ├─ Check if persistence is enabled (VITE_SUPABASE_URL set)
  │
  ├─ If disabled: Hide heatmap card entirely
  │
  ├─ If enabled:
  │     │
  │     ├─ useHistoricData hook triggers fetch
  │     │
  │     ├─ GET /query edge function
  │     │     Headers: Authorization: Bearer <token>
  │     │     Query: ?days=7 or ?days=30
  │     │
  │     ├─ Edge function validates token
  │     │
  │     ├─ Edge function queries get_hourly_aggregates()
  │     │
  │     └─ Returns HourlyAggregate[] to client
  │
  ├─ Store aggregates in Zustand
  │
  └─ Heatmap merges historic + real-time data
```

## Validation Rules

### Event Validation (Ingest)

| Field | Rule | Error |
|-------|------|-------|
| id | Non-empty, starts with `evt_` | `invalid_event_id` |
| source | Non-empty, matches X-Source-ID header | `source_mismatch` |
| timestamp | Valid RFC 3339 format | `invalid_timestamp` |
| event_type | One of allowed values | `invalid_event_type` |
| payload | Valid JSON object | `invalid_payload` |

### Query Validation

| Parameter | Rule | Default |
|-----------|------|---------|
| days | 7 or 30 | 7 |
| source | Optional, non-empty if provided | null (all sources) |
| Authorization | Valid bearer token | Required |

## Retention

The specification does not require automatic data retention/cleanup. For production, consider:

```sql
-- Optional: Create retention policy (run periodically)
DELETE FROM public.events
WHERE timestamp < NOW() - INTERVAL '90 days';
```

This could be implemented as a scheduled Supabase function or external cron job.

## Migration Strategy

### Initial Migration (20260203_create_events_table.sql)

```sql
-- Migration: Create events table and functions
-- Version: 001
-- Date: 2026-02-03

-- Create events table
CREATE TABLE IF NOT EXISTS public.events (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL CHECK (event_type IN (
    'session', 'activity', 'tool', 'agent', 'summary', 'error'
  )),
  payload JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON public.events (timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_events_source ON public.events (source);
CREATE INDEX IF NOT EXISTS idx_events_source_timestamp ON public.events (source, timestamp DESC);

-- Enable RLS
ALTER TABLE public.events ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.events FORCE ROW LEVEL SECURITY;

-- Create functions (see above for full definitions)
-- bulk_insert_events(JSONB)
-- get_hourly_aggregates(INTEGER, TEXT)
```

### Applying Migrations

```bash
# Local development
supabase db push

# Production (via Supabase dashboard or CLI)
supabase db push --linked
```
