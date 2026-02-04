/**
 * MSW (Mock Service Worker) handlers for VibeTea query endpoint.
 *
 * Provides mock handlers that simulate the Supabase edge function
 * behavior for testing client-side data fetching.
 */

import { http, HttpResponse } from 'msw';
import { MOCK_BEARER_TOKEN, createQueryResponse, errorResponses } from './data';

/**
 * Extracts and validates the bearer token from Authorization header.
 *
 * @param request - The incoming request object
 * @returns The token if present and valid format, null otherwise
 */
function extractBearerToken(request: Request): string | null {
  const authHeader = request.headers.get('Authorization');

  if (authHeader === null) {
    return null;
  }

  const parts = authHeader.split(' ');
  if (parts.length !== 2 || parts[0] !== 'Bearer') {
    return null;
  }

  return parts[1] ?? null;
}

/**
 * Parses and validates the days query parameter.
 *
 * @param url - The request URL
 * @returns The validated days value (7 or 30), or null if invalid
 */
function parseDaysParam(url: URL): 7 | 30 | null {
  const daysParam = url.searchParams.get('days');

  // Default to 7 if not provided (per API spec)
  if (daysParam === null) {
    return 7;
  }

  const days = parseInt(daysParam, 10);

  if (days !== 7 && days !== 30) {
    return null;
  }

  return days;
}

/**
 * Handler for GET /functions/v1/query endpoint.
 *
 * Validates:
 * - Authorization header with bearer token
 * - days query parameter (must be 7 or 30)
 *
 * Returns:
 * - 401 for missing/invalid authentication
 * - 400 for invalid days parameter
 * - 200 with mock aggregates on success
 */
const queryHandler = http.get('*/functions/v1/query', ({ request }) => {
  // Step 1: Validate Authorization header
  const token = extractBearerToken(request);

  if (token === null) {
    return HttpResponse.json(errorResponses.missingAuth, { status: 401 });
  }

  if (token !== MOCK_BEARER_TOKEN) {
    return HttpResponse.json(errorResponses.invalidToken, { status: 401 });
  }

  // Step 2: Validate days parameter
  const url = new URL(request.url);
  const days = parseDaysParam(url);

  if (days === null) {
    return HttpResponse.json(errorResponses.invalidDays, { status: 400 });
  }

  // Step 3: Return mock data
  const response = createQueryResponse(days);

  return HttpResponse.json(response, { status: 200 });
});

/**
 * All query-related MSW handlers.
 *
 * Export this array and spread it into your MSW server handlers
 * to enable mocking for the query endpoint.
 *
 * @example
 * ```ts
 * import { setupServer } from 'msw/node';
 * import { queryHandlers } from './handlers';
 *
 * const server = setupServer(...queryHandlers);
 * ```
 */
export const queryHandlers = [queryHandler] as const;
