# Quickstart: Supabase Authentication

**Branch**: `002-supabase-auth` | **Date**: 2026-02-04

## Prerequisites

### Required Accounts
- **Supabase Account**: [supabase.com](https://supabase.com) (free tier works)
- **GitHub OAuth App**: For GitHub provider configuration in Supabase

### Local Development Tools
- Rust 1.75+ with cargo
- Node.js 20+ with npm
- Supabase CLI: `npm install -g supabase`
- Docker (for local Supabase)

## Supabase Project Setup

### 1. Create Supabase Project

```bash
# Login to Supabase
supabase login

# Initialize Supabase in project root (if not already done)
supabase init

# Start local Supabase (requires Docker)
supabase start
```

The local Supabase stack provides:
- API URL: `http://localhost:54321`
- Anon Key: Displayed after `supabase start`
- Service Role Key: Displayed after `supabase start`

### 2. Configure GitHub OAuth Provider

1. Go to **GitHub Settings > Developer Settings > OAuth Apps**
2. Create new OAuth App:
   - **Application Name**: VibeTea Local Dev
   - **Homepage URL**: `http://localhost:5173`
   - **Authorization callback URL**: `http://localhost:54321/auth/v1/callback`
3. Copy Client ID and Client Secret
4. In Supabase Dashboard (or local Studio at `http://localhost:54323`):
   - Go to **Authentication > Providers > GitHub**
   - Enable GitHub provider
   - Paste Client ID and Client Secret

### 3. Create Monitor Public Keys Table

Apply the migration:

```bash
supabase db push
```

Or manually create the table:

```sql
CREATE TABLE monitor_public_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id TEXT NOT NULL UNIQUE,
    public_key TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Partial index for active keys lookup
CREATE INDEX idx_monitor_public_keys_active
ON monitor_public_keys(active)
WHERE active = true;
```

### 4. Deploy Public Keys Edge Function

```bash
# Create function directory
mkdir -p supabase/functions/public-keys

# Deploy to local Supabase
supabase functions deploy public-keys --local
```

## Environment Variables

### Server (.env or environment)

```bash
# Supabase Configuration
SUPABASE_URL=http://localhost:54321
SUPABASE_ANON_KEY=<your-anon-key>
SUPABASE_SERVICE_ROLE_KEY=<your-service-role-key>

# Session Configuration (optional, defaults shown)
SESSION_TOKEN_TTL_SECS=300
SESSION_MAX_CAPACITY=10000
PUBLIC_KEY_REFRESH_SECS=30

# Existing Configuration
BEARER_TOKEN=<existing-monitor-token>
RUST_LOG=info,server=debug
```

### Client (.env.local)

```bash
VITE_SUPABASE_URL=http://localhost:54321
VITE_SUPABASE_ANON_KEY=<your-anon-key>
VITE_WS_URL=ws://localhost:8080/ws
```

## Quick Test

### 1. Start Local Supabase

```bash
supabase start
```

### 2. Add Test Monitor Public Key

```bash
# Generate test keypair (if needed)
# The existing monitor uses Ed25519 keys

# Insert a test key into local database
supabase db execute --local "
INSERT INTO monitor_public_keys (source_id, public_key, active)
VALUES ('test-monitor', 'dGVzdHB1YmxpY2tleWJhc2U2NA==', true);
"
```

### 3. Start Server

```bash
cd server
cargo run
```

### 4. Start Client

```bash
cd client
npm install
npm run dev
```

### 5. Test Auth Flow

1. Open `http://localhost:5173`
2. Click "Sign in with GitHub"
3. Complete GitHub OAuth flow
4. Verify WebSocket connection established

## Development Workflow

### Running Tests

```bash
# Server tests (single-threaded for env var safety)
cd server
cargo test --test-threads=1

# Client tests
cd client
npm test
```

### Checking Types

```bash
# Server
cd server
cargo check

# Client
cd client
npm run type-check
```

### Linting

```bash
# Server
cd server
cargo clippy -- -D warnings

# Client
cd client
npm run lint
```

## Common Issues

### "Supabase service unavailable" (503)

- Ensure local Supabase is running: `supabase status`
- Check SUPABASE_URL is correct
- Verify network connectivity to Supabase

### "Invalid JWT" (401)

- JWT may have expired (1 hour default)
- Client should automatically refresh tokens
- Check Supabase anon key matches client and server

### "Unknown source" when posting events

- Verify monitor's public key is in database
- Check `active = true` for the key
- Server refreshes keys every 30 seconds

### Edge function not responding

- Deploy function: `supabase functions deploy public-keys --local`
- Check function logs: `supabase functions logs public-keys`

## Related Documentation

- [spec.md](./spec.md) - Feature specification
- [data-model.md](./data-model.md) - Entity definitions and relationships
- [contracts/auth-api.yaml](./contracts/auth-api.yaml) - API contract
- [contracts/public-keys-function.yaml](./contracts/public-keys-function.yaml) - Edge function contract
