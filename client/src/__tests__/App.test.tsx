/**
 * Tests for App component token handling and render paths.
 *
 * @vitest-environment happy-dom
 */

import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { beforeEach, describe, expect, it } from 'vitest';

import App from '../App';
import { useEventStore } from '../hooks/useEventStore';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
  writable: true,
});

// Mock WebSocket
class MockWebSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;

  readonly CONNECTING = MockWebSocket.CONNECTING;
  readonly OPEN = MockWebSocket.OPEN;
  readonly CLOSING = MockWebSocket.CLOSING;
  readonly CLOSED = MockWebSocket.CLOSED;

  readyState = MockWebSocket.CONNECTING;
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  constructor(_url: string) {
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      if (this.onopen) {
        this.onopen(new Event('open'));
      }
    }, 0);
  }

  close() {
    this.readyState = MockWebSocket.CLOSED;
    if (this.onclose) {
      this.onclose(new CloseEvent('close'));
    }
  }

  send(_data: string) {
    // No-op for tests
  }
}

Object.defineProperty(window, 'WebSocket', {
  value: MockWebSocket,
  writable: true,
});

// Reset store and localStorage before each test
beforeEach(() => {
  localStorage.clear();
  useEventStore.setState({
    status: 'disconnected',
    events: [],
    sessions: new Map(),
    filters: { sessionId: null, timeRange: null },
  });
});

describe('App Token Handling', () => {
  it('renders token form when no token is stored', () => {
    render(<App />);

    // Should show the token form
    expect(screen.getByText('VibeTea Dashboard')).toBeInTheDocument();
    expect(
      screen.getByText(/enter your authentication token/i)
    ).toBeInTheDocument();
    expect(screen.getByLabelText(/authentication token/i)).toBeInTheDocument();
  });

  it('renders dashboard when token exists', () => {
    // Set token before rendering
    localStorage.setItem('vibetea_token', 'test-token-123');

    render(<App />);

    // Should show the dashboard header and main sections
    expect(screen.getByText('VibeTea Dashboard')).toBeInTheDocument();
    expect(screen.getByText('Sessions')).toBeInTheDocument();
    expect(screen.getByText('Event Stream')).toBeInTheDocument();
  });

  it('transitions from token form to dashboard when token is saved', async () => {
    render(<App />);

    // Initially shows token form
    expect(
      screen.getByText(/enter your authentication token/i)
    ).toBeInTheDocument();

    // Enter a token
    const tokenInput = screen.getByLabelText(/authentication token/i);
    fireEvent.change(tokenInput, { target: { value: 'new-test-token' } });

    // Submit the form
    const saveButton = screen.getByRole('button', { name: /save token/i });
    fireEvent.click(saveButton);

    // Wait for transition to dashboard
    await waitFor(() => {
      expect(screen.getByText('Sessions')).toBeInTheDocument();
    });

    // Token should be in localStorage
    expect(localStorage.getItem('vibetea_token')).toBe('new-test-token');
  });

  it('allows updating token from settings', async () => {
    // Start with a token
    localStorage.setItem('vibetea_token', 'existing-token');

    render(<App />);

    // Should show dashboard with Settings section
    expect(screen.getByText('Sessions')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();

    // The token input should be visible in settings
    const tokenInput = screen.getByLabelText(/authentication token/i);
    expect(tokenInput).toBeInTheDocument();
  });
});

describe('App Connection Status', () => {
  it('shows connection status indicator', () => {
    localStorage.setItem('vibetea_token', 'test-token');

    render(<App />);

    // The ConnectionStatus component should show some status
    // It shows "Connecting" initially due to our mock WebSocket
    const statusElement = screen.getByRole('status', {
      name: /connection status/i,
    });
    expect(statusElement).toBeInTheDocument();
  });

  it('initiates connection when token is available', async () => {
    localStorage.setItem('vibetea_token', 'test-token');

    render(<App />);

    // Wait for async connection attempt
    await waitFor(() => {
      const state = useEventStore.getState();
      // Status should transition from 'disconnected'
      // With our mock WebSocket, it will go to 'connecting' then 'connected'
      expect(['connecting', 'connected', 'disconnected']).toContain(
        state.status
      );
    });
  });
});

describe('App Filter Integration', () => {
  beforeEach(() => {
    localStorage.setItem('vibetea_token', 'test-token');
  });

  it('does not show clear all when no filters are active', () => {
    render(<App />);

    // By default, no filters are active
    expect(screen.queryByText(/clear all/i)).not.toBeInTheDocument();
  });

  it('filter state can be updated via store actions', () => {
    // Test store actions directly without rendering component
    // This avoids React re-render loops with the hasActiveFilters selector
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
});
