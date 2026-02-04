# Testing Strategy

**Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
**Generated**: 2026-02-04
**Last Updated**: 2026-02-04 (Phase 6: Heatmap component testing with fake timers and store mocking)

## Test Framework

### TypeScript/Client (Updated for Phase 6)

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Vitest | `vite.config.ts` test config | In use |
| Component | @testing-library/react | jsdom/happy-dom environment | In use (Phase 6) |
| E2E | Not selected | TBD | Not started |
| Mocks | Mock Service Worker (MSW) | `client/src/mocks/` | In use (Phase 5+) |

### Rust/Server and Monitor (Phase 4)

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Rust built-in | `#[cfg(test)] mod tests` inline | In use |
| Integration | Rust built-in + wiremock | `tests/` directory with MockServer | In use (Phase 4) |
| E2E | Not selected | TBD | Not started |

### Running Tests

#### TypeScript/Client (Phase 6)

| Command | Purpose |
|---------|---------|
| `npm test` | Run all tests in watch mode |
| `npm run test` | Run all tests once |
| `npm run test:watch` | Run tests in watch mode |
| `npm run test:coverage` | Run with coverage report |

#### Rust/Server and Monitor (Phase 4)

| Command | Purpose |
|---------|---------|
| `cargo test` | Run all tests in the workspace |
| `cargo test --lib` | Run library unit tests only |
| `cargo test --test '*'` | Run integration tests only |
| `cargo test -- --nocapture` | Run tests with println output |
| `cargo test -- --test-threads=1` | Run tests sequentially (prevents env var interference) |
| `cargo test -p vibetea-monitor persistence` | Run persistence module tests |
| `cargo test --test sender_recovery_test` | Run sender recovery integration tests |
| `cargo test -p vibetea-server unsafe_mode` | Run unsafe mode integration tests |

## Test Organization

### TypeScript/Client Directory Structure (Phase 6)

```
client/
├── src/
│   ├── __tests__/                           # Test directory
│   │   ├── App.test.tsx                     # App component tests
│   │   ├── components/
│   │   │   └── Heatmap.test.tsx             # Heatmap component tests (36 tests, Phase 6)
│   │   ├── hooks/
│   │   │   └── useHistoricData.test.tsx     # Hook tests with MSW
│   │   ├── events.test.ts                   # Event type tests
│   │   └── formatting.test.ts               # Utility tests
│   ├── components/
│   │   ├── Heatmap.tsx                      # Heatmap component (Phase 6)
│   │   ├── EventStream.tsx
│   │   └── ...
│   ├── hooks/                               # React hooks
│   │   ├── useEventStore.ts
│   │   ├── useHistoricData.ts
│   │   ├── useWebSocket.ts
│   │   └── useSessionTimeouts.ts
│   ├── mocks/                               # MSW mock setup
│   │   ├── handlers.ts
│   │   ├── server.ts
│   │   ├── data.ts
│   │   └── index.ts
│   ├── utils/
│   │   ├── persistence.ts                   # Persistence utilities (Phase 6)
│   │   ├── formatDate.ts
│   │   └── ...
│   ├── types/
│   │   └── events.ts
│   └── App.tsx
└── vite.config.ts                           # Vitest configuration
```

### Rust/Monitor Directory Structure (Phase 4)

```
monitor/
├── src/
│   ├── config.rs
│   ├── error.rs
│   ├── types.rs
│   ├── watcher.rs
│   ├── parser.rs
│   ├── privacy.rs
│   ├── crypto.rs
│   ├── sender.rs
│   ├── persistence.rs
│   ├── lib.rs
│   └── main.rs
└── tests/
    ├── privacy_test.rs
    └── sender_recovery_test.rs
```

## Test Patterns

### Component Testing with Store Setup (Phase 6 - New)

Test components that use Zustand store with direct state management:

```typescript
/**
 * Tests for Heatmap component.
 *
 * Tests the activity heatmap including persistence checks, data merging,
 * loading/error states, view toggles, and cell interactions.
 *
 * @vitest-environment happy-dom
 */

import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useEventStore } from '../../hooks/useEventStore';
import { Heatmap } from '../../components/Heatmap';

// Store original env values
const originalSupabaseUrl = import.meta.env.VITE_SUPABASE_URL;
const originalSupabaseToken = import.meta.env.VITE_SUPABASE_TOKEN;

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

beforeEach(() => {
  // Reset timers and mocks
  vi.clearAllMocks();

  // Enable persistence by default
  import.meta.env.VITE_SUPABASE_URL = 'https://test.supabase.co';
  import.meta.env.VITE_SUPABASE_TOKEN = 'test-token';

  // Reset store state
  resetStore();
});

afterEach(() => {
  // Restore env values
  import.meta.env.VITE_SUPABASE_URL = originalSupabaseUrl;
  import.meta.env.VITE_SUPABASE_TOKEN = originalSupabaseToken;

  // Reset store state
  resetStore();

  // Cleanup timers if used
  vi.useRealTimers();
});
```

Key component testing patterns (Phase 6):
1. **Environment isolation**: Save and restore env vars in setup/teardown
2. **Store reset**: Use `setState` to reset to clean state before each test
3. **Happy-dom environment**: Use `@vitest-environment happy-dom` for lighter DOM
4. **Fake timers**: Use `vi.useFakeTimers()` for time-dependent logic
5. **Cleanup timers**: Always restore with `vi.useRealTimers()` in afterEach

### Persistence Check Tests (Phase 6 - New)

Test feature detection and early returns:

```typescript
describe('Heatmap - Persistence Check', () => {
  it('should return null when isPersistenceEnabled() returns false', () => {
    // Disable persistence
    import.meta.env.VITE_SUPABASE_URL = '';
    import.meta.env.VITE_SUPABASE_TOKEN = '';

    const { container } = render(<Heatmap />);

    // Component should render nothing
    expect(container.firstChild).toBeNull();
  });

  it('should render when persistence is enabled', () => {
    // Persistence enabled in beforeEach
    render(<Heatmap />);

    // Component should render the activity heading
    expect(
      screen.getByRole('region', { name: 'Activity heatmap' })
    ).toBeInTheDocument();
  });
});
```

Key persistence test patterns (Phase 6):
1. **Feature gates**: Test that components return null when disabled
2. **Environment variables**: Manipulate VITE_ constants for testing
3. **Container assertions**: Check `container.firstChild` for null rendering
4. **Region/heading checks**: Verify component structure when enabled

### Data Merging Tests (Phase 6 - New)

Test complex data combination logic:

```typescript
describe('Heatmap - Data Merging Logic', () => {
  it('should use real-time events for current hour over historic data', async () => {
    const now = new Date();
    const currentHour = now.getHours();
    const currentDateStr = now.toISOString().split('T')[0];

    // Set up historic data for current hour with 5 events
    const historicData: HourlyAggregate[] = [
      createHourlyAggregate({
        date: currentDateStr,
        hour: currentHour,
        eventCount: 5, // Historic says 5 events
      }),
    ];

    // Add 10 real-time events for current hour
    const realtimeEvents: VibeteaEvent[] = [];
    for (let i = 0; i < 10; i++) {
      realtimeEvents.push(createMockEvent());
    }

    // Set up store state with fresh historic data and real-time events
    await act(async () => {
      useEventStore.setState({
        events: realtimeEvents,
        historicData,
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    });

    render(<Heatmap />);

    // The grid should be rendered (we have events)
    await waitFor(() => {
      expect(screen.getByRole('grid')).toBeInTheDocument();
    });

    // Find cells and verify the current hour shows real-time count (10), not historic (5)
    const cells = screen.getAllByRole('gridcell');
    const currentHourCell = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('10 events')
    );
    expect(currentHourCell).toBeInTheDocument();
  });

  it('should use historic data for past hours', async () => {
    const now = new Date();
    const yesterday = new Date(now);
    yesterday.setDate(yesterday.getDate() - 1);
    const yesterdayDateStr = yesterday.toISOString().split('T')[0];

    // Set up historic data for yesterday at noon with 50 events
    const historicData: HourlyAggregate[] = [
      createHourlyAggregate({
        date: yesterdayDateStr,
        hour: 12,
        eventCount: 50,
      }),
    ];

    await act(async () => {
      useEventStore.setState({
        events: [],
        historicData,
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    });

    render(<Heatmap />);

    await waitFor(() => {
      expect(screen.getByRole('grid')).toBeInTheDocument();
    });

    // Grid should render with cells
    const cells = screen.getAllByRole('gridcell');
    expect(cells.length).toBeGreaterThan(0);
  });
});
```

Key data merging test patterns (Phase 6):
1. **Timestamp manipulation**: Use `new Date()` and `setDate()` for date control
2. **Store act wrapper**: Wrap all state changes in `act()`
3. **Multiple assertions**: Test both presence and counts
4. **Aria-label matching**: Use regex to find cells by event count
5. **Grid structure**: Verify grid renders with expected cell count

### Loading State with Fake Timers (Phase 6 - New)

Test async operations and timeouts:

```typescript
describe('Heatmap - Loading State', () => {
  it('should show "Fetching historic data..." with spinner when loading', () => {
    // Set up loading state
    useEventStore.setState({
      events: [],
      historicData: [],
      historicDataStatus: 'loading',
      historicDataFetchedAt: null,
      historicDataError: null,
    });

    render(<Heatmap />);

    // Should show loading indicator
    expect(screen.getByText('Fetching historic data...')).toBeInTheDocument();

    // Should have a spinner (svg with animate-spin class)
    const spinner = document.querySelector('svg.animate-spin');
    expect(spinner).toBeInTheDocument();
  });

  it('should transition loading state to error after 5 second timeout', async () => {
    vi.useFakeTimers();

    // Mock that maintains loading state (never resolves to success)
    const loadingMock = vi.fn().mockImplementation(() => {
      // Keep the loading state - don't change it
      return Promise.resolve();
    });

    useEventStore.setState({
      fetchHistoricData: loadingMock,
      historicData: [],
      historicDataStatus: 'loading',
      historicDataFetchedAt: null,
      historicDataError: null,
    });

    render(<Heatmap />);

    // Initially should show loading
    expect(screen.getByText('Fetching historic data...')).toBeInTheDocument();

    // Advance time by 5 seconds (the LOADING_TIMEOUT_MS)
    act(() => {
      vi.advanceTimersByTime(5000);
    });

    // After timeout, should show error message
    expect(
      screen.getByText(
        'Unable to load historic data. Showing real-time events only.'
      )
    ).toBeInTheDocument();
  });

  it('should not show loading indicator when historic data exists', () => {
    // Still loading but have cached data
    const historicData: HourlyAggregate[] = [
      createHourlyAggregate({ eventCount: 10 }),
    ];

    useEventStore.setState({
      events: [],
      historicData,
      historicDataStatus: 'loading',
      historicDataFetchedAt: Date.now() - 6 * 60 * 1000, // 6 minutes ago (stale)
      historicDataError: null,
    });

    render(<Heatmap />);

    // Should NOT show loading indicator when we have cached data
    expect(
      screen.queryByText('Fetching historic data...')
    ).not.toBeInTheDocument();
  });
});
```

Key fake timer test patterns (Phase 6):
1. **useFakeTimers**: Enable with `vi.useFakeTimers()` at start of test
2. **advanceTimersByTime**: Skip forward by milliseconds in tests
3. **act wrapper**: Always wrap timer advances in `act()`
4. **Cleanup**: Call `vi.useRealTimers()` in afterEach
5. **Cache-aware logic**: Test that cached data prevents loading state

### Error State Tests (Phase 6)

Test error handling and recovery:

```typescript
describe('Heatmap - Error State', () => {
  it('should show error message when status is error', async () => {
    const errorMock = vi.fn().mockImplementation(() => {
      return Promise.resolve();
    });

    await act(async () => {
      useEventStore.setState({
        fetchHistoricData: errorMock,
        events: [],
        historicData: [],
        historicDataStatus: 'error',
        historicDataFetchedAt: Date.now(),
        historicDataError: 'Database query failed',
      });
    });

    render(<Heatmap />);

    expect(
      screen.getByText(
        'Unable to load historic data. Showing real-time events only.'
      )
    ).toBeInTheDocument();
  });

  it('should show Retry button that triggers refetch', async () => {
    let refetchCalled = false;

    const mockFetch = vi.fn().mockImplementation(() => {
      refetchCalled = true;
      useEventStore.setState({
        historicData: [],
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
      return Promise.resolve();
    });

    await act(async () => {
      useEventStore.setState({
        fetchHistoricData: mockFetch,
        events: [],
        historicData: [],
        historicDataStatus: 'error',
        historicDataFetchedAt: Date.now(),
        historicDataError: 'Network error',
      });
    });

    render(<Heatmap />);

    const retryButton = screen.getByRole('button', { name: 'Retry' });
    expect(retryButton).toBeInTheDocument();

    fireEvent.click(retryButton);

    await waitFor(() => {
      expect(refetchCalled).toBe(true);
    });
  });

  it('should still display real-time data during error state', async () => {
    const realtimeEvents = [
      createMockEvent(),
      createMockEvent(),
      createMockEvent(),
    ];

    const errorMock = vi.fn().mockImplementation(() => Promise.resolve());

    await act(async () => {
      useEventStore.setState({
        fetchHistoricData: errorMock,
        events: realtimeEvents,
        historicData: [],
        historicDataStatus: 'error',
        historicDataFetchedAt: Date.now(),
        historicDataError: 'Server error',
      });
    });

    render(<Heatmap />);

    // Error message should be shown
    expect(
      screen.getByText(
        'Unable to load historic data. Showing real-time events only.'
      )
    ).toBeInTheDocument();

    // But the grid should still be rendered with real-time data
    expect(screen.getByRole('grid')).toBeInTheDocument();

    // Should have cells showing the 3 events
    const cells = screen.getAllByRole('gridcell');
    const cellWithEvents = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('3 events')
    );
    expect(cellWithEvents).toBeInTheDocument();
  });
});
```

Key error state patterns (Phase 6):
1. **Error message verification**: Check specific error text appears
2. **Retry button testing**: Find and click retry, verify action called
3. **Graceful degradation**: Show real-time data even on error
4. **Mock function tracking**: Use boolean flags to track callback invocation

### View Toggle Tests (Phase 6)

Test view switching and data fetching:

```typescript
describe('Heatmap - View Toggle', () => {
  it('should render 7-day and 30-day toggle buttons', () => {
    useEventStore.setState({
      historicData: [createHourlyAggregate({ eventCount: 5 })],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
    });

    render(<Heatmap />);

    expect(screen.getByRole('button', { name: '7 Days' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '30 Days' })).toBeInTheDocument();
  });

  it('should have 7-day view selected by default', () => {
    useEventStore.setState({
      historicData: [createHourlyAggregate({ eventCount: 5 })],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
    });

    render(<Heatmap />);

    const sevenDayButton = screen.getByRole('button', { name: '7 Days' });
    expect(sevenDayButton).toHaveAttribute('aria-pressed', 'true');

    const thirtyDayButton = screen.getByRole('button', { name: '30 Days' });
    expect(thirtyDayButton).toHaveAttribute('aria-pressed', 'false');
  });

  it('should switch to 30-day view when clicked', () => {
    useEventStore.setState({
      historicData: [createHourlyAggregate({ eventCount: 5 })],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
    });

    render(<Heatmap />);

    const thirtyDayButton = screen.getByRole('button', { name: '30 Days' });
    fireEvent.click(thirtyDayButton);

    expect(thirtyDayButton).toHaveAttribute('aria-pressed', 'true');

    const sevenDayButton = screen.getByRole('button', { name: '7 Days' });
    expect(sevenDayButton).toHaveAttribute('aria-pressed', 'false');
  });

  it('should render more cells in 30-day view', async () => {
    const events = [createMockEvent()];

    const preservingMock = vi.fn().mockImplementation(() => {
      const state = useEventStore.getState();
      useEventStore.setState({
        historicData: state.historicData,
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
      return Promise.resolve();
    });

    await act(async () => {
      useEventStore.setState({
        fetchHistoricData: preservingMock,
        events,
        historicData: [],
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    });

    render(<Heatmap />);

    // Wait for grid to render
    await waitFor(() => {
      expect(screen.getByRole('grid')).toBeInTheDocument();
    });

    // In 7-day view, should have 7 * 24 = 168 cells
    let cells = screen.getAllByRole('gridcell');
    expect(cells.length).toBe(7 * 24);

    // Switch to 30-day view
    fireEvent.click(screen.getByRole('button', { name: '30 Days' }));

    // In 30-day view, should have 30 * 24 = 720 cells
    cells = screen.getAllByRole('gridcell');
    expect(cells.length).toBe(30 * 24);
  });
});
```

Key view toggle patterns (Phase 6):
1. **aria-pressed attribute**: Check toggle button state
2. **Cell count verification**: Verify grid size changes
3. **Button interaction**: Click buttons and verify state changes
4. **Rerender handling**: Grid updates when view changes

### Cell Interaction Tests (Phase 6)

Test user interactions and callbacks:

```typescript
describe('Heatmap - Cell Interaction', () => {
  it('should show tooltip with correct count on hover', async () => {
    const events = [createMockEvent(), createMockEvent(), createMockEvent()];

    useEventStore.setState({
      events,
      historicData: [],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });

    render(<Heatmap />);

    // Find the cell with 3 events
    const cells = screen.getAllByRole('gridcell');
    const cellWith3Events = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('3 events')
    );

    if (cellWith3Events) {
      fireEvent.mouseEnter(cellWith3Events);
    }

    // Tooltip should appear
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
      expect(screen.getByText('3 events')).toBeInTheDocument();
    });
  });

  it('should call onCellClick with correct time range when cell is clicked', async () => {
    const onCellClick = vi.fn();

    const events = [createMockEvent()];

    useEventStore.setState({
      events,
      historicData: [],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });

    render(<Heatmap onCellClick={onCellClick} />);

    // Find a cell with event and click it
    const cells = screen.getAllByRole('gridcell');
    const cellWithEvent = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('1 event')
    );

    if (cellWithEvent) {
      fireEvent.click(cellWithEvent);
    }

    // onCellClick should be called with start and end times
    expect(onCellClick).toHaveBeenCalledTimes(1);

    const [startTime, endTime] = onCellClick.mock.calls[0] as [Date, Date];

    // Verify times are Date objects
    expect(startTime).toBeInstanceOf(Date);
    expect(endTime).toBeInstanceOf(Date);

    // End time should be exactly 1 hour after start time
    const timeDiff = endTime.getTime() - startTime.getTime();
    expect(timeDiff).toBe(60 * 60 * 1000); // 1 hour in ms
  });

  it('should support keyboard navigation with Enter key', () => {
    const onCellClick = vi.fn();

    const events = [createMockEvent()];

    useEventStore.setState({
      events,
      historicData: [],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });

    render(<Heatmap onCellClick={onCellClick} />);

    const cells = screen.getAllByRole('gridcell');
    const cellWithEvent = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('1 event')
    );

    if (cellWithEvent) {
      cellWithEvent.focus();
      fireEvent.keyDown(cellWithEvent, { key: 'Enter' });
      expect(onCellClick).toHaveBeenCalledTimes(1);
    }
  });

  it('should support keyboard navigation with Space key', () => {
    const onCellClick = vi.fn();

    const events = [createMockEvent()];

    useEventStore.setState({
      events,
      historicData: [],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });

    render(<Heatmap onCellClick={onCellClick} />);

    const cells = screen.getAllByRole('gridcell');
    const cellWithEvent = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('1 event')
    );

    if (cellWithEvent) {
      cellWithEvent.focus();
      fireEvent.keyDown(cellWithEvent, { key: ' ' });
      expect(onCellClick).toHaveBeenCalledTimes(1);
    }
  });
});
```

Key cell interaction patterns (Phase 6):
1. **Hover interactions**: Use `fireEvent.mouseEnter` for tooltips
2. **Click handling**: Fire click events on cells
3. **Callback verification**: Check mock function calls and arguments
4. **Time range validation**: Verify 1-hour duration calculations
5. **Keyboard support**: Test Enter and Space key interactions

### Accessibility Tests (Phase 6)

Test ARIA labels and keyboard navigation:

```typescript
describe('Heatmap - Accessibility', () => {
  it('should have proper ARIA labels on cells', async () => {
    const events = [createMockEvent()];

    await act(async () => {
      useEventStore.setState({
        events,
        historicData: [],
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    });

    render(<Heatmap />);

    await waitFor(() => {
      expect(screen.getByRole('grid')).toBeInTheDocument();
    });

    const cells = screen.getAllByRole('gridcell');

    // All cells should have aria-label
    cells.forEach((cell) => {
      expect(cell).toHaveAttribute('aria-label');
    });

    // Check format of aria-label (should include event count and time)
    const cellWithEvent = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('event')
    );
    expect(cellWithEvent?.getAttribute('aria-label')).toMatch(
      /\d+ events? on .+ at \d{2}:00/
    );
  });

  it('should have proper region role and label', () => {
    render(<Heatmap />);

    const region = screen.getByRole('region', { name: 'Activity heatmap' });
    expect(region).toBeInTheDocument();
  });

  it('should have accessible view toggle buttons', async () => {
    await act(async () => {
      useEventStore.setState({
        historicData: [createHourlyAggregate({ eventCount: 5 })],
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
      });
    });

    render(<Heatmap />);

    const group = screen.getByRole('group', { name: 'View range selector' });
    expect(group).toBeInTheDocument();

    const buttons = screen.getAllByRole('button');
    const toggleButtons = buttons.filter(
      (btn) =>
        btn.textContent?.includes('Days') && btn.hasAttribute('aria-pressed')
    );
    expect(toggleButtons.length).toBe(2);
  });

  it('should have focusable cells with tabIndex', async () => {
    const events = [createMockEvent()];

    await act(async () => {
      useEventStore.setState({
        events,
        historicData: [],
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    });

    render(<Heatmap />);

    await waitFor(() => {
      expect(screen.getByRole('grid')).toBeInTheDocument();
    });

    const cells = screen.getAllByRole('gridcell');

    // All cells should be focusable
    cells.forEach((cell) => {
      expect(cell).toHaveAttribute('tabIndex', '0');
    });
  });
});
```

Key accessibility test patterns (Phase 6):
1. **ARIA label validation**: Check aria-label format with regex
2. **Region/role verification**: Verify semantic structure
3. **Group roles**: Check button groups are properly labeled
4. **TabIndex**: Ensure interactive elements are focusable
5. **Role matching**: Use semantic roles in queries

### Empty State and Color Scale Tests (Phase 6)

Test edge cases and visual states:

```typescript
describe('Heatmap - Empty State', () => {
  it('should show empty state when no events exist', () => {
    useEventStore.setState({
      events: [],
      historicData: [],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });

    render(<Heatmap />);

    expect(screen.getByText('No activity data')).toBeInTheDocument();
    expect(
      screen.getByText('Events will appear here as they occur')
    ).toBeInTheDocument();
  });

  it('should not show grid when there are no events', () => {
    useEventStore.setState({
      events: [],
      historicData: [],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });

    render(<Heatmap />);

    expect(screen.queryByRole('grid')).not.toBeInTheDocument();
  });
});

describe('Heatmap - Color Scale', () => {
  it('should render legend with color scale', async () => {
    const events = [createMockEvent()];

    await act(async () => {
      useEventStore.setState({
        events,
        historicData: [],
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    });

    render(<Heatmap />);

    await waitFor(() => {
      expect(screen.getByRole('grid')).toBeInTheDocument();
    });

    expect(screen.getByText('Less')).toBeInTheDocument();
    expect(screen.getByText('More')).toBeInTheDocument();
  });

  it('should apply different colors based on event count', async () => {
    const now = new Date();
    const events: VibeteaEvent[] = [];

    // Add 55 events to get bright green color (51+)
    for (let i = 0; i < 55; i++) {
      events.push(createMockEvent({ timestamp: now.toISOString() }));
    }

    await act(async () => {
      useEventStore.setState({
        events,
        historicData: [],
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    });

    render(<Heatmap />);

    await waitFor(() => {
      expect(screen.getByRole('grid')).toBeInTheDocument();
    });

    // Find the cell with 55 events
    const cells = screen.getAllByRole('gridcell');
    const cellWith55Events = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('55 events')
    );

    expect(cellWith55Events).toBeInTheDocument();

    // Should have the bright green color (#5dad6f) for 51+ events
    expect(cellWith55Events).toHaveStyle({ backgroundColor: '#5dad6f' });
  });
});
```

Key pattern testing patterns (Phase 6):
1. **Empty state verification**: Check text and grid absence
2. **Legend rendering**: Verify color scale legend appears
3. **Color computation**: Verify backgroundColor styles match counts
4. **Multiple event creation**: Generate test data programmatically

## Mocking Strategy (Phase 6 Update)

### TypeScript/Client - MSW Mocks

MSW handlers intercept HTTP requests at the browser level:

```typescript
// mocks/handlers.ts
import { http, HttpResponse } from 'msw';

const queryHandler = http.get('*/functions/v1/query', ({ request }) => {
  // Extract and validate authorization
  const authHeader = request.headers.get('Authorization');
  if (authHeader === null) {
    return HttpResponse.json(
      { error: 'missing_auth', message: 'Authorization header required' },
      { status: 401 }
    );
  }

  // Extract and validate days parameter
  const url = new URL(request.url);
  const days = url.searchParams.get('days') ?? '7';

  if (!['7', '30'].includes(days)) {
    return HttpResponse.json(
      { error: 'invalid_days', message: 'days parameter must be 7 or 30' },
      { status: 400 }
    );
  }

  // Return mock data
  return HttpResponse.json(createQueryResponse(Number(days) as 7 | 30), {
    status: 200,
  });
});

export const queryHandlers = [queryHandler] as const;
```

### Zustand Store Mocking (Phase 6)

Mock store state by resetting and injecting test data:

```typescript
beforeEach(() => {
  // Reset to clean state
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

  // Can also pre-populate with test data
  useEventStore.setState({
    status: 'connected',
    events: [mockEvent1, mockEvent2],
    sessions: new Map([['session-1', mockSession]]),
  });
});
```

### Vitest Mock Functions (Phase 6)

Use `vi.fn()` to track mock behavior:

```typescript
// Track fetch calls
let fetchCalls: number[] = [];

beforeEach(() => {
  fetchCalls = [];

  const mockFetch = vi.fn().mockImplementation((days: 7 | 30) => {
    fetchCalls.push(days);
    useEventStore.setState({
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
    });
    return Promise.resolve();
  });
  useEventStore.setState({ fetchHistoricData: mockFetch });
});

// In test
fireEvent.click(screen.getByRole('button', { name: '30 Days' }));
expect(fetchCalls).toContain(30);
```

Key mocking patterns (Phase 6):
1. **vi.fn()**: Create traceable mock functions
2. **mockImplementation**: Control mock behavior per test
3. **Track calls**: Use arrays to count invocations
4. **State injection**: Pre-populate store with test data

### Mock Locations

| Mock Type | Location | Usage |
|-----------|----------|-------|
| HTTP handlers | `client/src/mocks/handlers.ts` | MSW handlers for endpoints |
| Mock data factories | `client/src/mocks/data.ts` | Generate test data |
| Server setup | `client/src/mocks/server.ts` | MSW server configuration |
| Store reset | Test files inline | Reset store before tests |
| Fixtures | Test files inline | Small, test-specific data |

## Coverage Requirements (Phase 6 Update)

### Targets

| Metric | Target | Status |
|--------|--------|--------|
| Line coverage | 80% | Phase 6: Heatmap tests expand coverage |
| Branch coverage | 75% | Phase 6: New conditional logic |
| Function coverage | 80% | Phase 6: Helper functions, sub-components |

### New Coverage Areas (Phase 6)

- `Heatmap`: All code paths (persistence checks, data merging, loading/error states)
- `persistence.ts`: isPersistenceEnabled, getPersistenceStatus functions
- Data merging logic: Current hour, past hours, edge cases
- Loading/error states: Timeout logic, error recovery, retry handling
- View toggling: 7-day vs 30-day view switching
- Cell interactions: Hover, click, keyboard navigation
- Accessibility: ARIA labels, keyboard support, semantic structure

### Excluded Files

Files excluded from coverage:

- `types/` - Type definitions only
- `*.config.ts` - Configuration files
- `main.tsx` - App entry point
- `mocks/` - Test infrastructure (generate during tests)

## Test Categories (Phase 6 Update)

### Client Component Tests (New for Phase 6)

| Test | Purpose | Framework | Status |
|------|---------|-----------|--------|
| `Heatmap.test.tsx` | 36 tests for heatmap component | @testing-library/react + vitest | Phase 6 |
| Persistence check (2 tests) | Feature gate verification | Direct rendering | Phase 6 |
| Data merging (3 tests) | Historic + real-time merge logic | Grid cell verification | Phase 6 |
| Loading state (3 tests) | Loading indicator and timeout | Fake timers | Phase 6 |
| Error state (4 tests) | Error handling and recovery | Retry button interaction | Phase 6 |
| View toggle (5 tests) | 7-day/30-day switching | Button state and grid size | Phase 6 |
| Cell interaction (7 tests) | Hover, click, keyboard | Tooltip, callback, navigation | Phase 6 |
| Empty state (2 tests) | No events state | Text and grid checks | Phase 6 |
| Accessibility (4 tests) | ARIA labels, keyboard, roles | Semantic structure | Phase 6 |
| Color scale (2 tests) | Legend and color computation | Style verification | Phase 6 |

### Client Unit Tests (Phase 5+)

| Test | Purpose | Framework | Status |
|------|---------|-----------|--------|
| `events.test.ts` | Event type structure | Vitest | Phase 5 |
| `formatting.test.ts` | Utility functions | Vitest | Phase 5 |

### Client Integration Tests (Phase 5+)

| Test | Purpose | Framework | Status |
|------|---------|-----------|--------|
| `App.test.tsx` | Token handling, filter state | @testing-library/react | Phase 5 |
| `useHistoricData.test.tsx` (future) | Hook with MSW | renderHook + MSW | Future |

### Persistence Tests (Phase 4)

| Test | Purpose | Framework | Status |
|------|---------|-----------|--------|
| `test_oversized_event_does_not_block_normal_events` | 413 handling | wiremock | Phase 4 |
| `test_multiple_oversized_events_all_skipped` | Batch cleanup | wiremock | Phase 4 |
| `test_normal_events_flush_successfully` | Happy path | wiremock | Phase 4 |
| `test_server_error_still_fails_flush` | 5xx handling | wiremock | Phase 4 |

## CI Integration

### Test Pipeline (Phase 6 Update)

```yaml
# Client tests
- Unit tests: npm test (vitest)
- Component tests: npm test (heatmap.test.tsx)
- Coverage: vitest --coverage

# Server/Monitor tests
- Unit tests: cargo test --lib
- Integration tests: cargo test --test '*'
- Parallel safety: cargo test -- --test-threads=1
```

### Required Checks

| Check | Blocking | Phase |
|-------|----------|-------|
| Client unit tests pass | Yes | Phase 5+ |
| Heatmap component tests pass | Yes | Phase 6 |
| Server unit tests pass | Yes | Phase 2 |
| Integration tests pass | Yes | Phase 4 |
| Coverage threshold met | Yes | Phase 4+ |
| MSW handlers verified | Yes | Phase 5+ |

## Test Setup & Utilities

### Vitest Configuration (Phase 6)

```typescript
// vite.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    // Use jsdom only for component tests via inline config
    // Other tests use default node environment
    include: ['src/__tests__/**/*.test.{ts,tsx}'],
  },
});
```

Tests can specify environment inline (Phase 6):

```typescript
/**
 * @vitest-environment happy-dom
 */
import { render } from '@testing-library/react';
```

### Test File Structure (Phase 6)

Standard test file structure for component tests:

```typescript
/**
 * Tests for [component/hook/function].
 *
 * [Description of what is tested]
 *
 * @vitest-environment happy-dom
 */

import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// Setup: save original values
const originalValue = import.meta.env.VITE_VALUE;

// Helper functions
function resetState(): void {
  useEventStore.setState({ /* clean state */ });
}

function createMockData(): TestData {
  return { /* test data */ };
}

// Global setup
beforeEach(() => {
  vi.clearAllMocks();
  resetState();
});

afterEach(() => {
  import.meta.env.VITE_VALUE = originalValue;
  resetState();
  vi.useRealTimers();
});

// Tests organized by feature
describe('Component - Feature', () => {
  it('should do X when Y', () => {
    // Arrange
    // Act
    // Assert
  });
});
```

---

*This document describes HOW to test for Phase 6 (Heatmap component testing). Update when testing strategy changes.*
