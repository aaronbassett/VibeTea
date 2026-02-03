/**
 * Unit tests for the VibeTea Ingest Edge Function authentication
 *
 * Tests Ed25519 signature authentication per the ingest.yaml contract.
 * Focus is on authentication flow - database operations are not tested here.
 *
 * @see specs/001-supabase-persistence/contracts/ingest.yaml
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.224.0/assert/mod.ts";
import * as ed from "https://esm.sh/@noble/ed25519@2.0.0";

// Import the auth utilities for direct testing
import {
  verifyIngestAuth,
  verifySignature,
  getPublicKeyForSource,
  type AuthResult,
} from "../_shared/auth.ts";

/**
 * Test Ed25519 key pair for the "test-monitor" source
 * Generated using @noble/ed25519
 */
const TEST_PRIVATE_KEY = new Uint8Array([
  0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60,
  0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec, 0x2c, 0xc4,
  0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19,
  0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae, 0x7f, 0x60,
]);

/**
 * Test source identifier
 */
const TEST_SOURCE_ID = "test-monitor";

/**
 * Alternative test source with different key
 */
const ALT_SOURCE_ID = "alt-monitor";
const ALT_PRIVATE_KEY = new Uint8Array([
  0x4c, 0xcd, 0x08, 0x9b, 0x28, 0xff, 0x96, 0xda,
  0x9d, 0xb6, 0xc3, 0x46, 0xec, 0x11, 0x4e, 0x0f,
  0x5b, 0x8a, 0x31, 0x9f, 0x35, 0xab, 0xa6, 0x24,
  0xda, 0x8c, 0xf6, 0xed, 0x4f, 0xb8, 0xa6, 0xfb,
]);

/**
 * Test event data that matches the Event schema
 */
const TEST_EVENT = {
  id: "evt_k7m2n9p4q1r6s3t8u5v0",
  source: TEST_SOURCE_ID,
  timestamp: "2026-02-03T14:30:00Z",
  eventType: "tool",
  payload: {
    sessionId: "550e8400-e29b-41d4-a716-446655440000",
    tool: "Read",
    status: "completed",
  },
};

/**
 * Helper to encode bytes to base64
 */
function base64Encode(bytes: Uint8Array): string {
  return btoa(String.fromCharCode(...bytes));
}

/**
 * Helper to get the public key from a private key
 */
async function getPublicKey(privateKey: Uint8Array): Promise<Uint8Array> {
  return await ed.getPublicKeyAsync(privateKey);
}

/**
 * Helper to sign a message with a private key
 */
async function signMessage(
  privateKey: Uint8Array,
  message: string
): Promise<string> {
  const messageBytes = new TextEncoder().encode(message);
  const signature = await ed.signAsync(messageBytes, privateKey);
  return base64Encode(signature);
}

/**
 * Sets up the VIBETEA_PUBLIC_KEYS environment variable for testing
 */
async function setupTestKeys(): Promise<void> {
  const testPublicKey = await getPublicKey(TEST_PRIVATE_KEY);
  const altPublicKey = await getPublicKey(ALT_PRIVATE_KEY);
  const keysConfig = `${TEST_SOURCE_ID}:${base64Encode(testPublicKey)},${ALT_SOURCE_ID}:${base64Encode(altPublicKey)}`;
  Deno.env.set("VIBETEA_PUBLIC_KEYS", keysConfig);
}

/**
 * Cleans up test environment variables
 */
function cleanupTestKeys(): void {
  Deno.env.delete("VIBETEA_PUBLIC_KEYS");
}

/**
 * Create a mock Request with the specified headers and body
 */
function createMockRequest(
  options: {
    sourceId?: string;
    signature?: string;
    body?: string;
  }
): Request {
  const headers = new Headers();
  if (options.sourceId !== undefined) {
    headers.set("X-Source-ID", options.sourceId);
  }
  if (options.signature !== undefined) {
    headers.set("X-Signature", options.signature);
  }
  headers.set("Content-Type", "application/json");

  return new Request("https://example.com/ingest", {
    method: "POST",
    headers,
    body: options.body ?? JSON.stringify([TEST_EVENT]),
  });
}

// =============================================================================
// Test Suite: Missing Headers (401 missing_auth)
// =============================================================================

Deno.test("ingest auth: returns error when X-Source-ID header is missing", async () => {
  await setupTestKeys();
  try {
    const body = JSON.stringify([TEST_EVENT]);
    const signature = await signMessage(TEST_PRIVATE_KEY, body);
    const request = createMockRequest({ signature, body });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, false);
    assertEquals(result.error, "Missing X-Source-ID header");
    assertEquals(result.sourceId, undefined);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: returns error when X-Signature header is missing", async () => {
  await setupTestKeys();
  try {
    const body = JSON.stringify([TEST_EVENT]);
    const request = createMockRequest({ sourceId: TEST_SOURCE_ID, body });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, false);
    assertEquals(result.error, "Missing X-Signature header");
    assertEquals(result.sourceId, undefined);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: returns error when both headers are missing", async () => {
  await setupTestKeys();
  try {
    const body = JSON.stringify([TEST_EVENT]);
    const request = createMockRequest({ body });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, false);
    // X-Source-ID is checked first
    assertEquals(result.error, "Missing X-Source-ID header");
    assertEquals(result.sourceId, undefined);
  } finally {
    cleanupTestKeys();
  }
});

// =============================================================================
// Test Suite: Unknown Source (401 unknown_source)
// =============================================================================

Deno.test("ingest auth: returns error when source ID is not configured", async () => {
  await setupTestKeys();
  try {
    const body = JSON.stringify([TEST_EVENT]);
    const signature = await signMessage(TEST_PRIVATE_KEY, body);
    const request = createMockRequest({
      sourceId: "unknown-monitor",
      signature,
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, false);
    assertEquals(result.error, "Unknown source: unknown-monitor");
    assertEquals(result.sourceId, undefined);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: returns error when VIBETEA_PUBLIC_KEYS is not set", async () => {
  // Ensure the env var is not set
  cleanupTestKeys();

  const body = JSON.stringify([TEST_EVENT]);
  const signature = await signMessage(TEST_PRIVATE_KEY, body);
  const request = createMockRequest({
    sourceId: TEST_SOURCE_ID,
    signature,
    body,
  });

  const result: AuthResult = await verifyIngestAuth(request, body);

  assertEquals(result.isValid, false);
  assertEquals(result.error, `Unknown source: ${TEST_SOURCE_ID}`);
  assertEquals(result.sourceId, undefined);
});

// =============================================================================
// Test Suite: Invalid Signature (401 invalid_signature)
// =============================================================================

Deno.test("ingest auth: returns error when signature is invalid", async () => {
  await setupTestKeys();
  try {
    const body = JSON.stringify([TEST_EVENT]);
    // Sign with a different key than what's registered for TEST_SOURCE_ID
    const wrongSignature = await signMessage(ALT_PRIVATE_KEY, body);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature: wrongSignature,
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, false);
    assertEquals(result.error, "Invalid signature");
    assertEquals(result.sourceId, undefined);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: returns error when signature does not match body", async () => {
  await setupTestKeys();
  try {
    const originalBody = JSON.stringify([TEST_EVENT]);
    // Sign the original body
    const signature = await signMessage(TEST_PRIVATE_KEY, originalBody);

    // But send a different body
    const tamperedBody = JSON.stringify([{ ...TEST_EVENT, id: "evt_tampereddata00000000" }]);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature,
      body: tamperedBody,
    });

    const result: AuthResult = await verifyIngestAuth(request, tamperedBody);

    assertEquals(result.isValid, false);
    assertEquals(result.error, "Invalid signature");
    assertEquals(result.sourceId, undefined);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: returns error when signature is malformed base64", async () => {
  await setupTestKeys();
  try {
    const body = JSON.stringify([TEST_EVENT]);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature: "not-valid-base64!!!",
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, false);
    assertEquals(result.error, "Invalid signature");
    assertEquals(result.sourceId, undefined);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: returns error when signature has wrong length", async () => {
  await setupTestKeys();
  try {
    const body = JSON.stringify([TEST_EVENT]);
    // Create a signature that's too short (32 bytes instead of 64)
    const shortSignature = base64Encode(new Uint8Array(32));
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature: shortSignature,
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, false);
    assertEquals(result.error, "Invalid signature");
    assertEquals(result.sourceId, undefined);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: returns error when signature is empty", async () => {
  await setupTestKeys();
  try {
    const body = JSON.stringify([TEST_EVENT]);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature: "",
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, false);
    // Empty string still counts as missing the header content-wise
    // but the header is technically present, so we get invalid signature
    assertEquals(result.error, "Invalid signature");
    assertEquals(result.sourceId, undefined);
  } finally {
    cleanupTestKeys();
  }
});

// =============================================================================
// Test Suite: Valid Authentication
// =============================================================================

Deno.test("ingest auth: succeeds with valid signature", async () => {
  await setupTestKeys();
  try {
    const body = JSON.stringify([TEST_EVENT]);
    const signature = await signMessage(TEST_PRIVATE_KEY, body);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature,
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, true);
    assertEquals(result.error, undefined);
    assertEquals(result.sourceId, TEST_SOURCE_ID);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: succeeds with alternative source and valid signature", async () => {
  await setupTestKeys();
  try {
    const event = { ...TEST_EVENT, source: ALT_SOURCE_ID };
    const body = JSON.stringify([event]);
    const signature = await signMessage(ALT_PRIVATE_KEY, body);
    const request = createMockRequest({
      sourceId: ALT_SOURCE_ID,
      signature,
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, true);
    assertEquals(result.error, undefined);
    assertEquals(result.sourceId, ALT_SOURCE_ID);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: succeeds with empty body array", async () => {
  await setupTestKeys();
  try {
    const body = "[]";
    const signature = await signMessage(TEST_PRIVATE_KEY, body);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature,
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    // Auth passes even with empty array - that's a validation concern, not auth
    assertEquals(result.isValid, true);
    assertEquals(result.error, undefined);
    assertEquals(result.sourceId, TEST_SOURCE_ID);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: succeeds with large body", async () => {
  await setupTestKeys();
  try {
    // Create a large array of events
    const events = Array.from({ length: 100 }, (_, i) => ({
      ...TEST_EVENT,
      id: `evt_${String(i).padStart(20, "0")}`,
    }));
    const body = JSON.stringify(events);
    const signature = await signMessage(TEST_PRIVATE_KEY, body);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature,
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, true);
    assertEquals(result.error, undefined);
    assertEquals(result.sourceId, TEST_SOURCE_ID);
  } finally {
    cleanupTestKeys();
  }
});

// =============================================================================
// Test Suite: verifySignature utility function
// =============================================================================

Deno.test("verifySignature: returns true for valid signature", async () => {
  const testPublicKey = await getPublicKey(TEST_PRIVATE_KEY);
  const message = "test message";
  const signature = await signMessage(TEST_PRIVATE_KEY, message);
  const messageBytes = new TextEncoder().encode(message);

  const result = await verifySignature(
    base64Encode(testPublicKey),
    signature,
    messageBytes
  );

  assertEquals(result, true);
});

Deno.test("verifySignature: returns false for invalid signature", async () => {
  const testPublicKey = await getPublicKey(TEST_PRIVATE_KEY);
  const altPublicKey = await getPublicKey(ALT_PRIVATE_KEY);
  const message = "test message";
  // Sign with alt key but verify with test key
  const signature = await signMessage(ALT_PRIVATE_KEY, message);
  const messageBytes = new TextEncoder().encode(message);

  const result = await verifySignature(
    base64Encode(testPublicKey),
    signature,
    messageBytes
  );

  assertEquals(result, false);
});

Deno.test("verifySignature: returns false for invalid public key length", async () => {
  const message = "test message";
  const signature = await signMessage(TEST_PRIVATE_KEY, message);
  const messageBytes = new TextEncoder().encode(message);
  // Use a 16-byte key instead of 32
  const invalidKey = base64Encode(new Uint8Array(16));

  const result = await verifySignature(invalidKey, signature, messageBytes);

  assertEquals(result, false);
});

Deno.test("verifySignature: returns false for invalid signature length", async () => {
  const testPublicKey = await getPublicKey(TEST_PRIVATE_KEY);
  const message = "test message";
  const messageBytes = new TextEncoder().encode(message);
  // Use a 32-byte signature instead of 64
  const invalidSignature = base64Encode(new Uint8Array(32));

  const result = await verifySignature(
    base64Encode(testPublicKey),
    invalidSignature,
    messageBytes
  );

  assertEquals(result, false);
});

// =============================================================================
// Test Suite: getPublicKeyForSource utility function
// =============================================================================

Deno.test("getPublicKeyForSource: returns key for configured source", async () => {
  await setupTestKeys();
  try {
    const result = getPublicKeyForSource(TEST_SOURCE_ID);

    assertExists(result);
    // Verify it's a valid base64 string that decodes to 32 bytes
    const decoded = Uint8Array.from(atob(result), (c) => c.charCodeAt(0));
    assertEquals(decoded.length, 32);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("getPublicKeyForSource: returns null for unknown source", async () => {
  await setupTestKeys();
  try {
    const result = getPublicKeyForSource("nonexistent-source");

    assertEquals(result, null);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("getPublicKeyForSource: returns null when env var is not set", () => {
  cleanupTestKeys();

  const result = getPublicKeyForSource(TEST_SOURCE_ID);

  assertEquals(result, null);
});

Deno.test("getPublicKeyForSource: handles multiple sources correctly", async () => {
  await setupTestKeys();
  try {
    const testKey = getPublicKeyForSource(TEST_SOURCE_ID);
    const altKey = getPublicKeyForSource(ALT_SOURCE_ID);

    assertExists(testKey);
    assertExists(altKey);
    // Keys should be different
    assertEquals(testKey !== altKey, true);
  } finally {
    cleanupTestKeys();
  }
});

// =============================================================================
// Test Suite: Edge Cases
// =============================================================================

Deno.test("ingest auth: handles unicode in body", async () => {
  await setupTestKeys();
  try {
    const event = {
      ...TEST_EVENT,
      payload: { message: "Hello, 世界! \u{1F600}" },
    };
    const body = JSON.stringify([event]);
    const signature = await signMessage(TEST_PRIVATE_KEY, body);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature,
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, true);
    assertEquals(result.sourceId, TEST_SOURCE_ID);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: handles special characters in body", async () => {
  await setupTestKeys();
  try {
    const event = {
      ...TEST_EVENT,
      payload: { path: "/foo/bar?baz=qux&x=1", special: '<script>alert("xss")</script>' },
    };
    const body = JSON.stringify([event]);
    const signature = await signMessage(TEST_PRIVATE_KEY, body);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature,
      body,
    });

    const result: AuthResult = await verifyIngestAuth(request, body);

    assertEquals(result.isValid, true);
    assertEquals(result.sourceId, TEST_SOURCE_ID);
  } finally {
    cleanupTestKeys();
  }
});

Deno.test("ingest auth: signature is sensitive to whitespace changes", async () => {
  await setupTestKeys();
  try {
    // Sign compact JSON
    const compactBody = JSON.stringify([TEST_EVENT]);
    const signature = await signMessage(TEST_PRIVATE_KEY, compactBody);

    // But send pretty-printed JSON (different bytes)
    const prettyBody = JSON.stringify([TEST_EVENT], null, 2);
    const request = createMockRequest({
      sourceId: TEST_SOURCE_ID,
      signature,
      body: prettyBody,
    });

    const result: AuthResult = await verifyIngestAuth(request, prettyBody);

    // Signature should fail because body bytes are different
    assertEquals(result.isValid, false);
    assertEquals(result.error, "Invalid signature");
  } finally {
    cleanupTestKeys();
  }
});
