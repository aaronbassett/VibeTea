-- Migration: Create database functions
-- Version: 001
-- Date: 2026-02-03
-- Description: PostgreSQL functions for bulk event insertion and hourly aggregation

-- ============================================================================
-- Function: bulk_insert_events
-- Description: Atomic batch insertion of events from the ingest edge function
-- Parameters: events_json - JSONB array of event objects
-- Returns: Number of successfully inserted events
-- Note: Uses ON CONFLICT DO NOTHING for idempotency
-- ============================================================================
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

-- Grant execute permission to service role only
GRANT EXECUTE ON FUNCTION public.bulk_insert_events(JSONB) TO service_role;

-- ============================================================================
-- Function: get_hourly_aggregates
-- Description: Retrieve hourly event counts for heatmap visualization
-- Parameters:
--   days_back - Number of days to look back (default: 7)
--   source_filter - Optional source filter (default: NULL for all sources)
-- Returns: Table of (source, date, hour, event_count) sorted by date/hour DESC
-- ============================================================================
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

-- Grant execute permission to service role only
GRANT EXECUTE ON FUNCTION public.get_hourly_aggregates(INTEGER, TEXT) TO service_role;
