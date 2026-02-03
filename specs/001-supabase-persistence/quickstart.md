# Quickstart: Supabase Persistence Development

**Feature**: 001-supabase-persistence
**Date**: 2026-02-03

This guide covers local development setup for the Supabase persistence layer.

## Prerequisites

- [Supabase CLI](https://supabase.com/docs/guides/cli) (v1.150+)
- Docker (required for local Supabase)
- Node.js 18+ (for client development)
- Rust toolchain (for monitor development)

## Initial Setup

### 1. Install Supabase CLI

```bash
# macOS
brew install supabase/tap/supabase

# Linux/Windows (via npm)
npm install -g supabase

# Verify installation
supabase --version
```

### 2. Initialize Supabase in VibeTea

```bash
cd /home/ubuntu/Projects/VibeTea

# Initialize Supabase (creates supabase/ directory)
supabase init

# Start local Supabase (PostgreSQL, Auth, Edge Functions)
supabase start
```

After `supabase start`, you'll see output like:
```
Started supabase local development setup.

         API URL: http://127.0.0.1:54321
     GraphQL URL: http://127.0.0.1:54321/graphql/v1
          DB URL: postgresql://postgres:postgres@127.0.0.1:54322/postgres
      Studio URL: http://127.0.0.1:54323
    Inbucket URL: http://127.0.0.1:54324
      JWT secret: super-secret-jwt-token-...
        anon key: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
service_role key: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Save these values!** You'll need them for configuration.

### 3. Apply Database Migrations

```bash
# Create migration file
supabase migration new create_events_table

# Edit the migration file (copy schema from data-model.md)
# File: supabase/migrations/20260203000000_create_events_table.sql

# Apply migrations to local database
supabase db push
```

### 4. Create Edge Functions

```bash
# Create ingest function
supabase functions new ingest

# Create query function
supabase functions new query

# This creates:
# supabase/functions/ingest/index.ts
# supabase/functions/query/index.ts
```

## Environment Configuration

### Local Development (.env.local)

Create `supabase/.env.local`:

```bash
# Supabase URLs (from supabase start output)
SUPABASE_URL=http://127.0.0.1:54321
SUPABASE_SERVICE_ROLE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

# VibeTea-specific configuration
VIBETEA_SUBSCRIBER_TOKEN=dev-token-for-testing

# Public keys for signature verification (base64-encoded)
# Format: source_id:public_key (comma-separated for multiple)
VIBETEA_PUBLIC_KEYS=dev-monitor:MCowBQYDK2VwAyEA...
```

### Monitor Configuration

Add to your shell environment or `.env`:

```bash
# Enable persistence
export VIBETEA_SUPABASE_URL=http://127.0.0.1:54321/functions/v1

# Optional: Configure batch interval (default: 60 seconds)
export VIBETEA_SUPABASE_BATCH_INTERVAL_SECS=10
```

### Client Configuration

Create `client/.env.local`:

```bash
# Enable persistence features
VITE_SUPABASE_URL=http://127.0.0.1:54321/functions/v1

# WebSocket server (existing)
VITE_WS_URL=ws://localhost:8080/ws
```

## Running Edge Functions Locally

### Start Function Server

```bash
# Serve all functions with hot reload
supabase functions serve --env-file supabase/.env.local

# Output:
# Serving functions on http://127.0.0.1:54321/functions/v1/<function-name>
```

### Test Ingest Endpoint

```bash
# Generate a test signature (you'll need your private key)
# For development, you can temporarily disable signature verification

curl -X POST http://127.0.0.1:54321/functions/v1/ingest \
  -H "Content-Type: application/json" \
  -H "X-Source-ID: dev-monitor" \
  -H "X-Signature: <base64-signature>" \
  -d '[
    {
      "id": "evt_test123456789012345",
      "source": "dev-monitor",
      "timestamp": "2026-02-03T14:30:00Z",
      "eventType": "tool",
      "payload": {
        "sessionId": "550e8400-e29b-41d4-a716-446655440000",
        "tool": "Read",
        "status": "completed",
        "context": "main.rs"
      }
    }
  ]'

# Expected response:
# {"inserted": 1, "message": "Successfully processed 1 events"}
```

### Test Query Endpoint

```bash
curl http://127.0.0.1:54321/functions/v1/query?days=7 \
  -H "Authorization: Bearer dev-token-for-testing"

# Expected response:
# {"aggregates": [...], "meta": {"totalCount": 24, "daysRequested": 7, "fetchedAt": "..."}}
```

## Development Workflow

### 1. Edge Function Development

```bash
# Watch for changes and auto-reload
supabase functions serve --env-file supabase/.env.local

# In another terminal, test your changes
curl http://127.0.0.1:54321/functions/v1/ingest ...
```

### 2. Database Schema Changes

```bash
# Create a new migration
supabase migration new add_index_to_events

# Edit the migration file
# supabase/migrations/20260203000001_add_index_to_events.sql

# Apply to local database
supabase db push

# View current schema
supabase db dump --schema public
```

### 3. Monitor Development

```bash
# Run monitor with persistence enabled
VIBETEA_SUPABASE_URL=http://127.0.0.1:54321/functions/v1 \
  cargo run -p vibetea-monitor -- run
```

### 4. Client Development

```bash
cd client

# Start with persistence enabled
npm run dev
# Heatmap will now fetch historic data from edge function
```

## Testing

### Edge Function Tests

```bash
# Run Deno tests for edge functions
deno test supabase/functions/

# Run specific function tests
deno test supabase/functions/ingest/
deno test supabase/functions/query/
```

### Monitor Persistence Tests

```bash
# Run monitor tests (with mocked HTTP)
cargo test -p vibetea-monitor persistence
```

### Client Tests

```bash
cd client

# Run tests (MSW mocks edge function responses)
npm test
```

## Debugging

### View Function Logs

```bash
# Stream logs from edge functions
supabase functions logs ingest --follow
supabase functions logs query --follow
```

### Access Local Database

```bash
# Connect via psql
psql postgresql://postgres:postgres@127.0.0.1:54322/postgres

# Or use Supabase Studio
open http://127.0.0.1:54323
```

### Inspect Events Table

```sql
-- Count events
SELECT COUNT(*) FROM public.events;

-- View recent events
SELECT id, source, timestamp, event_type
FROM public.events
ORDER BY timestamp DESC
LIMIT 10;

-- Check hourly aggregates
SELECT * FROM public.get_hourly_aggregates(7, NULL);
```

## Common Issues

### "Connection refused" from edge functions

Ensure Docker is running and Supabase is started:
```bash
docker ps  # Should show supabase containers
supabase status  # Shows service URLs
```

### Signature verification failing

1. Ensure public key in `VIBETEA_PUBLIC_KEYS` matches the monitor's private key
2. Verify base64 encoding is correct
3. Check that the request body matches exactly what was signed

### RLS blocking access

Edge functions must use the service role key:
```typescript
const supabase = createClient(
  Deno.env.get("SUPABASE_URL")!,
  Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!  // NOT anon key
);
```

### Heatmap not showing data

1. Check `VITE_SUPABASE_URL` is set in client `.env.local`
2. Verify edge function is running (`supabase functions serve`)
3. Check browser console for fetch errors
4. Ensure bearer token matches `VIBETEA_SUBSCRIBER_TOKEN`

## Stopping Local Supabase

```bash
# Stop all services (preserves data)
supabase stop

# Stop and delete all data
supabase stop --no-backup
```

## Next Steps

1. Implement edge functions following contracts in `contracts/`
2. Add persistence module to monitor
3. Integrate historic data hook in client
4. Deploy to Supabase hosted project
