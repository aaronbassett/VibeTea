/**
 * VibeTea Ingest Edge Function
 *
 * Handles batch event ingestion from monitors with Ed25519 signature authentication.
 *
 * POST /ingest
 * - Request body: JSON array of events (max 1000)
 * - Headers: X-Source-ID, X-Signature (Ed25519)
 * - Returns: { inserted: number, message: string }
 *
 * @see specs/001-supabase-persistence/contracts/ingest.yaml
 */

import { createClient, SupabaseClient } from "https://esm.sh/@supabase/supabase-js@2";
import { verifyIngestAuth, type AuthResult } from "../_shared/auth.ts";

/** Maximum number of events allowed per batch */
const MAX_BATCH_SIZE = 1000;

/**
 * Valid event types per API contract
 */
const VALID_EVENT_TYPES = [
  "session",
  "activity",
  "tool",
  "agent",
  "summary",
  "error",
] as const;

type EventType = (typeof VALID_EVENT_TYPES)[number];

/**
 * Event schema per API contract
 * @see specs/001-supabase-persistence/contracts/ingest.yaml
 */
interface Event {
  readonly id: string;
  readonly source: string;
  readonly timestamp: string;
  readonly eventType: EventType;
  readonly payload: Record<string, unknown>;
}

/**
 * Regex pattern for event ID: evt_ followed by 20 lowercase alphanumeric characters
 */
const EVENT_ID_PATTERN = /^evt_[a-z0-9]{20}$/;

/**
 * Regex pattern for RFC 3339 timestamp validation
 * Matches: YYYY-MM-DDTHH:mm:ss with optional fractional seconds and timezone
 */
const RFC3339_PATTERN =
  /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$/;

/**
 * Standard error response format per API contract
 */
interface ErrorResponse {
  readonly error: string;
  readonly message: string;
}

/**
 * Success response format per API contract
 */
interface IngestResponse {
  readonly inserted: number;
  readonly message: string;
}

/**
 * Initialize Supabase client for database access
 *
 * @throws Error if SUPABASE_URL or SUPABASE_SERVICE_ROLE_KEY is not configured
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
 * Create a JSON error response with appropriate status code
 */
function errorResponse(
  status: number,
  error: string,
  message: string
): Response {
  const body: ErrorResponse = { error, message };
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

/**
 * Result of validating an event
 */
type EventValidationResult =
  | { readonly isValid: true; readonly event: Event }
  | { readonly isValid: false; readonly error: string; readonly errorCode: string };

/**
 * Validate a single event against the Event schema
 */
function validateEvent(value: unknown, index: number): EventValidationResult {
  if (typeof value !== "object" || value === null) {
    return {
      isValid: false,
      error: `Event at index ${index} must be an object`,
      errorCode: "invalid_event",
    };
  }

  const obj = value as Record<string, unknown>;

  // Validate id: string, pattern ^evt_[a-z0-9]{20}$
  if (typeof obj.id !== "string") {
    return {
      isValid: false,
      error: `Event at index ${index} missing required field 'id'`,
      errorCode: "invalid_event",
    };
  }
  if (!EVENT_ID_PATTERN.test(obj.id)) {
    return {
      isValid: false,
      error: `Event at index ${index} has invalid id format '${obj.id}'`,
      errorCode: "invalid_event",
    };
  }

  // Validate source: string, non-empty
  if (typeof obj.source !== "string") {
    return {
      isValid: false,
      error: `Event at index ${index} missing required field 'source'`,
      errorCode: "invalid_event",
    };
  }
  if (obj.source.length === 0) {
    return {
      isValid: false,
      error: `Event at index ${index} has empty 'source' field`,
      errorCode: "invalid_event",
    };
  }

  // Validate timestamp: string, RFC 3339 format
  if (typeof obj.timestamp !== "string") {
    return {
      isValid: false,
      error: `Event at index ${index} missing required field 'timestamp'`,
      errorCode: "invalid_event",
    };
  }
  if (!RFC3339_PATTERN.test(obj.timestamp)) {
    return {
      isValid: false,
      error: `Event at index ${index} has invalid timestamp format '${obj.timestamp}'`,
      errorCode: "invalid_event",
    };
  }

  // Validate eventType: string, one of valid types
  if (typeof obj.eventType !== "string") {
    return {
      isValid: false,
      error: `Event at index ${index} missing required field 'eventType'`,
      errorCode: "invalid_event",
    };
  }
  if (!VALID_EVENT_TYPES.includes(obj.eventType as EventType)) {
    return {
      isValid: false,
      error: `Invalid event type '${obj.eventType}' at index ${index}`,
      errorCode: "invalid_event_type",
    };
  }

  // Validate payload: object
  if (typeof obj.payload !== "object" || obj.payload === null) {
    return {
      isValid: false,
      error: `Event at index ${index} missing required field 'payload' or payload is not an object`,
      errorCode: "invalid_event",
    };
  }

  return {
    isValid: true,
    event: {
      id: obj.id,
      source: obj.source,
      timestamp: obj.timestamp,
      eventType: obj.eventType as EventType,
      payload: obj.payload as Record<string, unknown>,
    },
  };
}

/**
 * Validate that event source matches authenticated source
 */
function validateSourceMatch(
  event: Event,
  authenticatedSourceId: string
): { readonly isValid: true } | { readonly isValid: false; readonly error: string } {
  if (event.source !== authenticatedSourceId) {
    return {
      isValid: false,
      error: `Event source '${event.source}' does not match authenticated source '${authenticatedSourceId}'`,
    };
  }
  return { isValid: true };
}

/**
 * Create a JSON success response
 */
function successResponse(inserted: number, total: number): Response {
  const duplicates = total - inserted;
  const message =
    duplicates > 0
      ? `Successfully processed ${total} events (${duplicates} duplicates skipped)`
      : `Successfully processed ${total} events`;

  const body: IngestResponse = { inserted, message };
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

/**
 * Main request handler for the ingest endpoint
 */
async function handleRequest(request: Request): Promise<Response> {
  // Only accept POST requests
  if (request.method !== "POST") {
    return errorResponse(405, "method_not_allowed", "Only POST is supported");
  }

  // Read the request body as text (needed for signature verification)
  let bodyText: string;
  try {
    bodyText = await request.text();
  } catch {
    return errorResponse(400, "invalid_request", "Failed to read request body");
  }

  // Verify Ed25519 signature authentication
  const authResult: AuthResult = await verifyIngestAuth(request, bodyText);
  if (!authResult.isValid) {
    // Map auth errors to appropriate HTTP status codes
    if (
      authResult.error?.includes("Missing") ||
      authResult.error?.includes("Unknown source")
    ) {
      return errorResponse(
        401,
        authResult.error.includes("Missing") ? "missing_auth" : "unknown_source",
        authResult.error
      );
    }
    return errorResponse(401, "invalid_signature", authResult.error ?? "Authentication failed");
  }

  const sourceId = authResult.sourceId;

  // Parse and validate the request body
  let events: unknown[];
  try {
    const parsed: unknown = JSON.parse(bodyText);
    if (!Array.isArray(parsed)) {
      return errorResponse(
        400,
        "invalid_request",
        "Request body must be a JSON array"
      );
    }
    events = parsed;
  } catch {
    return errorResponse(400, "invalid_request", "Request body is not valid JSON");
  }

  // Validate batch constraints
  if (events.length === 0) {
    return errorResponse(400, "empty_batch", "Request body must be a non-empty array");
  }

  if (events.length > MAX_BATCH_SIZE) {
    return errorResponse(
      400,
      "batch_too_large",
      `Maximum batch size is ${MAX_BATCH_SIZE} events`
    );
  }

  // Validate each event against the Event schema
  const validatedEvents: Event[] = [];
  for (let i = 0; i < events.length; i++) {
    const validationResult = validateEvent(events[i], i);
    if (!validationResult.isValid) {
      // Use 422 for invalid_event_type per contract, 400 for other validation errors
      const status = validationResult.errorCode === "invalid_event_type" ? 422 : 400;
      return errorResponse(status, validationResult.errorCode, validationResult.error);
    }

    // Verify event source matches authenticated sourceId
    const sourceCheck = validateSourceMatch(validationResult.event, sourceId!);
    if (!sourceCheck.isValid) {
      return errorResponse(422, "source_mismatch", sourceCheck.error);
    }

    validatedEvents.push(validationResult.event);
  }

  // Initialize Supabase client and insert events into database
  const client = createSupabaseClient();

  // Call bulk_insert_events RPC function
  // The function handles ON CONFLICT DO NOTHING for duplicates
  const { data, error: rpcError } = await client.rpc("bulk_insert_events", {
    events_json: validatedEvents,
  });

  if (rpcError) {
    console.error("Database insert error:", rpcError);
    return errorResponse(500, "internal_error", "Failed to insert events into database");
  }

  // The RPC function returns a table with one row containing inserted_count
  const insertedCount = Array.isArray(data) && data.length > 0
    ? (data[0] as { inserted_count: number }).inserted_count
    : 0;

  return successResponse(insertedCount, validatedEvents.length);
}

/**
 * Deno serve handler with CORS support
 */
Deno.serve(async (request: Request): Promise<Response> => {
  // Handle CORS preflight requests
  if (request.method === "OPTIONS") {
    return new Response(null, {
      status: 204,
      headers: {
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "POST, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type, X-Source-ID, X-Signature",
        "Access-Control-Max-Age": "86400",
      },
    });
  }

  try {
    const response = await handleRequest(request);

    // Add CORS headers to all responses
    response.headers.set("Access-Control-Allow-Origin", "*");

    return response;
  } catch (error) {
    // Log unexpected errors for debugging
    console.error("Unexpected error in ingest handler:", error);

    return errorResponse(
      500,
      "internal_error",
      "An unexpected error occurred"
    );
  }
});
