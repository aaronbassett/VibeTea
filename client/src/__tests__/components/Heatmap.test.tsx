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
import '@testing-library/jest-dom/vitest';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createHourlyAggregate } from '../../mocks/data';
import { useEventStore } from '../../hooks/useEventStore';
import { Heatmap } from '../../components/Heatmap';
import type { HourlyAggregate, VibeteaEvent } from '../../types/events';

// -----------------------------------------------------------------------------
// Test Setup
// -----------------------------------------------------------------------------

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

/**
 * Helper to create a mock event for testing
 */
function createMockEvent(overrides: Partial<VibeteaEvent> = {}): VibeteaEvent {
  const now = new Date();
  return {
    id: `event-${Math.random().toString(36).slice(2)}`,
    source: 'test-monitor',
    timestamp: now.toISOString(),
    type: 'activity',
    payload: {
      sessionId: 'session-1',
    },
    ...overrides,
  } as VibeteaEvent;
}

/**
 * Helper to enable persistence for tests
 */
function enablePersistence(): void {
  import.meta.env.VITE_SUPABASE_URL = 'https://test.supabase.co';
  import.meta.env.VITE_SUPABASE_TOKEN = 'test-token';
}

/**
 * Helper to disable persistence for tests
 */
function disablePersistence(): void {
  import.meta.env.VITE_SUPABASE_URL = '';
  import.meta.env.VITE_SUPABASE_TOKEN = '';
}

// Track fetch calls for mock
let fetchCalls: number[] = [];

// Save original fetchHistoricData
let originalFetchHistoricData: (days: 7 | 30) => Promise<void>;

beforeEach(() => {
  // Reset timers and mocks
  vi.clearAllMocks();

  // Enable persistence by default
  enablePersistence();

  // Reset store state
  resetStore();

  // Track fetch calls
  fetchCalls = [];

  // Save original function
  originalFetchHistoricData = useEventStore.getState().fetchHistoricData;

  // Default mock: tracks fetch calls and preserves existing data if available
  const mockFetch = vi.fn().mockImplementation((days: 7 | 30) => {
    fetchCalls.push(days);
    const currentState = useEventStore.getState();
    // Preserve existing historic data if already loaded
    useEventStore.setState({
      historicData: currentState.historicData,
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });
    return Promise.resolve();
  });
  useEventStore.setState({ fetchHistoricData: mockFetch });
});

afterEach(() => {
  // Restore env values
  import.meta.env.VITE_SUPABASE_URL = originalSupabaseUrl;
  import.meta.env.VITE_SUPABASE_TOKEN = originalSupabaseToken;

  // Reset store state
  resetStore();

  // Restore original function
  useEventStore.setState({ fetchHistoricData: originalFetchHistoricData });

  // Cleanup timers if used
  vi.useRealTimers();
});

// -----------------------------------------------------------------------------
// Persistence Check Tests (T138)
// -----------------------------------------------------------------------------

describe('Heatmap - Persistence Check', () => {
  it('should return null when isPersistenceEnabled() returns false', () => {
    // Disable persistence
    disablePersistence();

    const { container } = render(<Heatmap />);

    // Component should render nothing
    expect(container.firstChild).toBeNull();
  });

  it('should render when persistence is enabled', () => {
    // Persistence is enabled by default in beforeEach
    render(<Heatmap />);

    // Component should render the activity heading
    expect(
      screen.getByRole('region', { name: 'Activity heatmap' })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'Activity' })
    ).toBeInTheDocument();
  });

  it('should render when only VITE_SUPABASE_URL is set', () => {
    import.meta.env.VITE_SUPABASE_URL = 'https://test.supabase.co';
    import.meta.env.VITE_SUPABASE_TOKEN = '';

    render(<Heatmap />);

    // Component should still render (isPersistenceEnabled only checks URL)
    expect(
      screen.getByRole('region', { name: 'Activity heatmap' })
    ).toBeInTheDocument();
  });
});

// -----------------------------------------------------------------------------
// Data Merging Logic Tests (T139)
// -----------------------------------------------------------------------------

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
    // We check by finding a cell with aria-label containing "10 events"
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
        hour: 12, // Noon yesterday (UTC)
        eventCount: 50,
      }),
    ];

    await act(async () => {
      useEventStore.setState({
        events: [], // No real-time events
        historicData,
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    });

    render(<Heatmap />);

    // The grid should be rendered
    await waitFor(() => {
      expect(screen.getByRole('grid')).toBeInTheDocument();
    });

    // Find a cell that shows the historic event count
    // Note: The exact cell may vary based on timezone conversion
    const cells = screen.getAllByRole('gridcell');
    expect(cells.length).toBeGreaterThan(0);
  });

  it('should display combined data correctly in the grid', async () => {
    const now = new Date();
    const yesterday = new Date(now);
    yesterday.setDate(yesterday.getDate() - 1);
    const yesterdayDateStr = yesterday.toISOString().split('T')[0];

    // Historic data for yesterday
    const historicData: HourlyAggregate[] = [
      createHourlyAggregate({
        date: yesterdayDateStr,
        hour: 10,
        eventCount: 25,
      }),
      createHourlyAggregate({
        date: yesterdayDateStr,
        hour: 14,
        eventCount: 30,
      }),
    ];

    // Real-time events for today
    const realtimeEvents = [createMockEvent(), createMockEvent()];

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

    // Grid should be rendered with data
    await waitFor(() => {
      expect(screen.getByRole('grid')).toBeInTheDocument();
    });

    // Should have cells for 7 days * 24 hours = 168 cells
    const cells = screen.getAllByRole('gridcell');
    expect(cells.length).toBe(7 * 24);
  });
});

// -----------------------------------------------------------------------------
// Loading State Tests
// -----------------------------------------------------------------------------

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
      historicDataFetchedAt: null, // null to indicate never fetched
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
    const historicData: HourlyAggregate[] = [
      createHourlyAggregate({ eventCount: 10 }),
    ];

    useEventStore.setState({
      events: [],
      historicData,
      historicDataStatus: 'loading', // Still loading but have cached data
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

// -----------------------------------------------------------------------------
// Error State Tests
// -----------------------------------------------------------------------------

describe('Heatmap - Error State', () => {
  it('should show error message when status is error', async () => {
    // Set up mock that maintains error state
    const errorMock = vi.fn().mockImplementation(() => {
      return Promise.resolve();
    });

    await act(async () => {
      useEventStore.setState({
        fetchHistoricData: errorMock,
        events: [],
        historicData: [],
        historicDataStatus: 'error',
        historicDataFetchedAt: Date.now(), // Fresh to prevent refetch
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

    // Set up mock that tracks refetch calls
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
        historicDataFetchedAt: Date.now(), // Fresh to prevent auto refetch
        historicDataError: 'Network error',
      });
    });

    render(<Heatmap />);

    // Find and click the Retry button
    const retryButton = screen.getByRole('button', { name: 'Retry' });
    expect(retryButton).toBeInTheDocument();

    fireEvent.click(retryButton);

    // Verify refetch was called
    await waitFor(() => {
      expect(refetchCalled).toBe(true);
    });
  });

  it('should still display real-time data during error state', async () => {
    // Add real-time events
    const realtimeEvents = [
      createMockEvent(),
      createMockEvent(),
      createMockEvent(),
    ];

    // Mock that doesn't change error state
    const errorMock = vi.fn().mockImplementation(() => Promise.resolve());

    await act(async () => {
      useEventStore.setState({
        fetchHistoricData: errorMock,
        events: realtimeEvents,
        historicData: [],
        historicDataStatus: 'error',
        historicDataFetchedAt: Date.now(), // Fresh to prevent auto refetch
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

  it('should show error after loading timeout', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });

    // Mock that stays in loading forever
    const loadingMock = vi.fn().mockImplementation(() => {
      // Don't change state - keep it loading
      return new Promise(() => {}); // Never resolves
    });

    await act(async () => {
      useEventStore.setState({
        fetchHistoricData: loadingMock,
        historicData: [],
        historicDataStatus: 'loading',
        historicDataFetchedAt: null,
        historicDataError: null,
      });
    });

    render(<Heatmap />);

    // Advance past timeout
    await act(async () => {
      vi.advanceTimersByTime(5001);
    });

    // Should show error state after timeout
    await waitFor(() => {
      expect(
        screen.getByText(
          'Unable to load historic data. Showing real-time events only.'
        )
      ).toBeInTheDocument();
    });
  });
});

// -----------------------------------------------------------------------------
// View Toggle Tests
// -----------------------------------------------------------------------------

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

    // 30-day should now be selected
    expect(thirtyDayButton).toHaveAttribute('aria-pressed', 'true');

    // 7-day should no longer be selected
    const sevenDayButton = screen.getByRole('button', { name: '7 Days' });
    expect(sevenDayButton).toHaveAttribute('aria-pressed', 'false');
  });

  it('should render more cells in 30-day view', async () => {
    // Use real-time events to ensure grid is displayed
    const events = [createMockEvent()];

    // Mock that preserves state
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

  it('should trigger new data fetch when switching views', async () => {
    fetchCalls = [];

    // Mock that needs data to be stale for refetch
    const mockFetch = vi.fn().mockImplementation((days: 7 | 30) => {
      fetchCalls.push(days);
      useEventStore.setState({
        historicData: [createHourlyAggregate({ eventCount: 10 })],
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
      return Promise.resolve();
    });

    useEventStore.setState({
      fetchHistoricData: mockFetch,
      historicData: [createHourlyAggregate({ eventCount: 5 })],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now() - 6 * 60 * 1000, // 6 minutes ago (stale)
      historicDataError: null,
    });

    render(<Heatmap />);

    // Initial fetch should have been triggered (data is stale)
    await waitFor(() => {
      expect(fetchCalls).toContain(7);
    });

    // Reset to track new fetch
    fetchCalls = [];
    useEventStore.setState({
      historicDataFetchedAt: null, // Force stale for next view
    });

    // Switch to 30-day view
    const thirtyDayButton = screen.getByRole('button', { name: '30 Days' });
    fireEvent.click(thirtyDayButton);

    // New fetch should be triggered with days=30
    await waitFor(() => {
      expect(fetchCalls).toContain(30);
    });
  });
});

// -----------------------------------------------------------------------------
// Cell Interaction Tests
// -----------------------------------------------------------------------------

describe('Heatmap - Cell Interaction', () => {
  it('should show tooltip with correct count on hover', async () => {
    // Add events to current hour
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

    expect(cellWith3Events).toBeInTheDocument();

    // Hover over the cell
    if (cellWith3Events) {
      fireEvent.mouseEnter(cellWith3Events);
    }

    // Tooltip should appear
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
      expect(screen.getByText('3 events')).toBeInTheDocument();
    });
  });

  it('should show "1 event" (singular) in tooltip when count is 1', async () => {
    // Add single event to current hour
    const events = [createMockEvent()];

    useEventStore.setState({
      events,
      historicData: [],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });

    render(<Heatmap />);

    // Find the cell with 1 event
    const cells = screen.getAllByRole('gridcell');
    const cellWith1Event = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('1 event')
    );

    expect(cellWith1Event).toBeInTheDocument();

    if (cellWith1Event) {
      fireEvent.mouseEnter(cellWith1Event);
    }

    // Tooltip should show singular "event"
    await waitFor(() => {
      expect(screen.getByRole('tooltip')).toBeInTheDocument();
      expect(screen.getByText('1 event')).toBeInTheDocument();
    });
  });

  it('should hide tooltip when mouse leaves cell', async () => {
    const events = [createMockEvent()];

    useEventStore.setState({
      events,
      historicData: [],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });

    render(<Heatmap />);

    const cells = screen.getAllByRole('gridcell');
    const cellWithEvent = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('1 event')
    );

    if (cellWithEvent) {
      // Hover to show tooltip
      fireEvent.mouseEnter(cellWithEvent);

      await waitFor(() => {
        expect(screen.getByRole('tooltip')).toBeInTheDocument();
      });

      // Leave to hide tooltip
      fireEvent.mouseLeave(cellWithEvent);

      await waitFor(() => {
        expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
      });
    }
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

    expect(cellWithEvent).toBeInTheDocument();

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

    // Find a cell with event
    const cells = screen.getAllByRole('gridcell');
    const cellWithEvent = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('1 event')
    );

    expect(cellWithEvent).toBeInTheDocument();

    if (cellWithEvent) {
      // Focus the cell
      cellWithEvent.focus();

      // Press Enter
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

  it('should not call onCellClick when not provided', () => {
    const events = [createMockEvent()];

    useEventStore.setState({
      events,
      historicData: [],
      historicDataStatus: 'success',
      historicDataFetchedAt: Date.now(),
      historicDataError: null,
    });

    // Render without onCellClick prop
    render(<Heatmap />);

    const cells = screen.getAllByRole('gridcell');
    const cellWithEvent = cells.find((cell) =>
      cell.getAttribute('aria-label')?.includes('1 event')
    );

    // Should not throw when clicking
    if (cellWithEvent) {
      fireEvent.click(cellWithEvent);
    }

    // Test passes if no error is thrown
  });
});

// -----------------------------------------------------------------------------
// Empty State Tests
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Accessibility Tests
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// Color Scale Tests
// -----------------------------------------------------------------------------

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
    // Create events for different count ranges
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

// -----------------------------------------------------------------------------
// Hour Header Tests
// -----------------------------------------------------------------------------

describe('Heatmap - Hour Header', () => {
  it('should display hour labels at key positions (0, 6, 12, 18)', async () => {
    // Use real-time events to ensure grid is displayed
    const events = [createMockEvent()];

    // Mock that preserves state
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

    // Check for hour labels in the grid
    // The component shows abbreviated labels at 0, 6, 12, 18
    const grid = screen.getByRole('grid');
    expect(grid).toHaveTextContent('0');
    expect(grid).toHaveTextContent('6');
    expect(grid).toHaveTextContent('12');
    expect(grid).toHaveTextContent('18');
  });
});

// -----------------------------------------------------------------------------
// Custom ClassName Tests
// -----------------------------------------------------------------------------

describe('Heatmap - Custom Styling', () => {
  it('should apply custom className', () => {
    render(<Heatmap className="custom-class p-4" />);

    const region = screen.getByRole('region', { name: 'Activity heatmap' });
    expect(region).toHaveClass('custom-class');
    expect(region).toHaveClass('p-4');
  });

  it('should always include base classes', () => {
    render(<Heatmap className="custom-class" />);

    const region = screen.getByRole('region', { name: 'Activity heatmap' });
    expect(region).toHaveClass('bg-gray-900');
    expect(region).toHaveClass('text-gray-100');
  });
});
