/**
 * Hook for fetching and caching historic event aggregates
 * with stale-while-revalidate pattern.
 *
 * This hook provides automatic background refetching when cached data
 * becomes stale (older than 5 minutes), while immediately returning
 * the cached data for a responsive user experience.
 *
 * @example
 * ```tsx
 * function HeatmapView() {
 *   const { data, status, error, refetch } = useHistoricData(7);
 *
 *   if (status === 'loading' && data.length === 0) {
 *     return <LoadingSpinner />;
 *   }
 *
 *   if (status === 'error') {
 *     return <ErrorMessage message={error} onRetry={refetch} />;
 *   }
 *
 *   return <Heatmap data={data} />;
 * }
 * ```
 */

import { useCallback, useEffect } from 'react';

import {
  useEventStore,
  type HistoricDataSnapshot,
  type HistoricDataStatus,
} from './useEventStore';
import type { HourlyAggregate } from '../types/events';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/**
 * Time threshold after which cached data is considered stale and should be refetched.
 * Set to 5 minutes (300,000 milliseconds).
 */
const STALE_THRESHOLD_MS = 5 * 60 * 1000;

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Return type for the useHistoricData hook.
 * Provides access to cached data, loading status, and manual refresh capability.
 */
export interface UseHistoricDataResult {
  /** The cached historic aggregate data (empty array if not yet fetched) */
  readonly data: readonly HourlyAggregate[];
  /** Current status of the data fetch operation */
  readonly status: HistoricDataStatus;
  /** Error message if the fetch failed, null otherwise */
  readonly error: string | null;
  /** Timestamp when data was last successfully fetched, null if never fetched */
  readonly fetchedAt: number | null;
  /** Function to manually trigger a data refresh, bypassing staleness check */
  readonly refetch: () => void;
}

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Determines if the cached data is stale based on the fetch timestamp.
 *
 * @param fetchedAt - Timestamp when data was last fetched, or null if never fetched
 * @returns true if data should be refetched (stale or never fetched)
 */
function isDataStale(fetchedAt: number | null): boolean {
  if (fetchedAt === null) {
    return true;
  }
  return Date.now() - fetchedAt > STALE_THRESHOLD_MS;
}

// -----------------------------------------------------------------------------
// Hook Implementation
// -----------------------------------------------------------------------------

/**
 * Hook for fetching and caching historic event aggregates with
 * stale-while-revalidate caching strategy.
 *
 * Features:
 * - Automatic background refresh when data is stale (>5 minutes old)
 * - Immediate return of cached data while revalidating
 * - Manual refetch capability for user-triggered refreshes
 * - Proper loading and error state handling
 *
 * @param days - Number of days of historic data to fetch (7 or 30)
 * @returns Object containing data, status, error, fetchedAt, and refetch function
 */
export function useHistoricData(days: 7 | 30): UseHistoricDataResult {
  // Select state from the store using individual selectors for optimal re-renders
  const historicData = useEventStore((state) => state.historicData);
  const historicDataStatus = useEventStore((state) => state.historicDataStatus);
  const historicDataFetchedAt = useEventStore(
    (state) => state.historicDataFetchedAt
  );
  const historicDataError = useEventStore((state) => state.historicDataError);
  const fetchHistoricData = useEventStore((state) => state.fetchHistoricData);

  // Memoized refetch function that bypasses staleness check
  const refetch = useCallback(() => {
    void fetchHistoricData(days);
  }, [days, fetchHistoricData]);

  // Effect to automatically fetch/refresh data when stale
  useEffect(() => {
    const shouldFetch = isDataStale(historicDataFetchedAt);

    // Only fetch if data is stale and we're not already loading
    if (shouldFetch && historicDataStatus !== 'loading') {
      void fetchHistoricData(days);
    }
  }, [days, fetchHistoricData, historicDataFetchedAt, historicDataStatus]);

  return {
    data: historicData,
    status: historicDataStatus,
    error: historicDataError,
    fetchedAt: historicDataFetchedAt,
    refetch,
  };
}

// -----------------------------------------------------------------------------
// Re-exports
// -----------------------------------------------------------------------------

// Re-export types that consumers might need
export type { HistoricDataSnapshot, HistoricDataStatus };
export type { HourlyAggregate };
