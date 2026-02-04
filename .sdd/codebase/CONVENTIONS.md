# Coding Conventions

**Purpose**: Document code style, naming conventions, error handling, and common patterns.
**Generated**: 2026-02-04
**Last Updated**: 2026-02-04 (Phase 5: Historic data UI and testing)

## Code Style

### Formatting Tools

| Tool | Configuration | Command |
|------|---------------|---------|
| Prettier (TypeScript/Client) | `.prettierrc` | `npm run format` |
| ESLint (TypeScript/Client) | `eslint.config.js` | `npm run lint` |
| rustfmt (Rust/Server/Monitor) | Default settings | `cargo fmt` |
| clippy (Rust/Server/Monitor) | Default lints | `cargo clippy` |
| Deno fmt (Supabase Edge Functions) | Default settings | `deno fmt` |

### Style Rules

#### TypeScript/Client (Phase 5 focus)

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 2 spaces | Standard JS/TS |
| Quotes | Single quotes | `'string'` |
| Semicolons | Always | `const x = 1;` |
| Line length | 100 chars (soft) | Prettier default |
| Comments | JSDoc for exports | `/** Description */` |
| Module docs | Every module/hook | Top-of-file comment block |

#### Rust/Server/Monitor (Phase 4 focus)

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 4 spaces | rustfmt default |
| Strings | Double quotes | `"string"` |
| Line length | 100 chars (soft) | rustfmt respects natural breaks |
| Comments | `//! ` for module docs, `///` for item docs | Doc comments on all public items |
| Module docs | Every module with overview and examples | `//! Event persistence with exponential backoff` |

## Naming Conventions

### TypeScript/Client (Phase 5)

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Components | PascalCase | `EventStream.tsx`, `HeatmapView.tsx` |
| Hooks | camelCase with `use` prefix | `useHistoricData.ts`, `useEventStore.ts` |
| Utilities | camelCase | `formatDate.ts`, `parseEvent.ts` |
| Types | PascalCase in `types/` | `types/events.ts` |
| Mocks | In `mocks/` directory | `mocks/handlers.ts`, `mocks/data.ts` |
| Tests | `__tests__/` directory + `.test.ts` suffix | `__tests__/events.test.ts` |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Variables | camelCase | `historicData`, `fetchStatus` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_EVENTS`, `STALE_THRESHOLD_MS` |
| Functions | camelCase, verb prefix | `createHourlyAggregate()`, `parseEvents()` |
| React components | PascalCase | `App`, `EventStream`, `SessionList` |
| Store hooks | `useStoreName` | `useEventStore` |
| Custom hooks | `use` + descriptor | `useHistoricData`, `useWebSocket` |
| Types/Interfaces | PascalCase | `VibeteaEvent`, `Session`, `EventFilters` |
| Event types | lowercase string literals | `'session'`, `'tool'`, `'summary'` |

### Rust/Server/Monitor (Phase 4)

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `persistence.rs`, `config.rs` |
| Module constants | SCREAMING_SNAKE_CASE | `MAX_BATCH_SIZE`, `INITIAL_RETRY_DELAY_MS` |
| Tests | Inline `#[cfg(test)] mod tests` | `sender_recovery_test.rs`, `privacy_test.rs` |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Structs | PascalCase | `EventBatcher`, `PersistenceConfig`, `RetryPolicy` |
| Enums | PascalCase | `PersistenceError`, `EventType`, `ToolStatus` |
| Error variants | PascalCase | `AuthFailed`, `MaxRetriesExceeded`, `ServerError` |
| Functions | snake_case | `new()`, `queue()`, `flush()`, `validated()` |
| Methods | snake_case | `.with_retry_policy()`, `.is_extension_allowed()` |
| Test functions | `test_` prefix | `test_oversized_event_does_not_block()` |

## Error Handling

### TypeScript/Client - Fetch Errors (Phase 5)

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Network errors | Try/catch with typed response | `useEventStore.ts` line 310-367 |
| API validation errors | Parse JSON error response | Error message extraction on line 324-338 |
| Missing config | Early return with error state | Lines 289-305 |
| Fallback messages | Use `statusText` if JSON fails | Line 337 |

Example from `useEventStore.ts` - Historic data fetch:

```typescript
try {
  const response = await fetch(`${supabaseUrl}/functions/v1/query?days=${days}`, {
    method: 'GET',
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    // Parse error response to extract message
    let errorMessage = `HTTP ${response.status}`;
    try {
      const errorBody = (await response.json()) as {
        error?: string;
        message?: string;
      };
      if (errorBody.message !== undefined) {
        errorMessage = errorBody.message;
      } else if (errorBody.error !== undefined) {
        errorMessage = errorBody.error;
      }
    } catch {
      // If JSON parsing fails, use status text
      errorMessage = response.statusText || `HTTP ${response.status}`;
    }

    set({
      historicDataStatus: 'error',
      historicDataError: errorMessage,
    });
    return;
  }

  const data = (await response.json()) as QueryResponse;
  // Success state update
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : 'Failed to fetch historic data';
  set({
    historicDataStatus: 'error',
    historicDataError: errorMessage,
  });
}
```

### Rust/Monitor - Persistence Module (Phase 4)

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Persistence errors | `#[derive(Error)]` enum with variants | `monitor/src/persistence.rs` |
| Max retries exceeded | Struct variant with field | `MaxRetriesExceeded { attempts: u8 }` |
| Server errors | Struct variant with status/message | `ServerError { status: u16, message: String }` |
| Auth failures | Simple variant | `AuthFailed` |
| HTTP errors | Automatic conversion | `#[from] reqwest::Error` |
| JSON errors | Automatic conversion | `#[from] serde_json::Error` |

### Logging Conventions

#### TypeScript/Client - Use `console` for browser environment

| Level | When to Use | Example |
|-------|-------------|---------|
| error | Unrecoverable failures, API errors | `console.error('Failed to fetch historic data', error)` |
| warn | Config issues, edge cases | `console.warn('Persistence not configured')` |
| info | Important state changes | `console.info('Historic data fetched successfully')` |
| debug | Store updates, hook effects | `console.debug('Stale data detected, refetching')` |

#### Rust/Monitor - Use `tracing` crate

| Level | When to Use | Example |
|-------|-------------|---------|
| error | Failures that affect operation | `error!("Batch submission failed after {attempts} retries", attempts)` |
| warn | Recoverable issues or unsafe modes | `warn!("Persistence buffer overflow, dropping oldest event")` |
| info | State changes and milestones | `info!("Batch of {count} events submitted successfully", count)` |
| debug | Diagnostic information | `debug!("Retry policy configured: initial={ms}ms, max={max}ms", ms, max)` |

## Common Patterns (Phase 5 Update)

### MSW Handler Pattern (Phase 5 - New)

Mock Service Worker handlers for testing data fetching:

```typescript
// mocks/handlers.ts
import { http, HttpResponse } from 'msw';

/**
 * Handler for GET /functions/v1/query endpoint.
 * Validates Authorization header and days query parameter.
 */
const queryHandler = http.get('*/functions/v1/query', ({ request }) => {
  // Step 1: Extract and validate bearer token
  const authHeader = request.headers.get('Authorization');
  if (authHeader === null) {
    return HttpResponse.json(errorResponses.missingAuth, { status: 401 });
  }

  const parts = authHeader.split(' ');
  if (parts.length !== 2 || parts[0] !== 'Bearer') {
    return HttpResponse.json(errorResponses.invalidToken, { status: 401 });
  }

  // Step 2: Validate days parameter
  const url = new URL(request.url);
  const daysParam = url.searchParams.get('days');
  const days = daysParam === null ? 7 : parseInt(daysParam, 10);

  if (days !== 7 && days !== 30) {
    return HttpResponse.json(errorResponses.invalidDays, { status: 400 });
  }

  // Step 3: Return mock data
  return HttpResponse.json(createQueryResponse(days), { status: 200 });
});

export const queryHandlers = [queryHandler] as const;
```

Key MSW patterns:
1. **URL pattern matching**: `*/functions/v1/query` matches any Supabase URL
2. **Request inspection**: Extract headers, query params from Request object
3. **Response simulation**: Return `HttpResponse.json()` with status code
4. **Handler arrays**: Export as readonly array for spread into server setup

### Mock Data Factory Pattern (Phase 5 - New)

Generate realistic test data:

```typescript
// mocks/data.ts
export interface QueryResponse {
  readonly aggregates: HourlyAggregate[];
  readonly meta: {
    readonly totalCount: number;
    readonly daysRequested: 7 | 30;
    readonly fetchedAt: string;
  };
}

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

export function generateMockAggregates(
  days: 7 | 30,
  source: string = MOCK_SOURCE
): HourlyAggregate[] {
  const aggregates: HourlyAggregate[] = [];

  for (let dayOffset = 0; dayOffset < days; dayOffset++) {
    const date = new Date(now);
    date.setUTCDate(date.getUTCDate() - dayOffset);
    const dateStr = date.toISOString().split('T')[0];

    // Higher event counts during work hours (9-17)
    for (let hour = 0; hour < 24; hour++) {
      if (Math.random() < 0.3) continue; // Skip some hours for realism

      const isWorkHour = hour >= 9 && hour <= 17;
      const baseCount = isWorkHour ? 80 : 20;
      const variance = isWorkHour ? 120 : 30;
      const eventCount = baseCount + Math.floor(Math.random() * variance);

      aggregates.push({ source, date: dateStr, hour, eventCount });
    }
  }

  return aggregates.sort((a, b) => {
    const dateCompare = b.date.localeCompare(a.date);
    return dateCompare !== 0 ? dateCompare : b.hour - a.hour;
  });
}
```

Key factory patterns:
1. **Override pattern**: Accept partial object for customization
2. **Realistic data**: Variable counts, simulated work hours, gaps
3. **Deterministic defaults**: Use today's date but allow overrides
4. **Type safety**: Return typed objects matching API responses

### Zustand Store Pattern with Async Actions (Phase 5)

State management with fetchHistoricData action:

```typescript
// hooks/useEventStore.ts
export interface EventStore {
  // State
  readonly historicData: readonly HourlyAggregate[];
  readonly historicDataStatus: HistoricDataStatus;
  readonly historicDataFetchedAt: number | null;
  readonly historicDataError: string | null;

  // Actions
  readonly fetchHistoricData: (days: 7 | 30) => Promise<void>;
  readonly clearHistoricData: () => void;
}

export const useEventStore = create<EventStore>()((set) => ({
  // Initial state
  historicData: [],
  historicDataStatus: 'idle',
  historicDataFetchedAt: null,
  historicDataError: null,

  // Async action with status management
  fetchHistoricData: async (days: 7 | 30): Promise<void> => {
    // Validate environment configuration
    const supabaseUrl = import.meta.env.VITE_SUPABASE_URL as string | undefined;
    if (supabaseUrl === undefined || supabaseUrl === '') {
      set({
        historicDataStatus: 'error',
        historicDataError: 'Persistence not configured',
      });
      return;
    }

    // Set loading state before fetch
    set({ historicDataStatus: 'loading', historicDataError: null });

    try {
      const response = await fetch(
        `${supabaseUrl}/functions/v1/query?days=${days}`,
        {
          method: 'GET',
          headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
          },
        }
      );

      if (!response.ok) {
        // Extract error message from response
        let errorMessage = `HTTP ${response.status}`;
        try {
          const errorBody = (await response.json()) as {
            error?: string;
            message?: string;
          };
          if (errorBody.message !== undefined) {
            errorMessage = errorBody.message;
          }
        } catch {
          errorMessage = response.statusText || `HTTP ${response.status}`;
        }

        set({
          historicDataStatus: 'error',
          historicDataError: errorMessage,
        });
        return;
      }

      // Success: update state with fetched data
      const data = (await response.json()) as QueryResponse;
      set({
        historicData: data.aggregates,
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    } catch (error) {
      set({
        historicDataStatus: 'error',
        historicDataError: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  },

  clearHistoricData: () => {
    set({
      historicData: [],
      historicDataStatus: 'idle',
      historicDataFetchedAt: null,
      historicDataError: null,
    });
  },
}));
```

Key store patterns:
1. **Status state machine**: `idle` → `loading` → `success`/`error`
2. **Timestamp tracking**: Use `Date.now()` for cache staleness detection
3. **Error first validation**: Check configuration before fetch
4. **Graceful degradation**: Parse error response, fallback to status text
5. **Selective state updates**: Only update changed fields

### Custom Hook Pattern with Store Selectors (Phase 5)

Composite hooks using store with memoization:

```typescript
// hooks/useHistoricData.ts
export interface UseHistoricDataResult {
  readonly data: readonly HourlyAggregate[];
  readonly status: HistoricDataStatus;
  readonly error: string | null;
  readonly fetchedAt: number | null;
  readonly refetch: () => void;
}

const STALE_THRESHOLD_MS = 5 * 60 * 1000; // 5 minutes

function isDataStale(fetchedAt: number | null): boolean {
  if (fetchedAt === null) {
    return true;
  }
  return Date.now() - fetchedAt > STALE_THRESHOLD_MS;
}

export function useHistoricData(days: 7 | 30): UseHistoricDataResult {
  // Select individual slices to optimize re-renders
  const historicData = useEventStore((state) => state.historicData);
  const historicDataStatus = useEventStore((state) => state.historicDataStatus);
  const historicDataFetchedAt = useEventStore((state) => state.historicDataFetchedAt);
  const historicDataError = useEventStore((state) => state.historicDataError);
  const fetchHistoricData = useEventStore((state) => state.fetchHistoricData);

  // Memoized refetch function
  const refetch = useCallback(() => {
    void fetchHistoricData(days);
  }, [days, fetchHistoricData]);

  // Auto-fetch when stale
  useEffect(() => {
    const shouldFetch = isDataStale(historicDataFetchedAt);

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
```

Key hook patterns:
1. **Selector optimization**: Use individual selectors to prevent unnecessary re-renders
2. **Stale-while-revalidate**: Auto-refetch when cache is older than threshold
3. **Memoized callbacks**: Use `useCallback` to prevent effect re-triggers
4. **Status checking**: Don't refetch if already loading
5. **Manual refetch**: Always provide explicit refetch function

### Store Direct Testing Pattern (Phase 5)

Test Zustand store state directly without rendering components:

```typescript
// __tests__/App.test.tsx
beforeEach(() => {
  localStorage.clear();
  useEventStore.setState({
    status: 'disconnected',
    events: [],
    sessions: new Map(),
    filters: { sessionId: null, timeRange: null },
  });
});

it('filter state can be updated via store actions', () => {
  // Test store actions directly without rendering component
  const { setSessionFilter, setTimeRangeFilter, clearFilters } =
    useEventStore.getState();

  // Set session filter
  setSessionFilter('test-session-123');
  expect(useEventStore.getState().filters.sessionId).toBe('test-session-123');

  // Set time range filter
  const startTime = new Date('2024-01-01T10:00:00Z');
  const endTime = new Date('2024-01-01T11:00:00Z');
  setTimeRangeFilter({ start: startTime, end: endTime });
  expect(useEventStore.getState().filters.timeRange).toEqual({
    start: startTime,
    end: endTime,
  });

  // Clear filters
  clearFilters();
  expect(useEventStore.getState().filters.sessionId).toBeNull();
  expect(useEventStore.getState().filters.timeRange).toBeNull();
});
```

Key direct testing patterns:
1. **Reset before each**: Use `setState` to reset to clean state
2. **Get actions**: Use `getState()` to access action functions
3. **Assert state changes**: Call actions then verify with `getState()`
4. **Avoid component rendering**: Test logic without React overhead
5. **Deterministic tests**: No async timing issues

## Import Ordering

Standard import order:

1. External packages (react, zustand, msw, etc.)
2. Internal hooks and utilities
3. Type imports
4. Test utilities (in test files only)

Example:

```typescript
import { useCallback, useEffect } from 'react';
import { useEventStore } from './useEventStore';
import type { HourlyAggregate } from '../types/events';

// Test imports (only in .test.ts files)
import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
```

## Comments & Documentation

| Type | When to Use | Format |
|------|-------------|--------|
| Module doc | Top of file, before imports | `/** ... */` block |
| Function doc | Public functions and hooks | JSDoc with @param, @returns |
| Inline | Complex logic or non-obvious code | `// Explanation` |
| TODO | Planned work | `// TODO: description` |
| FIXME | Known issues | `// FIXME: description` |

Example module documentation (Phase 5):

```typescript
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
 *   return <Heatmap data={data} />;
 * }
 * ```
 */
```

## Git Conventions

### Commit Messages (Phase 5 Update)

Format: `type(scope): description`

Phase 5 examples:
- `feat(client): implement historic data hook with stale-while-revalidate`
- `feat(client): add MSW handlers for query endpoint`
- `feat(client): implement Zustand store async data fetching`
- `test(client): add hook tests using renderHook and MSW`
- `test(client): add App component tests with store mocking`

### Branch Naming

Format: `{type}/{ticket}-{description}`

Example: `feat/005-historic-data-ui`

---

*This document defines HOW to write code for Phase 5 (Historic data UI). Update when conventions change.*
