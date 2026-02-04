-- Migration: Create monitor_public_keys table
-- This table stores Ed25519 public keys for VibeTea monitors
-- The server periodically fetches these keys via an edge function

CREATE TABLE IF NOT EXISTS monitor_public_keys (
    -- Unique identifier for the monitor source
    source_id TEXT PRIMARY KEY,

    -- Base64-encoded Ed25519 public key (44 characters for 32 bytes)
    public_key TEXT NOT NULL,

    -- Optional description of this monitor
    description TEXT,

    -- Timestamps for auditing
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add constraint to ensure public_key is not empty
ALTER TABLE monitor_public_keys
    ADD CONSTRAINT public_key_not_empty CHECK (LENGTH(public_key) > 0);

-- Add constraint to ensure source_id is not empty
ALTER TABLE monitor_public_keys
    ADD CONSTRAINT source_id_not_empty CHECK (LENGTH(source_id) > 0);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_monitor_public_keys_source_id
    ON monitor_public_keys(source_id);

-- Trigger to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_monitor_public_keys_updated_at
    BEFORE UPDATE ON monitor_public_keys
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Grant read access to the anon role for the edge function
-- The edge function will read public keys without authentication
GRANT SELECT ON monitor_public_keys TO anon;

-- Add comment explaining the table purpose
COMMENT ON TABLE monitor_public_keys IS
    'Stores Ed25519 public keys for VibeTea monitors. Keys are fetched by the server every 30 seconds.';

COMMENT ON COLUMN monitor_public_keys.source_id IS
    'Unique identifier for the monitor (e.g., "desktop-monitor-1")';

COMMENT ON COLUMN monitor_public_keys.public_key IS
    'Base64-encoded Ed25519 public key (32 bytes encoded as 44 characters)';
