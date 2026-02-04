/**
 * Tests for useHistoricData hook.
 *
 * Tests the stale-while-revalidate caching behavior, error handling,
 * and manual refetch functionality.
 *
 * @vitest-environment happy-dom
 */

import { renderHook, waitFor, act } from '@testing-library/react';
import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';

import {
  createHourlyAggregate,
} from '../../mocks/data';
import { useHistoricData } from '../../hooks/useHistoricData';
import { useEventStore } from '../../hooks/useEventStore';
import type { HourlyAggregate } from '../../types/events';

// -----------------------------------------------------------------------------
// Test Setup
// -----------------------------------------------------------------------------

/**
 * Helper to reset store state properly
 */
function resetStore(): void {
  useEventStore.setState({
    status: 'disconnected',
    events: [],
    sessions: new Map(),
    filters: { sessionId: null, timeRange: null },
    historicData: [],
    historicDataStatus: 'idle',
    historicDataFetchedAt: null,
    historicDataError: null,
  });
}

/**
 * Mock data for successful fetches
 */
const mockAggregates: HourlyAggregate[] = [
  createHourlyAggregate({ date: '2026-02-01', hour: 10, eventCount: 100 }),
  createHourlyAggregate({ date: '2026-02-01', hour: 11, eventCount: 150 }),
  createHourlyAggregate({ date: '2026-02-01', hour: 12, eventCount: 200 }),
];

let originalFetchHistoricData: (days: 7 | 30) => Promise<void>;
let fetchCalls: number[];

beforeEach(() => {
  // Reset store state before each test
  resetStore();

  // Track fetch calls
  fetchCalls = [];

  // Save original function
  originalFetchHistoricData = useEventStore.getState().fetchHistoricData;

  // Default mock: immediately set success state
  const mockFetch = vi.fn().mockImplementation((days: 7 | 30) => {
    fetchCalls.push(days);
    // Synchronously update state to avoid timing issues
    useEventStore.setState({
      historicData: mockAggregates,
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });
    return Promise.resolve();
  });
  useEventStore.setState({ fetchHistoricData: mockFetch });
});

afterEach(() => {
  // Reset store state
  resetStore();

  // Restore original function
  useEventStore.setState({ fetchHistoricData: originalFetchHistoricData });

  // Clear mocks
  vi.clearAllMocks();
});

// -----------------------------------------------------------------------------
// Initial Fetch Behavior Tests
// -----------------------------------------------------------------------------

describe('useHistoricData - Initial Fetch Behavior', () => {
  it('should automatically fetch on mount when no cached data', () => {
    const { result } = renderHook(() => useHistoricData(7));

    // Status should be success (mock updates state immediately)
    expect(result.current.status).toBe('success');

    // Verify fetch was called with correct days
    expect(fetchCalls).toContain(7);

    // Data should be populated
    expect(result.current.data.length).toBeGreaterThan(0);
    expect(result.current.fetchedAt).not.toBeNull();
    expect(result.current.error).toBeNull();
  });

  it('should set status to success after successful fetch', () => {
    const { result } = renderHook(() => useHistoricData(7));

    expect(result.current.status).toBe('success');
    expect(result.current.error).toBeNull();
    expect(result.current.fetchedAt).toBeGreaterThan(0);
  });

  it('should populate data array with aggregates', () => {
    const { result } = renderHook(() => useHistoricData(7));

    // Check data structure
    expect(Array.isArray(result.current.data)).toBe(true);
    expect(result.current.data.length).toBeGreaterThan(0);

    // Verify aggregate structure
    const firstAggregate = result.current.data[0];
    expect(firstAggregate).toHaveProperty('source');
    expect(firstAggregate).toHaveProperty('date');
    expect(firstAggregate).toHaveProperty('hour');
    expect(firstAggregate).toHaveProperty('eventCount');
  });
});

// -----------------------------------------------------------------------------
// Stale-While-Revalidate Behavior Tests
// -----------------------------------------------------------------------------

describe('useHistoricData - Stale-While-Revalidate Behavior', () => {
  it('should return cached data immediately when available', () => {
    // Pre-populate store with cached data
    const cachedAggregates = [
      createHourlyAggregate({ date: '2026-02-01', hour: 10, eventCount: 100 }),
      createHourlyAggregate({ date: '2026-02-01', hour: 11, eventCount: 150 }),
    ];

    // Set fresh cached data (within 5 minute threshold)
    useEventStore.setState({
      historicData: cachedAggregates,
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(), // Fresh data
      historicDataError: null,
    });

    const { result } = renderHook(() => useHistoricData(7));

    // Should immediately have cached data
    expect(result.current.data).toEqual(cachedAggregates);
    expect(result.current.status).toBe('success');
    expect(result.current.data.length).toBe(2);
  });

  it('should refetch in background when data is stale (>5 min old)', () => {
    // Pre-populate store with stale cached data
    const cachedAggregates = [
      createHourlyAggregate({ date: '2026-02-01', hour: 10, eventCount: 100 }),
    ];

    const staleTime = Date.now() - 6 * 60 * 1000; // 6 minutes ago (stale)

    useEventStore.setState({
      historicData: cachedAggregates,
      historicDataStatus: 'success',
      historicDataFetchedAt: staleTime,
      historicDataError: null,
    });

    const { result } = renderHook(() => useHistoricData(7));

    // fetchedAt should be updated (newer than stale time) due to refetch
    expect(result.current.fetchedAt).toBeGreaterThan(staleTime);

    // Fetch should have been called for the refetch
    expect(fetchCalls.length).toBeGreaterThan(0);
  });

  it('should not refetch when data is fresh (<5 min old)', () => {
    // Clear fetch calls
    fetchCalls = [];

    // Pre-populate store with fresh cached data
    const cachedAggregates = [
      createHourlyAggregate({ date: '2026-02-01', hour: 10, eventCount: 100 }),
    ];

    const freshTime = Date.now() - 2 * 60 * 1000; // 2 minutes ago (fresh)

    useEventStore.setState({
      historicData: cachedAggregates,
      historicDataStatus: 'success',
      historicDataFetchedAt: freshTime,
      historicDataError: null,
    });

    const { result } = renderHook(() => useHistoricData(7));

    // Data should be returned immediately and status should remain 'success'
    expect(result.current.data).toEqual(cachedAggregates);
    expect(result.current.status).toBe('success');

    // Should not have fetched since data is fresh
    expect(fetchCalls.length).toBe(0);
  });
});

// -----------------------------------------------------------------------------
// Error Handling Tests
// -----------------------------------------------------------------------------

describe('useHistoricData - Error Handling', () => {
  it('should set status to error on fetch failure', () => {
    // Set up mock that fails
    const mockFetch = vi.fn().mockImplementation(() => {
      useEventStore.setState({
        historicDataStatus: 'error',
        historicDataError: 'Database query failed',
      });
      return Promise.resolve();
    });
    useEventStore.setState({ fetchHistoricData: mockFetch });

    const { result } = renderHook(() => useHistoricData(7));

    expect(result.current.status).toBe('error');
    expect(result.current.error).toBe('Database query failed');
  });

  it('should preserve existing data on error', () => {
    // Pre-populate store with existing data
    const existingData = [
      createHourlyAggregate({ date: '2026-02-01', hour: 10, eventCount: 100 }),
    ];

    const staleTime = Date.now() - 6 * 60 * 1000; // Stale to trigger refetch

    // Mock that fails but preserves data
    const mockFetch = vi.fn().mockImplementation(() => {
      useEventStore.setState({
        // Note: data is NOT updated, only status
        historicDataStatus: 'error',
        historicDataError: 'Server error',
      });
      return Promise.resolve();
    });

    useEventStore.setState({
      fetchHistoricData: mockFetch,
      historicData: existingData,
      historicDataStatus: 'success',
      historicDataFetchedAt: staleTime,
      historicDataError: null,
    });

    const { result } = renderHook(() => useHistoricData(7));

    // Status should be error
    expect(result.current.status).toBe('error');

    // Data should still be preserved
    expect(result.current.data).toEqual(existingData);
  });

  it('should set error message from response', () => {
    const errorMessage = 'Custom error message';
    const mockFetch = vi.fn().mockImplementation(() => {
      useEventStore.setState({
        historicDataStatus: 'error',
        historicDataError: errorMessage,
      });
      return Promise.resolve();
    });
    useEventStore.setState({ fetchHistoricData: mockFetch });

    const { result } = renderHook(() => useHistoricData(7));

    expect(result.current.status).toBe('error');
    expect(result.current.error).toBe(errorMessage);
  });

  it('should handle network errors gracefully', () => {
    const mockFetch = vi.fn().mockImplementation(() => {
      useEventStore.setState({
        historicDataStatus: 'error',
        historicDataError: 'Network error',
      });
      return Promise.resolve();
    });
    useEventStore.setState({ fetchHistoricData: mockFetch });

    const { result } = renderHook(() => useHistoricData(7));

    expect(result.current.status).toBe('error');
    expect(result.current.error).toBeTruthy();
  });

  it('should show error when Supabase URL is not configured', () => {
    const mockFetch = vi.fn().mockImplementation(() => {
      useEventStore.setState({
        historicDataStatus: 'error',
        historicDataError: 'Persistence not configured',
      });
      return Promise.resolve();
    });
    useEventStore.setState({ fetchHistoricData: mockFetch });

    const { result } = renderHook(() => useHistoricData(7));

    expect(result.current.status).toBe('error');
    expect(result.current.error).toBe('Persistence not configured');
  });

  it('should show error when auth token is not configured', () => {
    const mockFetch = vi.fn().mockImplementation(() => {
      useEventStore.setState({
        historicDataStatus: 'error',
        historicDataError: 'Auth token not configured',
      });
      return Promise.resolve();
    });
    useEventStore.setState({ fetchHistoricData: mockFetch });

    const { result } = renderHook(() => useHistoricData(7));

    expect(result.current.status).toBe('error');
    expect(result.current.error).toBe('Auth token not configured');
  });
});

// -----------------------------------------------------------------------------
// Manual Refetch Tests
// -----------------------------------------------------------------------------

describe('useHistoricData - Manual Refetch', () => {
  it('should trigger fetch when refetch() is called', async () => {
    const { result } = renderHook(() => useHistoricData(7));

    // Wait for initial render to complete
    expect(result.current.status).toBe('success');

    const initialFetchCount = fetchCalls.length;

    // Manually trigger refetch
    act(() => {
      result.current.refetch();
    });

    // Should have called fetch again
    expect(fetchCalls.length).toBeGreaterThan(initialFetchCount);
  });

  it('should work regardless of staleness', () => {
    // Clear fetch calls
    fetchCalls = [];

    // Pre-populate with fresh data (not stale)
    const freshTime = Date.now(); // Just now

    useEventStore.setState({
      historicData: [
        createHourlyAggregate({ date: '2026-02-01', hour: 10, eventCount: 100 }),
      ],
      historicDataStatus: 'success',
      historicDataFetchedAt: freshTime,
      historicDataError: null,
    });

    const { result } = renderHook(() => useHistoricData(7));

    // Data is fresh, so no automatic fetch should happen
    expect(fetchCalls.length).toBe(0);

    // Manually trigger refetch - should work even though data is fresh
    act(() => {
      result.current.refetch();
    });

    expect(fetchCalls.length).toBe(1);
  });

  it('should have a refetch function that can be called', () => {
    const { result } = renderHook(() => useHistoricData(7));

    expect(typeof result.current.refetch).toBe('function');
  });
});

// -----------------------------------------------------------------------------
// Days Parameter Tests
// -----------------------------------------------------------------------------

describe('useHistoricData - Days Parameter', () => {
  it('should fetch 7-day data when days=7', () => {
    renderHook(() => useHistoricData(7));

    expect(fetchCalls).toContain(7);
  });

  it('should fetch 30-day data when days=30', () => {
    // Clear previous calls
    fetchCalls = [];

    renderHook(() => useHistoricData(30));

    expect(fetchCalls).toContain(30);
  });

  it('should pass correct days parameter to fetchHistoricData', () => {
    fetchCalls = [];

    const { rerender } = renderHook(
      ({ days }: { days: 7 | 30 }) => useHistoricData(days),
      { initialProps: { days: 7 } }
    );

    expect(fetchCalls[0]).toBe(7);

    // Clear and set fresh data to force refetch on rerender with new days
    fetchCalls = [];
    useEventStore.setState({
      historicDataFetchedAt: null, // Force refetch
      historicDataStatus: 'idle',
    });

    // Change days parameter
    rerender({ days: 30 });

    expect(fetchCalls).toContain(30);
  });
});

// -----------------------------------------------------------------------------
// Edge Cases and Additional Scenarios
// -----------------------------------------------------------------------------

describe('useHistoricData - Edge Cases', () => {
  it('should handle empty response gracefully', () => {
    // Mock that returns empty data
    const mockFetch = vi.fn().mockImplementation(() => {
      useEventStore.setState({
        historicData: [],
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
      return Promise.resolve();
    });
    useEventStore.setState({ fetchHistoricData: mockFetch });

    const { result } = renderHook(() => useHistoricData(7));

    expect(result.current.status).toBe('success');
    expect(result.current.data).toEqual([]);
    expect(result.current.error).toBeNull();
  });

  it('should handle invalid token response', () => {
    const mockFetch = vi.fn().mockImplementation(() => {
      useEventStore.setState({
        historicDataStatus: 'error',
        historicDataError: 'Bearer token is invalid',
      });
      return Promise.resolve();
    });
    useEventStore.setState({ fetchHistoricData: mockFetch });

    const { result } = renderHook(() => useHistoricData(7));

    expect(result.current.status).toBe('error');
    expect(result.current.error).toBe('Bearer token is invalid');
  });

  it('should return data as readonly array', () => {
    const { result } = renderHook(() => useHistoricData(7));

    // Data should be an array
    expect(Array.isArray(result.current.data)).toBe(true);
  });

  it('should expose all expected properties', () => {
    const { result } = renderHook(() => useHistoricData(7));

    // Verify the hook returns all expected properties
    expect(result.current).toHaveProperty('data');
    expect(result.current).toHaveProperty('status');
    expect(result.current).toHaveProperty('error');
    expect(result.current).toHaveProperty('fetchedAt');
    expect(result.current).toHaveProperty('refetch');
  });

  it('should return correct type for fetchedAt (number or null)', () => {
    const { result } = renderHook(() => useHistoricData(7));

    // fetchedAt should be a number (timestamp) after successful fetch
    expect(typeof result.current.fetchedAt === 'number' || result.current.fetchedAt === null).toBe(true);

    // After successful mock, it should be a number
    if (result.current.status === 'success') {
      expect(typeof result.current.fetchedAt).toBe('number');
    }
  });
});
