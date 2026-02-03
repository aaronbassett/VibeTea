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

// TODO (T051): Use createClient for database operations
// import { createClient } from "https://esm.sh/@supabase/supabase-js@2";
import { verifyIngestAuth, type AuthResult } from "../_shared/auth.ts";

/** Maximum number of events allowed per batch */
const MAX_BATCH_SIZE = 1000;

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
  // TODO (T044): Auth verification is implemented via verifyIngestAuth
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

  // sourceId will be used in T045 for source validation and T051 for database operations
  const _sourceId = authResult.sourceId;

  // Parse and validate the request body
  // TODO (T045): Implement full event validation with Zod schema
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

  // TODO (T045): Validate each event against the Event schema
  // - Validate event structure (id, source, timestamp, eventType, payload)
  // - Validate eventType enum values
  // - Verify event.source matches authenticated sourceId (422 source_mismatch)
  // - Validate timestamp format

  // TODO (T051): Insert events into database
  // - Initialize Supabase client using environment variables
  // - Call bulk_insert_events database function
  // - Handle ON CONFLICT DO NOTHING for duplicates
  // - Return actual inserted count

  // Placeholder: Return success with event count
  // This will be replaced with actual DB insertion in T051
  const insertedCount = events.length;

  return successResponse(insertedCount, events.length);
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
