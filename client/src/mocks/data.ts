/**
 * Mock data factories for VibeTea query endpoint testing.
 *
 * Provides factory functions to generate realistic HourlyAggregate
 * test data for various testing scenarios.
 */

import type { HourlyAggregate } from '../types/events';

/**
 * Response format for the query edge function.
 */
export interface QueryResponse {
  readonly aggregates: HourlyAggregate[];
  readonly meta: {
    readonly totalCount: number;
    readonly daysRequested: 7 | 30;
    readonly fetchedAt: string;
  };
}

/**
 * Error response format for the query edge function.
 */
export interface QueryErrorResponse {
  readonly error: string;
  readonly message: string;
}

/**
 * Default mock source identifier used in test data.
 */
export const MOCK_SOURCE = 'test-monitor';

/**
 * Valid bearer token for testing authentication.
 */
export const MOCK_BEARER_TOKEN = 'test-bearer-token';

/**
 * Creates a single HourlyAggregate with specified or default values.
 *
 * @param overrides - Partial HourlyAggregate to override default values
 * @returns A complete HourlyAggregate object
 */
export function createHourlyAggregate(
  overrides: Partial<HourlyAggregate> = {}
): HourlyAggregate {
  const now = new Date();
  const defaultDate = now.toISOString().split('T')[0] ?? '2026-02-04';

  return {
    source: MOCK_SOURCE,
    date: defaultDate,
    hour: now.getUTCHours(),
    eventCount: Math.floor(Math.random() * 200) + 10,
    ...overrides,
  };
}

/**
 * Generates an array of HourlyAggregates for a specified number of days.
 *
 * Creates realistic mock data with:
 * - Variable event counts (higher during work hours 9-17)
 * - Some hours with zero events (simulating off hours)
 * - Consistent source identifier
 *
 * @param days - Number of days to generate (7 or 30)
 * @param source - Optional source identifier override
 * @returns Array of HourlyAggregate objects sorted by date/hour descending
 */
export function generateMockAggregates(
  days: 7 | 30,
  source: string = MOCK_SOURCE
): HourlyAggregate[] {
  const aggregates: HourlyAggregate[] = [];
  const now = new Date();

  for (let dayOffset = 0; dayOffset < days; dayOffset++) {
    const date = new Date(now);
    date.setUTCDate(date.getUTCDate() - dayOffset);
    const dateStr = date.toISOString().split('T')[0];

    if (dateStr === undefined) {
      continue;
    }

    // Generate aggregates for working hours (simulate typical usage)
    // Not all hours will have data - this is realistic
    for (let hour = 0; hour < 24; hour++) {
      // Skip some hours randomly to simulate gaps
      if (Math.random() < 0.3) {
        continue;
      }

      // Higher event counts during work hours (9-17)
      const isWorkHour = hour >= 9 && hour <= 17;
      const baseCount = isWorkHour ? 80 : 20;
      const variance = isWorkHour ? 120 : 30;
      const eventCount = baseCount + Math.floor(Math.random() * variance);

      aggregates.push({
        source,
        date: dateStr,
        hour,
        eventCount,
      });
    }
  }

  // Sort by date descending, then hour descending
  return aggregates.sort((a, b) => {
    const dateCompare = b.date.localeCompare(a.date);
    if (dateCompare !== 0) {
      return dateCompare;
    }
    return b.hour - a.hour;
  });
}

/**
 * Creates a complete QueryResponse with generated aggregates.
 *
 * @param days - Number of days of data to include
 * @param source - Optional source identifier override
 * @returns A complete QueryResponse object
 */
export function createQueryResponse(
  days: 7 | 30,
  source: string = MOCK_SOURCE
): QueryResponse {
  const aggregates = generateMockAggregates(days, source);

  return {
    aggregates,
    meta: {
      totalCount: aggregates.length,
      daysRequested: days,
      fetchedAt: new Date().toISOString(),
    },
  };
}

/**
 * Creates an empty QueryResponse (for testing empty state).
 *
 * @param days - Number of days requested
 * @returns A QueryResponse with no aggregates
 */
export function createEmptyQueryResponse(days: 7 | 30): QueryResponse {
  return {
    aggregates: [],
    meta: {
      totalCount: 0,
      daysRequested: days,
      fetchedAt: new Date().toISOString(),
    },
  };
}

/**
 * Creates error responses for testing error handling.
 */
export const errorResponses = {
  missingAuth: {
    error: 'missing_auth',
    message: 'Authorization header is required',
  } satisfies QueryErrorResponse,

  invalidToken: {
    error: 'invalid_token',
    message: 'Bearer token is invalid',
  } satisfies QueryErrorResponse,

  invalidDays: {
    error: 'invalid_days',
    message: 'days parameter must be 7 or 30',
  } satisfies QueryErrorResponse,

  internalError: {
    error: 'internal_error',
    message: 'Database query failed',
  } satisfies QueryErrorResponse,
} as const;
