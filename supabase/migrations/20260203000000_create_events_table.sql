-- Migration: Create events table
-- Version: 001
-- Date: 2026-02-03
-- Description: Main events table for storing privacy-filtered events from monitors

-- Create events table
CREATE TABLE IF NOT EXISTS public.events (
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
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON public.events (timestamp DESC);

-- Index for source filtering (optional, for future multi-tenant queries)
CREATE INDEX IF NOT EXISTS idx_events_source ON public.events (source);

-- Composite index for common query pattern (source + time range)
CREATE INDEX IF NOT EXISTS idx_events_source_timestamp ON public.events (source, timestamp DESC);

-- Enable Row Level Security (implicit deny-all without policies)
ALTER TABLE public.events ENABLE ROW LEVEL SECURITY;

-- Force RLS for table owners (defense in depth)
ALTER TABLE public.events FORCE ROW LEVEL SECURITY;

-- No policies = service_role only access (SC-003)
