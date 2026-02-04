/**
 * Tests for App component authentication routing.
 *
 * @vitest-environment happy-dom
 */

import { render, screen, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { beforeEach, describe, expect, it, vi } from 'vitest';

// Mock Supabase before importing App
vi.mock('../services/supabase', () => ({
  supabase: {
    auth: {
      getSession: vi
        .fn()
        .mockResolvedValue({ data: { session: null }, error: null }),
      onAuthStateChange: vi.fn().mockReturnValue({
        data: { subscription: { unsubscribe: vi.fn() } },
      }),
      signInWithOAuth: vi.fn().mockResolvedValue({ error: null }),
      signOut: vi.fn().mockResolvedValue({ error: null }),
    },
  },
  getSession: vi.fn().mockResolvedValue(null),
}));

import App from '../App';
import { useEventStore } from '../hooks/useEventStore';
import { supabase } from '../services/supabase';

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
  vi.clearAllMocks();
});

describe('App Authentication Routing', () => {
  it('shows loading state initially', () => {
    // Mock getSession to delay
    vi.mocked(supabase.auth.getSession).mockImplementation(
      () => new Promise(() => {}) // Never resolves - keeps loading state
    );

    render(<App />);

    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('renders login page when not authenticated', async () => {
    // Mock no session
    vi.mocked(supabase.auth.getSession).mockResolvedValue({
      data: { session: null },
      error: null,
    });

    render(<App />);

    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: /sign in with github/i })
      ).toBeInTheDocument();
    });
  });

  it('renders dashboard when authenticated', async () => {
    // Mock authenticated session
    const mockUser = {
      id: 'test-user-id',
      email: 'test@example.com',
      user_metadata: {
        full_name: 'Test User',
        avatar_url: 'https://example.com/avatar.jpg',
      },
    };

    const mockSession = {
      user: mockUser,
      access_token: 'test-access-token',
      refresh_token: 'test-refresh-token',
    };

    vi.mocked(supabase.auth.getSession).mockResolvedValue({
      data: { session: mockSession },
      error: null,
    });

    // Also need to mock onAuthStateChange to emit the session
    vi.mocked(supabase.auth.onAuthStateChange).mockImplementation(
      (callback) => {
        // Immediately call with the session
        setTimeout(() => callback('SIGNED_IN', mockSession), 0);
        return { data: { subscription: { unsubscribe: vi.fn() } } };
      }
    );

    render(<App />);

    await waitFor(() => {
      expect(screen.getByText('VibeTea Dashboard')).toBeInTheDocument();
    });
  });

  it('shows sign out button when authenticated', async () => {
    const mockUser = {
      id: 'test-user-id',
      email: 'test@example.com',
      user_metadata: { full_name: 'Test User' },
    };

    const mockSession = {
      user: mockUser,
      access_token: 'test-access-token',
      refresh_token: 'test-refresh-token',
    };

    vi.mocked(supabase.auth.getSession).mockResolvedValue({
      data: { session: mockSession },
      error: null,
    });

    vi.mocked(supabase.auth.onAuthStateChange).mockImplementation(
      (callback) => {
        setTimeout(() => callback('SIGNED_IN', mockSession), 0);
        return { data: { subscription: { unsubscribe: vi.fn() } } };
      }
    );

    render(<App />);

    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: /sign out/i })
      ).toBeInTheDocument();
    });
  });
});

describe('App Filter Integration', () => {
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
});
