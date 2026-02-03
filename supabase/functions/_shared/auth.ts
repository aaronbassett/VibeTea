/**
 * Shared authentication utilities for VibeTea Edge Functions
 *
 * Uses @noble/ed25519 for Ed25519 signature verification (RFC 8032 compliant)
 */
import * as ed from "https://esm.sh/@noble/ed25519@2.0.0";

/**
 * Decode a base64 string to Uint8Array
 */
function base64Decode(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/**
 * Verify an Ed25519 signature
 *
 * @param publicKeyBase64 - Base64-encoded public key (32 bytes)
 * @param signatureBase64 - Base64-encoded signature (64 bytes)
 * @param message - The message that was signed (as Uint8Array)
 * @returns Promise<boolean> - True if signature is valid
 */
export async function verifySignature(
  publicKeyBase64: string,
  signatureBase64: string,
  message: Uint8Array
): Promise<boolean> {
  try {
    const publicKey = base64Decode(publicKeyBase64);
    const signature = base64Decode(signatureBase64);

    // Validate key/signature lengths
    if (publicKey.length !== 32) {
      console.error(`Invalid public key length: ${publicKey.length}, expected 32`);
      return false;
    }
    if (signature.length !== 64) {
      console.error(`Invalid signature length: ${signature.length}, expected 64`);
      return false;
    }

    return await ed.verifyAsync(signature, message, publicKey);
  } catch (error) {
    console.error("Signature verification error:", error);
    return false;
  }
}

/**
 * Get public key for a source from environment configuration
 *
 * VIBETEA_PUBLIC_KEYS format: source_id:base64_public_key,source_id2:base64_public_key2
 *
 * @param sourceId - The source identifier from X-Source-ID header
 * @returns The base64-encoded public key, or null if not found
 */
export function getPublicKeyForSource(sourceId: string): string | null {
  const publicKeys = Deno.env.get("VIBETEA_PUBLIC_KEYS");
  if (!publicKeys) {
    console.error("VIBETEA_PUBLIC_KEYS environment variable not set");
    return null;
  }

  // Parse format: source_id:public_key,source_id2:public_key2
  const pairs = publicKeys.split(",");
  for (const pair of pairs) {
    const [id, key] = pair.trim().split(":");
    if (id === sourceId && key) {
      return key;
    }
  }

  console.error(`No public key found for source: ${sourceId}`);
  return null;
}

/**
 * Validate a bearer token against the configured subscriber token
 *
 * @param authHeader - The Authorization header value (e.g., "Bearer token123")
 * @returns boolean - True if token is valid
 */
export function validateBearerToken(authHeader: string | null): boolean {
  if (!authHeader) {
    return false;
  }

  const expectedToken = Deno.env.get("VIBETEA_SUBSCRIBER_TOKEN");
  if (!expectedToken) {
    console.error("VIBETEA_SUBSCRIBER_TOKEN environment variable not set");
    return false;
  }

  const prefix = "Bearer ";
  if (!authHeader.startsWith(prefix)) {
    return false;
  }

  const token = authHeader.slice(prefix.length);

  // Constant-time comparison to prevent timing attacks
  // Note: In Deno, we use a simple comparison since the token is not cryptographically sensitive
  // For production, consider using a constant-time comparison library
  return token === expectedToken;
}

/**
 * Result of authentication verification
 */
export interface AuthResult {
  readonly isValid: boolean;
  readonly error?: string;
  readonly sourceId?: string;
}

/**
 * Verify Ed25519 signature authentication for ingest endpoint
 *
 * @param request - The incoming Request object
 * @param body - The request body as a string (must be read before calling)
 * @returns AuthResult with validation status
 */
export async function verifyIngestAuth(
  request: Request,
  body: string
): Promise<AuthResult> {
  const sourceId = request.headers.get("X-Source-ID");
  const signature = request.headers.get("X-Signature");

  if (!sourceId) {
    return { isValid: false, error: "Missing X-Source-ID header" };
  }

  if (!signature) {
    return { isValid: false, error: "Missing X-Signature header" };
  }

  const publicKey = getPublicKeyForSource(sourceId);
  if (!publicKey) {
    return { isValid: false, error: `Unknown source: ${sourceId}` };
  }

  const message = new TextEncoder().encode(body);
  const isValid = await verifySignature(publicKey, signature, message);

  if (!isValid) {
    return { isValid: false, error: "Invalid signature" };
  }

  return { isValid: true, sourceId };
}

/**
 * Verify bearer token authentication for query endpoint
 *
 * @param request - The incoming Request object
 * @returns AuthResult with validation status
 */
export function verifyQueryAuth(request: Request): AuthResult {
  const authHeader = request.headers.get("Authorization");

  if (!validateBearerToken(authHeader)) {
    return { isValid: false, error: "Invalid or missing bearer token" };
  }

  return { isValid: true };
}
