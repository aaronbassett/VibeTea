/**
 * Integration tests for Row Level Security (RLS) on the events table (SC-003)
 *
 * These tests verify that RLS properly denies direct database access for
 * unauthenticated users and users with the anon key, while allowing
 * service_role key to bypass RLS.
 *
 * The events table has:
 * - ENABLE ROW LEVEL SECURITY
 * - FORCE ROW LEVEL SECURITY
 * - No policies (implicit deny-all for non-service roles)
 *
 * @see supabase/migrations/20260203000000_create_events_table.sql
 * @see specs/001-supabase-persistence/spec.md (SC-003)
 *
 * Run with: deno test --allow-env --allow-net supabase/functions/_tests/rls.test.ts
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.224.0/assert/mod.ts";
import { createClient, SupabaseClient } from "https://esm.sh/@supabase/supabase-js@2";

/**
 * Environment variables for Supabase connection
 */
const SUPABASE_URL = Deno.env.get("SUPABASE_URL");
const SUPABASE_ANON_KEY = Deno.env.get("SUPABASE_ANON_KEY");
const SUPABASE_SERVICE_ROLE_KEY = Deno.env.get("SUPABASE_SERVICE_ROLE_KEY");

/**
 * Test event data that matches the events table schema
 */
const TEST_EVENT = {
  id: "evt_rls_test_00000000001",
  source: "rls-test-source",
  timestamp: new Date().toISOString(),
  event_type: "tool",
  payload: {
    sessionId: "550e8400-e29b-41d4-a716-446655440000",
    tool: "Read",
    status: "completed",
  },
} as const;

/**
 * Check if the required environment variables are set
 */
function hasRequiredEnvVars(): boolean {
  return Boolean(SUPABASE_URL) && Boolean(SUPABASE_ANON_KEY);
}

/**
 * Check if the service role key is available for positive case testing
 */
function hasServiceRoleKey(): boolean {
  return Boolean(SUPABASE_SERVICE_ROLE_KEY);
}

/**
 * Create a Supabase client with the anon key
 */
function createAnonClient(): SupabaseClient {
  if (!SUPABASE_URL || !SUPABASE_ANON_KEY) {
    throw new Error("Missing required environment variables");
  }
  return createClient(SUPABASE_URL, SUPABASE_ANON_KEY);
}

/**
 * Create a Supabase client with the service role key
 */
function createServiceRoleClient(): SupabaseClient {
  if (!SUPABASE_URL || !SUPABASE_SERVICE_ROLE_KEY) {
    throw new Error("Missing SUPABASE_SERVICE_ROLE_KEY environment variable");
  }
  return createClient(SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY);
}

// =============================================================================
// Test Suite: RLS denies SELECT for anon key
// =============================================================================

Deno.test({
  name: "RLS: anon key cannot SELECT from events table",
  ignore: !hasRequiredEnvVars(),
  async fn() {
    const client = createAnonClient();

    // Attempt to SELECT from events table
    const { data, error } = await client.from("events").select("*").limit(1);

    // RLS should return empty result (not an error) when no policies allow access
    // This is the default Supabase behavior - empty result set, not an error
    assertEquals(error, null, "Expected no error, RLS returns empty result");
    assertExists(data, "Expected data to be defined");
    assertEquals(data.length, 0, "Expected empty result due to RLS deny");
  },
});

Deno.test({
  name: "RLS: anon key cannot SELECT with specific filters",
  ignore: !hasRequiredEnvVars(),
  async fn() {
    const client = createAnonClient();

    // Attempt to SELECT with a filter - should still be denied
    const { data, error } = await client
      .from("events")
      .select("id, source, timestamp")
      .eq("source", "any-source")
      .limit(10);

    assertEquals(error, null, "Expected no error, RLS returns empty result");
    assertExists(data, "Expected data to be defined");
    assertEquals(data.length, 0, "Expected empty result due to RLS deny");
  },
});

// =============================================================================
// Test Suite: RLS denies INSERT for anon key
// =============================================================================

Deno.test({
  name: "RLS: anon key cannot INSERT into events table",
  ignore: !hasRequiredEnvVars(),
  async fn() {
    const client = createAnonClient();

    // Attempt to INSERT into events table
    const { data, error } = await client.from("events").insert(TEST_EVENT).select();

    // RLS should return an error for INSERT when no policies allow access
    assertExists(error, "Expected error due to RLS deny on INSERT");
    assertEquals(data, null, "Expected null data on INSERT failure");

    // The error message should indicate a policy violation
    // Supabase returns a 42501 (insufficient_privilege) or similar RLS error
    assertExists(error.code, "Expected error code to be present");
  },
});

Deno.test({
  name: "RLS: anon key cannot INSERT multiple events",
  ignore: !hasRequiredEnvVars(),
  async fn() {
    const client = createAnonClient();

    // Attempt to INSERT multiple events
    const events = [
      { ...TEST_EVENT, id: "evt_rls_test_00000000002" },
      { ...TEST_EVENT, id: "evt_rls_test_00000000003" },
    ];

    const { data, error } = await client.from("events").insert(events).select();

    assertExists(error, "Expected error due to RLS deny on bulk INSERT");
    assertEquals(data, null, "Expected null data on INSERT failure");
  },
});

// =============================================================================
// Test Suite: RLS denies UPDATE for anon key
// =============================================================================

Deno.test({
  name: "RLS: anon key cannot UPDATE events table",
  ignore: !hasRequiredEnvVars(),
  async fn() {
    const client = createAnonClient();

    // Attempt to UPDATE events table
    const { data, error } = await client
      .from("events")
      .update({ source: "hacked-source" })
      .eq("id", TEST_EVENT.id)
      .select();

    // RLS should return empty result for UPDATE when no rows match policies
    assertEquals(error, null, "Expected no error, RLS returns empty result for UPDATE");
    assertExists(data, "Expected data to be defined");
    assertEquals(data.length, 0, "Expected no rows updated due to RLS deny");
  },
});

// =============================================================================
// Test Suite: RLS denies DELETE for anon key
// =============================================================================

Deno.test({
  name: "RLS: anon key cannot DELETE from events table",
  ignore: !hasRequiredEnvVars(),
  async fn() {
    const client = createAnonClient();

    // Attempt to DELETE from events table
    const { data, error } = await client
      .from("events")
      .delete()
      .eq("id", TEST_EVENT.id)
      .select();

    // RLS should return empty result for DELETE when no rows match policies
    assertEquals(error, null, "Expected no error, RLS returns empty result for DELETE");
    assertExists(data, "Expected data to be defined");
    assertEquals(data.length, 0, "Expected no rows deleted due to RLS deny");
  },
});

// =============================================================================
// Test Suite: Service role key bypasses RLS (positive case)
// =============================================================================

Deno.test({
  name: "RLS: service role key can INSERT and SELECT (bypass RLS)",
  ignore: !hasRequiredEnvVars() || !hasServiceRoleKey(),
  async fn() {
    const client = createServiceRoleClient();
    const testId = `evt_rls_svc_${Date.now()}`;
    const testEvent = { ...TEST_EVENT, id: testId };

    try {
      // INSERT should succeed with service role
      const { data: insertData, error: insertError } = await client
        .from("events")
        .insert(testEvent)
        .select();

      assertEquals(insertError, null, "Service role INSERT should succeed");
      assertExists(insertData, "Expected insert data to be returned");
      assertEquals(insertData.length, 1, "Expected one row inserted");
      assertEquals(insertData[0].id, testId, "Expected correct event ID");

      // SELECT should succeed with service role
      const { data: selectData, error: selectError } = await client
        .from("events")
        .select("*")
        .eq("id", testId);

      assertEquals(selectError, null, "Service role SELECT should succeed");
      assertExists(selectData, "Expected select data to be returned");
      assertEquals(selectData.length, 1, "Expected one row selected");
      assertEquals(selectData[0].source, TEST_EVENT.source, "Expected correct source");
    } finally {
      // Clean up: delete the test event
      await client.from("events").delete().eq("id", testId);
    }
  },
});

Deno.test({
  name: "RLS: service role key can UPDATE events (bypass RLS)",
  ignore: !hasRequiredEnvVars() || !hasServiceRoleKey(),
  async fn() {
    const client = createServiceRoleClient();
    const testId = `evt_rls_upd_${Date.now()}`;
    const testEvent = { ...TEST_EVENT, id: testId };

    try {
      // First insert an event
      await client.from("events").insert(testEvent);

      // UPDATE should succeed with service role
      const updatedSource = "updated-source";
      const { data: updateData, error: updateError } = await client
        .from("events")
        .update({ source: updatedSource })
        .eq("id", testId)
        .select();

      assertEquals(updateError, null, "Service role UPDATE should succeed");
      assertExists(updateData, "Expected update data to be returned");
      assertEquals(updateData.length, 1, "Expected one row updated");
      assertEquals(updateData[0].source, updatedSource, "Expected updated source");
    } finally {
      // Clean up
      await client.from("events").delete().eq("id", testId);
    }
  },
});

Deno.test({
  name: "RLS: service role key can DELETE events (bypass RLS)",
  ignore: !hasRequiredEnvVars() || !hasServiceRoleKey(),
  async fn() {
    const client = createServiceRoleClient();
    const testId = `evt_rls_del_${Date.now()}`;
    const testEvent = { ...TEST_EVENT, id: testId };

    // First insert an event
    await client.from("events").insert(testEvent);

    // DELETE should succeed with service role
    const { data: deleteData, error: deleteError } = await client
      .from("events")
      .delete()
      .eq("id", testId)
      .select();

    assertEquals(deleteError, null, "Service role DELETE should succeed");
    assertExists(deleteData, "Expected delete data to be returned");
    assertEquals(deleteData.length, 1, "Expected one row deleted");

    // Verify the event is actually deleted
    const { data: verifyData } = await client
      .from("events")
      .select("*")
      .eq("id", testId);

    assertExists(verifyData, "Expected verify data to be defined");
    assertEquals(verifyData.length, 0, "Expected event to be deleted");
  },
});

// =============================================================================
// Test Suite: Verify RLS behavior with edge cases
// =============================================================================

Deno.test({
  name: "RLS: anon key upsert is denied",
  ignore: !hasRequiredEnvVars(),
  async fn() {
    const client = createAnonClient();

    // Attempt to UPSERT (insert or update)
    const { data, error } = await client
      .from("events")
      .upsert(TEST_EVENT)
      .select();

    // UPSERT should fail due to RLS
    assertExists(error, "Expected error due to RLS deny on UPSERT");
    assertEquals(data, null, "Expected null data on UPSERT failure");
  },
});

Deno.test({
  name: "RLS: anon key count query returns zero",
  ignore: !hasRequiredEnvVars(),
  async fn() {
    const client = createAnonClient();

    // Attempt to count events
    const { count, error } = await client
      .from("events")
      .select("*", { count: "exact", head: true });

    // Count should succeed but return 0 due to RLS
    assertEquals(error, null, "Expected no error on count query");
    assertEquals(count, 0, "Expected count of 0 due to RLS deny");
  },
});

Deno.test({
  name: "RLS: anon key aggregation query returns empty",
  ignore: !hasRequiredEnvVars(),
  async fn() {
    const client = createAnonClient();

    // Attempt to query with aggregation-like patterns
    const { data, error } = await client
      .from("events")
      .select("source, event_type")
      .limit(100);

    assertEquals(error, null, "Expected no error, RLS returns empty result");
    assertExists(data, "Expected data to be defined");
    assertEquals(data.length, 0, "Expected empty result due to RLS deny");
  },
});
