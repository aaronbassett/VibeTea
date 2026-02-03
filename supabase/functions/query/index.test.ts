/**
 * Unit tests for VibeTea Query Edge Function
 *
 * Tests authentication and query parameter validation without requiring
 * a real Supabase instance. Database operations are stubbed.
 *
 * Run with: deno test --allow-env --allow-net supabase/functions/query/index.test.ts
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.224.0/assert/mod.ts";
import {
  afterEach,
  beforeEach,
  describe,
  it,
} from "https://deno.land/std@0.224.0/testing/bdd.ts";

// Import the shared auth module for isolated testing
import { validateBearerToken, verifyQueryAuth } from "../_shared/auth.ts";

/**
 * Test configuration constants
 */
const TEST_SUBSCRIBER_TOKEN = "test-subscriber-token-12345";
const BASE_URL = "http://localhost:54321/functions/v1/query";

/**
 * Environment variable management for tests
 */
class EnvGuard {
  private readonly savedValues: Map<string, string | undefined> = new Map();

  save(key: string): void {
    this.savedValues.set(key, Deno.env.get(key));
  }

  restore(): void {
    for (const [key, value] of this.savedValues) {
      if (value === undefined) {
        Deno.env.delete(key);
      } else {
        Deno.env.set(key, value);
      }
    }
    this.savedValues.clear();
  }
}

/**
 * Create a mock Request object with optional headers and URL
 */
function createMockRequest(options: {
  url?: string;
  method?: string;
  authHeader?: string | null;
}): Request {
  const url = options.url ?? BASE_URL;
  const method = options.method ?? "GET";
  const headers = new Headers();

  if (options.authHeader !== null && options.authHeader !== undefined) {
    headers.set("Authorization", options.authHeader);
  }

  return new Request(url, {
    method,
    headers,
  });
}

/**
 * Parse JSON response body
 */
async function parseJsonResponse<T>(response: Response): Promise<T> {
  return (await response.json()) as T;
}

/**
 * Error response shape
 */
interface ErrorResponse {
  readonly error: string;
  readonly message: string;
}

// ============================================================================
// Authentication Tests - Using shared auth module directly
// ============================================================================

describe("Query Authentication", () => {
  const envGuard = new EnvGuard();

  beforeEach(() => {
    envGuard.save("VIBETEA_SUBSCRIBER_TOKEN");
    Deno.env.set("VIBETEA_SUBSCRIBER_TOKEN", TEST_SUBSCRIBER_TOKEN);
  });

  afterEach(() => {
    envGuard.restore();
  });

  describe("validateBearerToken", () => {
    it("returns false when Authorization header is null", () => {
      const result = validateBearerToken(null);
      assertEquals(result, false);
    });

    it("returns false when Authorization header is empty", () => {
      const result = validateBearerToken("");
      assertEquals(result, false);
    });

    it("returns false when Authorization header lacks Bearer prefix", () => {
      const result = validateBearerToken(TEST_SUBSCRIBER_TOKEN);
      assertEquals(result, false);
    });

    it("returns false when token uses lowercase bearer prefix", () => {
      const result = validateBearerToken(`bearer ${TEST_SUBSCRIBER_TOKEN}`);
      assertEquals(result, false);
    });

    it("returns false when bearer token is wrong", () => {
      const result = validateBearerToken("Bearer wrong-token");
      assertEquals(result, false);
    });

    it("returns false when Bearer prefix has no token", () => {
      const result = validateBearerToken("Bearer ");
      assertEquals(result, false);
    });

    it("returns true when bearer token is correct", () => {
      const result = validateBearerToken(`Bearer ${TEST_SUBSCRIBER_TOKEN}`);
      assertEquals(result, true);
    });

    it("returns false when VIBETEA_SUBSCRIBER_TOKEN is not set", () => {
      Deno.env.delete("VIBETEA_SUBSCRIBER_TOKEN");
      const result = validateBearerToken(`Bearer ${TEST_SUBSCRIBER_TOKEN}`);
      assertEquals(result, false);
    });
  });

  describe("verifyQueryAuth", () => {
    it("returns isValid: false when Authorization header is missing", () => {
      const request = createMockRequest({ authHeader: null });
      const result = verifyQueryAuth(request);

      assertEquals(result.isValid, false);
      assertExists(result.error);
    });

    it("returns isValid: false when token format is invalid", () => {
      const request = createMockRequest({
        authHeader: "InvalidFormat token123",
      });
      const result = verifyQueryAuth(request);

      assertEquals(result.isValid, false);
      assertExists(result.error);
    });

    it("returns isValid: false when token is wrong", () => {
      const request = createMockRequest({
        authHeader: "Bearer incorrect-token",
      });
      const result = verifyQueryAuth(request);

      assertEquals(result.isValid, false);
      assertExists(result.error);
    });

    it("returns isValid: true when token is correct", () => {
      const request = createMockRequest({
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const result = verifyQueryAuth(request);

      assertEquals(result.isValid, true);
      assertEquals(result.error, undefined);
    });
  });
});

// ============================================================================
// Query Parameter Validation Tests
// ============================================================================

/**
 * Standalone parseQueryParams function for testing
 * (Mirrors the implementation in index.ts)
 */
const VALID_DAYS = [7, 30] as const;
type ValidDays = (typeof VALID_DAYS)[number];

function parseQueryParams(
  url: URL,
): { days: ValidDays; source: string | null } | {
  error: string;
  message: string;
} {
  const daysParam = url.searchParams.get("days");
  const source = url.searchParams.get("source");

  let days: ValidDays = 7;

  if (daysParam !== null) {
    const parsedDays = parseInt(daysParam, 10);

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

describe("Query Parameter Validation", () => {
  describe("days parameter", () => {
    it("defaults to 7 when days parameter is not provided", () => {
      const url = new URL(BASE_URL);
      const result = parseQueryParams(url);

      assertEquals("error" in result, false);
      if (!("error" in result)) {
        assertEquals(result.days, 7);
      }
    });

    it("returns invalid_days error when days=5", () => {
      const url = new URL(`${BASE_URL}?days=5`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, true);
      if ("error" in result) {
        assertEquals(result.error, "invalid_days");
        assertEquals(result.message, "days parameter must be 7 or 30");
      }
    });

    it("returns invalid_days error when days=1", () => {
      const url = new URL(`${BASE_URL}?days=1`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, true);
      if ("error" in result) {
        assertEquals(result.error, "invalid_days");
      }
    });

    it("returns invalid_days error when days=14", () => {
      const url = new URL(`${BASE_URL}?days=14`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, true);
      if ("error" in result) {
        assertEquals(result.error, "invalid_days");
      }
    });

    it("returns invalid_days error when days=60", () => {
      const url = new URL(`${BASE_URL}?days=60`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, true);
      if ("error" in result) {
        assertEquals(result.error, "invalid_days");
      }
    });

    it("returns invalid_days error when days is negative", () => {
      const url = new URL(`${BASE_URL}?days=-7`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, true);
      if ("error" in result) {
        assertEquals(result.error, "invalid_days");
      }
    });

    it("returns invalid_days error when days is not a number", () => {
      const url = new URL(`${BASE_URL}?days=abc`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, true);
      if ("error" in result) {
        assertEquals(result.error, "invalid_days");
      }
    });

    it("returns invalid_days error when days is empty string", () => {
      const url = new URL(`${BASE_URL}?days=`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, true);
      if ("error" in result) {
        assertEquals(result.error, "invalid_days");
      }
    });

    it("accepts days=7 as valid", () => {
      const url = new URL(`${BASE_URL}?days=7`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, false);
      if (!("error" in result)) {
        assertEquals(result.days, 7);
      }
    });

    it("accepts days=30 as valid", () => {
      const url = new URL(`${BASE_URL}?days=30`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, false);
      if (!("error" in result)) {
        assertEquals(result.days, 30);
      }
    });
  });

  describe("source parameter", () => {
    it("returns null source when not provided", () => {
      const url = new URL(BASE_URL);
      const result = parseQueryParams(url);

      assertEquals("error" in result, false);
      if (!("error" in result)) {
        assertEquals(result.source, null);
      }
    });

    it("returns source value when provided", () => {
      const url = new URL(`${BASE_URL}?source=macbook-pro-monitor`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, false);
      if (!("error" in result)) {
        assertEquals(result.source, "macbook-pro-monitor");
      }
    });

    it("returns source with days when both provided", () => {
      const url = new URL(`${BASE_URL}?days=30&source=test-monitor`);
      const result = parseQueryParams(url);

      assertEquals("error" in result, false);
      if (!("error" in result)) {
        assertEquals(result.days, 30);
        assertEquals(result.source, "test-monitor");
      }
    });
  });
});

// ============================================================================
// Integration-style Tests (using mock handler simulation)
// ============================================================================

/**
 * Simulated request handler for integration testing
 * Mirrors the auth and validation logic from index.ts without database access
 */
function simulateRequestHandler(request: Request): Response {
  // Only allow GET requests
  if (request.method !== "GET") {
    return new Response(
      JSON.stringify({
        error: "method_not_allowed",
        message: "Only GET method is allowed",
      }),
      { status: 405, headers: { "Content-Type": "application/json" } },
    );
  }

  // Verify bearer token authentication
  const authResult = verifyQueryAuth(request);
  if (!authResult.isValid) {
    const authHeader = request.headers.get("Authorization");
    if (!authHeader) {
      return new Response(
        JSON.stringify({
          error: "missing_auth",
          message: "Authorization header is required",
        }),
        { status: 401, headers: { "Content-Type": "application/json" } },
      );
    }
    return new Response(
      JSON.stringify({
        error: "invalid_token",
        message: "Bearer token is invalid",
      }),
      { status: 401, headers: { "Content-Type": "application/json" } },
    );
  }

  // Parse and validate query parameters
  const url = new URL(request.url);
  const params = parseQueryParams(url);

  if ("error" in params) {
    return new Response(JSON.stringify(params), {
      status: 400,
      headers: { "Content-Type": "application/json" },
    });
  }

  // Return mock success response (database stubbed)
  return new Response(
    JSON.stringify({
      aggregates: [],
      meta: {
        totalCount: 0,
        daysRequested: params.days,
        fetchedAt: new Date().toISOString(),
      },
    }),
    { status: 200, headers: { "Content-Type": "application/json" } },
  );
}

describe("Query Edge Function Integration", () => {
  const envGuard = new EnvGuard();

  beforeEach(() => {
    envGuard.save("VIBETEA_SUBSCRIBER_TOKEN");
    Deno.env.set("VIBETEA_SUBSCRIBER_TOKEN", TEST_SUBSCRIBER_TOKEN);
  });

  afterEach(() => {
    envGuard.restore();
  });

  describe("Authentication Responses", () => {
    it("returns 401 missing_auth when Authorization header is missing", async () => {
      const request = createMockRequest({ authHeader: null });
      const response = await simulateRequestHandler(request);
      const body = await parseJsonResponse<ErrorResponse>(response);

      assertEquals(response.status, 401);
      assertEquals(body.error, "missing_auth");
      assertEquals(body.message, "Authorization header is required");
    });

    it("returns 401 invalid_token when Authorization header lacks Bearer prefix", async () => {
      const request = createMockRequest({
        authHeader: "Basic dXNlcjpwYXNz",
      });
      const response = await simulateRequestHandler(request);
      const body = await parseJsonResponse<ErrorResponse>(response);

      assertEquals(response.status, 401);
      assertEquals(body.error, "invalid_token");
      assertEquals(body.message, "Bearer token is invalid");
    });

    it("returns 401 invalid_token when bearer token is wrong", async () => {
      const request = createMockRequest({
        authHeader: "Bearer wrong-token-value",
      });
      const response = await simulateRequestHandler(request);
      const body = await parseJsonResponse<ErrorResponse>(response);

      assertEquals(response.status, 401);
      assertEquals(body.error, "invalid_token");
      assertEquals(body.message, "Bearer token is invalid");
    });

    it("returns 401 invalid_token when Bearer has no token", async () => {
      const request = createMockRequest({
        authHeader: "Bearer ",
      });
      const response = await simulateRequestHandler(request);
      const body = await parseJsonResponse<ErrorResponse>(response);

      assertEquals(response.status, 401);
      assertEquals(body.error, "invalid_token");
      assertEquals(body.message, "Bearer token is invalid");
    });

    it("returns 200 when bearer token is valid", async () => {
      const request = createMockRequest({
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);

      assertEquals(response.status, 200);
    });
  });

  describe("Query Parameter Responses", () => {
    it("returns 400 invalid_days when days=5", async () => {
      const request = createMockRequest({
        url: `${BASE_URL}?days=5`,
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);
      const body = await parseJsonResponse<ErrorResponse>(response);

      assertEquals(response.status, 400);
      assertEquals(body.error, "invalid_days");
      assertEquals(body.message, "days parameter must be 7 or 30");
    });

    it("returns 400 invalid_days when days=0", async () => {
      const request = createMockRequest({
        url: `${BASE_URL}?days=0`,
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);
      const body = await parseJsonResponse<ErrorResponse>(response);

      assertEquals(response.status, 400);
      assertEquals(body.error, "invalid_days");
    });

    it("returns 200 when days=7", async () => {
      const request = createMockRequest({
        url: `${BASE_URL}?days=7`,
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);

      assertEquals(response.status, 200);

      const body = await response.json();
      assertEquals(body.meta.daysRequested, 7);
    });

    it("returns 200 when days=30", async () => {
      const request = createMockRequest({
        url: `${BASE_URL}?days=30`,
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);

      assertEquals(response.status, 200);

      const body = await response.json();
      assertEquals(body.meta.daysRequested, 30);
    });

    it("defaults to days=7 when not specified", async () => {
      const request = createMockRequest({
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);

      assertEquals(response.status, 200);

      const body = await response.json();
      assertEquals(body.meta.daysRequested, 7);
    });
  });

  describe("HTTP Method Validation", () => {
    it("returns 405 method_not_allowed for POST requests", async () => {
      const request = createMockRequest({
        method: "POST",
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);
      const body = await parseJsonResponse<ErrorResponse>(response);

      assertEquals(response.status, 405);
      assertEquals(body.error, "method_not_allowed");
      assertEquals(body.message, "Only GET method is allowed");
    });

    it("returns 405 method_not_allowed for PUT requests", async () => {
      const request = createMockRequest({
        method: "PUT",
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);

      assertEquals(response.status, 405);
    });

    it("returns 405 method_not_allowed for DELETE requests", async () => {
      const request = createMockRequest({
        method: "DELETE",
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);

      assertEquals(response.status, 405);
    });
  });

  describe("Response Headers", () => {
    it("returns Content-Type: application/json on success", async () => {
      const request = createMockRequest({
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);

      assertEquals(response.headers.get("Content-Type"), "application/json");
    });

    it("returns Content-Type: application/json on auth error", async () => {
      const request = createMockRequest({ authHeader: null });
      const response = await simulateRequestHandler(request);

      assertEquals(response.headers.get("Content-Type"), "application/json");
    });

    it("returns Content-Type: application/json on validation error", async () => {
      const request = createMockRequest({
        url: `${BASE_URL}?days=99`,
        authHeader: `Bearer ${TEST_SUBSCRIBER_TOKEN}`,
      });
      const response = await simulateRequestHandler(request);

      assertEquals(response.headers.get("Content-Type"), "application/json");
    });
  });
});

// ============================================================================
// Edge Cases and Security Tests
// ============================================================================

describe("Security and Edge Cases", () => {
  const envGuard = new EnvGuard();

  beforeEach(() => {
    envGuard.save("VIBETEA_SUBSCRIBER_TOKEN");
    Deno.env.set("VIBETEA_SUBSCRIBER_TOKEN", TEST_SUBSCRIBER_TOKEN);
  });

  afterEach(() => {
    envGuard.restore();
  });

  it("rejects token with extra whitespace", () => {
    const result = validateBearerToken(`Bearer  ${TEST_SUBSCRIBER_TOKEN}`);
    assertEquals(result, false);
  });

  it("rejects token with leading whitespace", () => {
    const result = validateBearerToken(` Bearer ${TEST_SUBSCRIBER_TOKEN}`);
    assertEquals(result, false);
  });

  it("rejects token with trailing whitespace in token value", () => {
    const result = validateBearerToken(`Bearer ${TEST_SUBSCRIBER_TOKEN} `);
    assertEquals(result, false);
  });

  it("rejects token that is a prefix of the correct token", () => {
    const partialToken = TEST_SUBSCRIBER_TOKEN.slice(0, -5);
    const result = validateBearerToken(`Bearer ${partialToken}`);
    assertEquals(result, false);
  });

  it("rejects token that is a suffix of the correct token", () => {
    const partialToken = TEST_SUBSCRIBER_TOKEN.slice(5);
    const result = validateBearerToken(`Bearer ${partialToken}`);
    assertEquals(result, false);
  });

  it("handles very long invalid tokens", () => {
    const longToken = "x".repeat(10000);
    const result = validateBearerToken(`Bearer ${longToken}`);
    assertEquals(result, false);
  });

  it("handles special characters in token comparison", () => {
    // Ensure the function handles tokens with special regex characters
    const specialToken = "token-with-$pecial.chars*";
    Deno.env.set("VIBETEA_SUBSCRIBER_TOKEN", specialToken);
    const result = validateBearerToken(`Bearer ${specialToken}`);
    assertEquals(result, true);
  });

  it("is case-sensitive for token comparison", () => {
    const result = validateBearerToken(
      `Bearer ${TEST_SUBSCRIBER_TOKEN.toUpperCase()}`,
    );
    assertEquals(result, false);
  });
});
