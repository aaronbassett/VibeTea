/**
 * VibeTea Query Edge Function
 *
 * GET endpoint for querying hourly event aggregates.
 * Used by clients to fetch historic activity data for heatmap visualization.
 *
 * Authentication: Bearer token (Authorization header)
 * Query Parameters:
 *   - days: 7 | 30 (default: 7) - Number of days of historic data
 *   - source: string (optional) - Filter by monitor source
 *
 * Response: Array of HourlyAggregate records with metadata
 */

import { createClient, SupabaseClient } from "https://esm.sh/@supabase/supabase-js@2";
import { verifyQueryAuth, type AuthResult } from "../_shared/auth.ts";

/**
 * Hourly aggregate record returned by the query
 */
interface HourlyAggregate {
  readonly source: string;
  readonly date: string;
  readonly hour: number;
  readonly eventCount: number;
}

/**
 * Query response metadata
 */
interface QueryMeta {
  readonly totalCount: number;
  readonly daysRequested: number;
  readonly fetchedAt: string;
}

/**
 * Successful query response
 */
interface QueryResponse {
  readonly aggregates: readonly HourlyAggregate[];
  readonly meta: QueryMeta;
}

/**
 * Error response
 */
interface ErrorResponse {
  readonly error: string;
  readonly message: string;
}

/**
 * Valid values for the days query parameter
 */
const VALID_DAYS = [7, 30] as const;
type ValidDays = (typeof VALID_DAYS)[number];

/**
 * Create a JSON response with proper headers
 */
function jsonResponse<T>(data: T, status: number): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: {
      "Content-Type": "application/json",
    },
  });
}

/**
 * Create an error response
 */
function errorResponse(error: string, message: string, status: number): Response {
  const body: ErrorResponse = { error, message };
  return jsonResponse(body, status);
}

/**
 * Initialize Supabase client for database access
 */
function createSupabaseClient(): SupabaseClient {
  const supabaseUrl = Deno.env.get("SUPABASE_URL");
  const supabaseServiceKey = Deno.env.get("SUPABASE_SERVICE_ROLE_KEY");

  if (!supabaseUrl || !supabaseServiceKey) {
    throw new Error("Missing Supabase configuration");
  }

  return createClient(supabaseUrl, supabaseServiceKey);
}

/**
 * Parse and validate query parameters
 *
 * @param url - Request URL to extract query parameters from
 * @returns Parsed parameters or an error object
 */
function parseQueryParams(
  url: URL
): { days: ValidDays; source: string | null } | { error: string; message: string } {
  const daysParam = url.searchParams.get("days");
  const source = url.searchParams.get("source");

  // Default to 7 days if not specified
  let days: ValidDays = 7;

  if (daysParam !== null) {
    const parsedDays = parseInt(daysParam, 10);

    // Validate days is one of the allowed values (7 or 30)
    if (!VALID_DAYS.includes(parsedDays as ValidDays)) {
      return {
        error: "invalid_days",
        message: "days parameter must be 7 or 30",
      };
    }
    days = parsedDays as ValidDays;
  }

  return { days, source };
}

/**
 * Query hourly aggregates from the database
 *
 * TODO (T053): Implement database query
 *   - Query hourly_aggregates table
 *   - Filter by date range (now - days)
 *   - Filter by source if provided
 *   - Order by date/hour descending
 */
async function queryAggregates(
  _client: SupabaseClient,
  _days: ValidDays,
  _source: string | null
): Promise<{ aggregates: HourlyAggregate[] } | { error: string; message: string }> {
  // TODO (T053): Implement actual database query
  // Placeholder return for scaffold
  return {
    aggregates: [],
  };
}

/**
 * Main request handler
 */
async function handleRequest(request: Request): Promise<Response> {
  // Only allow GET requests
  if (request.method !== "GET") {
    return errorResponse(
      "method_not_allowed",
      "Only GET method is allowed",
      405
    );
  }

  // Verify bearer token authentication
  const authResult: AuthResult = verifyQueryAuth(request);
  if (!authResult.isValid) {
    // Determine specific error type for response
    const authHeader = request.headers.get("Authorization");
    if (!authHeader) {
      return errorResponse("missing_auth", "Authorization header is required", 401);
    }
    return errorResponse("invalid_token", "Bearer token is invalid", 401);
  }

  // Parse and validate query parameters
  const url = new URL(request.url);
  const params = parseQueryParams(url);

  if ("error" in params) {
    return errorResponse(params.error, params.message, 400);
  }

  const { days, source } = params;

  // Initialize Supabase client and query database
  try {
    const client = createSupabaseClient();

    // TODO (T053): Implement actual database query
    const result = await queryAggregates(client, days, source);

    if ("error" in result) {
      return errorResponse(result.error, result.message, 500);
    }

    // Build successful response
    const response: QueryResponse = {
      aggregates: result.aggregates,
      meta: {
        totalCount: result.aggregates.length,
        daysRequested: days,
        fetchedAt: new Date().toISOString(),
      },
    };

    return jsonResponse(response, 200);
  } catch (err) {
    console.error("Query error:", err);
    return errorResponse("internal_error", "Database query failed", 500);
  }
}

// Deno Edge Function entry point
Deno.serve(handleRequest);
