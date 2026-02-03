# Research: Supabase Persistence Layer

**Feature**: 001-supabase-persistence
**Date**: 2026-02-03

## 1. Ed25519 Signature Verification in Deno

### Decision
Use `@noble/ed25519` library for Ed25519 signature verification in Supabase Edge Functions.

### Rationale
- `@noble/ed25519` is a well-audited, zero-dependency library that works across all JavaScript runtimes including Deno
- It's the same library family used by many crypto projects and has been security-audited
- Web Crypto API in Deno supports Ed25519 but `@noble/ed25519` provides a simpler API

### Implementation Pattern

```typescript
import * as ed from "https://esm.sh/@noble/ed25519@2.0.0";

async function verifySignature(
  publicKeyBase64: string,
  signatureBase64: string,
  message: Uint8Array
): Promise<boolean> {
  const publicKey = Uint8Array.from(atob(publicKeyBase64), c => c.charCodeAt(0));
  const signature = Uint8Array.from(atob(signatureBase64), c => c.charCodeAt(0));

  return await ed.verifyAsync(signature, message, publicKey);
}

// In edge function handler:
const sourceId = req.headers.get("X-Source-ID");
const signature = req.headers.get("X-Signature");
const body = await req.text();

const publicKey = await getPublicKeyForSource(sourceId); // from env or DB
const isValid = await verifySignature(publicKey, signature, new TextEncoder().encode(body));

if (!isValid) {
  return new Response(JSON.stringify({ error: "Invalid signature" }), { status: 401 });
}
```

### Alternatives Considered
- **Web Crypto API directly**: More complex API, requires key import steps
- **tweetnacl**: Older library, less maintained than noble

---

## 2. Supabase Row Level Security (RLS) Configuration

### Decision
Enable RLS with no policies (implicit deny-all) and use service role key in edge functions.

### Rationale
- When RLS is enabled without any policies, PostgreSQL denies all access by default
- The `service_role` key has the `BYPASSRLS` attribute, allowing edge functions full access
- This ensures the database is completely locked down from direct client access

### Implementation

```sql
-- Enable RLS (creates implicit deny-all)
ALTER TABLE public.events ENABLE ROW LEVEL SECURITY;

-- Force RLS even for table owners (defense in depth)
ALTER TABLE public.events FORCE ROW LEVEL SECURITY;

-- NO policies needed - service_role bypasses RLS automatically
```

### Edge Function Access Pattern

```typescript
import { createClient } from "https://esm.sh/@supabase/supabase-js@2";

// Service role client bypasses ALL RLS
const supabase = createClient(
  Deno.env.get("SUPABASE_URL")!,
  Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!
);

// Full access regardless of RLS
const { data, error } = await supabase.from("events").insert(events);
```

### Verification Steps
1. Test with anon key: `curl` should return empty array or 403
2. Test with service_role key: Should return data
3. SQL check: `SELECT rowsecurity FROM pg_tables WHERE tablename = 'events'` should be `true`

---

## 3. Batch Insert Performance

### Decision
Use PostgreSQL function with `jsonb_populate_recordset` called via Supabase RPC.

### Rationale
- Single network round-trip for up to 1000 events
- Atomic transaction (all-or-nothing)
- `ON CONFLICT DO NOTHING` handles duplicate event IDs gracefully
- Returns count of actually inserted rows

### Implementation

```sql
CREATE OR REPLACE FUNCTION bulk_insert_events(events_json jsonb)
RETURNS TABLE(inserted_count bigint) AS $$
BEGIN
  RETURN QUERY
  WITH inserted AS (
    INSERT INTO events (id, source, timestamp, event_type, payload)
    SELECT
      (e->>'id')::text,
      (e->>'source')::text,
      (e->>'timestamp')::timestamptz,
      (e->>'eventType')::text,
      e->'payload'
    FROM jsonb_array_elements(events_json) AS e
    ON CONFLICT (id) DO NOTHING
    RETURNING 1
  )
  SELECT count(*)::bigint FROM inserted;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
```

### Client Call

```typescript
const { data, error } = await supabase.rpc("bulk_insert_events", {
  events_json: eventsArray
});
// data = [{ inserted_count: 847 }] (if 153 were duplicates)
```

### Performance Notes
- Batch size of 1000 is well within Supabase limits (5MB payload max)
- Statement timeout: 8 seconds default, sufficient for 1000 rows
- Index on `timestamp` may slow inserts slightly but is necessary for queries

---

## 4. Deno Edge Function Testing

### Decision
Use Deno's built-in test framework with mocked Supabase client.

### Rationale
- No external test runner needed
- Mocking via dependency injection keeps tests isolated
- Supabase CLI supports local function testing

### Implementation Pattern

```typescript
// ingest/index.test.ts
import { assertEquals } from "https://deno.land/std@0.208.0/assert/mod.ts";
import { handler } from "./index.ts";

// Mock Supabase client
const mockSupabase = {
  rpc: async (fn: string, params: unknown) => ({
    data: [{ inserted_count: 5 }],
    error: null
  })
};

Deno.test("ingest rejects invalid signature", async () => {
  const req = new Request("http://localhost/ingest", {
    method: "POST",
    headers: {
      "X-Source-ID": "test-monitor",
      "X-Signature": "invalid-signature",
      "Content-Type": "application/json"
    },
    body: JSON.stringify([{ id: "evt_123" }])
  });

  const res = await handler(req, mockSupabase);
  assertEquals(res.status, 401);
});

Deno.test("ingest accepts valid batch", async () => {
  // Test with valid signature...
});
```

### Running Tests
```bash
# Local testing
deno test supabase/functions/ingest/

# With Supabase CLI
supabase functions serve --env-file .env.local
```

---

## 5. Client Query Caching Strategy

### Decision
Cache historic data in Zustand store with time-based refresh on component mount.

### Rationale
- Reuses existing Zustand pattern from the codebase
- Historic data is relatively stable (hourly aggregates don't change frequently)
- Refresh on mount ensures data is fresh when user visits dashboard

### Implementation Pattern

```typescript
// In useEventStore.ts - add historic data state
interface EventStore {
  // ... existing state
  historicData: HourlyAggregate[];
  historicDataStatus: 'idle' | 'loading' | 'error' | 'success';
  historicDataFetchedAt: number | null;

  fetchHistoricData: (days: number) => Promise<void>;
}

// In useHistoricData.ts hook
export function useHistoricData(days: 7 | 30) {
  const { historicData, historicDataStatus, fetchHistoricData, historicDataFetchedAt } = useEventStore();

  useEffect(() => {
    // Refresh if data is stale (older than 5 minutes) or not fetched
    const isStale = !historicDataFetchedAt || Date.now() - historicDataFetchedAt > 5 * 60 * 1000;
    if (isStale) {
      fetchHistoricData(days);
    }
  }, [days, fetchHistoricData, historicDataFetchedAt]);

  return { historicData, status: historicDataStatus };
}
```

### Cache Invalidation
- **Time-based**: Refresh if data is older than 5 minutes
- **On mount**: Always check staleness when Heatmap mounts
- **Manual**: Could add refresh button for user-triggered refresh

---

## Summary

| Area | Decision | Key Benefit |
|------|----------|-------------|
| Ed25519 | `@noble/ed25519` | Audited, works in Deno |
| RLS | Enable with no policies | Implicit deny-all, service role bypasses |
| Batch Insert | RPC with `jsonb_populate_recordset` | Single round-trip, atomic |
| Testing | Deno built-in + mocks | No external dependencies |
| Caching | Zustand + time-based refresh | Reuses existing patterns |
