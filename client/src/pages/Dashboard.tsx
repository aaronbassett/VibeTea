/**
 * Dashboard page component for VibeTea.
 *
 * Provides a real-time dashboard for monitoring Claude Code sessions
 * with event stream, session overview, activity heatmap, and connection status.
 * Extracted from App.tsx to separate concerns and enable page-based routing.
 */

import { LazyMotion, domAnimation, m } from 'framer-motion';
import { useCallback, useEffect, useState } from 'react';

import { SPRING_CONFIGS } from '../constants/design-tokens';
import { useReducedMotion } from '../hooks/useReducedMotion';

import { AnimatedBackground } from '../components/animated/AnimatedBackground';
import { AnimationErrorBoundary } from '../components/animated/ErrorBoundary';
import { ConnectionStatus } from '../components/ConnectionStatus';
import { EventStream } from '../components/EventStream';
import { ActivityGraph } from '../components/graphs/ActivityGraph';
import { EventDistributionChart } from '../components/graphs/EventDistributionChart';
import { Heatmap } from '../components/Heatmap';
import { SessionOverview } from '../components/SessionOverview';
import { TokenForm } from '../components/TokenForm';
import { hasActiveFilters, useEventStore } from '../hooks/useEventStore';
import type { TimeRange } from '../types/graphs';
import { useSessionTimeouts } from '../hooks/useSessionTimeouts';
import { TOKEN_STORAGE_KEY, useWebSocket } from '../hooks/useWebSocket';
import { useAuth } from '../hooks/useAuth';

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Props for the Dashboard component.
 */
export interface DashboardProps {
  /** Callback when user clicks sign out */
  onSignOut?: () => void;
}

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Check if a token exists in localStorage.
 */
function hasStoredToken(): boolean {
  return localStorage.getItem(TOKEN_STORAGE_KEY) !== null;
}

// -----------------------------------------------------------------------------
// Main Component
// -----------------------------------------------------------------------------

/**
 * Dashboard page component.
 *
 * Displays the main monitoring dashboard with:
 * - Header with user info, connection status, and sign out button
 * - Session overview (left column)
 * - Activity heatmap (left column)
 * - Activity trends graph (left column)
 * - Event distribution chart (left column)
 * - Settings with token form (left column)
 * - Real-time event stream (right column)
 *
 * @param props - Component props
 * @returns The rendered dashboard
 */
export function Dashboard({ onSignOut }: DashboardProps) {
  // Initialize session timeout checking at root level
  useSessionTimeouts();

  // Get user info from auth hook
  const { user, signOut } = useAuth();

  // Respect user's reduced motion preference (FR-008)
  const prefersReducedMotion = useReducedMotion();

  // Track whether user has a token saved
  const [hasToken, setHasToken] = useState<boolean>(() => hasStoredToken());

  // Time range for activity graph
  const [timeRange, setTimeRange] = useState<TimeRange>('1h');

  // WebSocket connection management
  const { connect, disconnect, isConnected } = useWebSocket();

  // Connection status from store
  const status = useEventStore((state) => state.status);

  // Events for graphs
  const events = useEventStore((state) => state.events);

  // Filter state and actions
  const filters = useEventStore((state) => state.filters);
  const setSessionFilter = useEventStore((state) => state.setSessionFilter);
  const setTimeRangeFilter = useEventStore((state) => state.setTimeRangeFilter);
  const clearFilters = useEventStore((state) => state.clearFilters);
  const filtersActive = useEventStore(hasActiveFilters);
  const sessions = useEventStore((state) => state.sessions);

  // Connect when token becomes available
  useEffect(() => {
    if (hasToken) {
      connect();
    }
    return () => {
      disconnect();
    };
  }, [hasToken, connect, disconnect]);

  /**
   * Handle token changes from TokenForm.
   */
  const handleTokenChange = useCallback(() => {
    const tokenExists = hasStoredToken();
    setHasToken(tokenExists);

    if (tokenExists) {
      // Reconnect with new token
      disconnect();
      connect();
    }
  }, [connect, disconnect]);

  /**
   * Handle session card click for filtering.
   * Clicking the same session again clears the filter.
   */
  const handleSessionClick = useCallback(
    (sessionId: string) => {
      if (filters.sessionId === sessionId) {
        // Toggle off if clicking the same session
        setSessionFilter(null);
      } else {
        setSessionFilter(sessionId);
      }
    },
    [filters.sessionId, setSessionFilter]
  );

  /**
   * Handle heatmap cell click for filtering.
   * Clicking sets the time range filter.
   */
  const handleHeatmapCellClick = useCallback(
    (startTime: Date, endTime: Date) => {
      setTimeRangeFilter({ start: startTime, end: endTime });
    },
    [setTimeRangeFilter]
  );

  /**
   * Handle sign out action.
   * Calls Supabase signOut and optional callback.
   */
  const handleSignOut = useCallback(async () => {
    try {
      await signOut();
      onSignOut?.();
    } catch (err) {
      console.error('[Dashboard] Sign out failed:', err);
    }
  }, [signOut, onSignOut]);

  // Extract user display info from Supabase user metadata
  const userDisplayName =
    user?.user_metadata?.full_name ??
    user?.user_metadata?.name ??
    user?.email ??
    'User';
  const userAvatarUrl = user?.user_metadata?.avatar_url ?? null;

  // Main dashboard layout
  return (
    <LazyMotion features={domAnimation}>
      <div className="min-h-screen bg-[#131313] text-white relative">
        {/* Animated background layer with error boundary */}
        <AnimationErrorBoundary>
          <AnimatedBackground showGrid showParticles />
        </AnimationErrorBoundary>

        {/* Header */}
        <header className="sticky top-0 z-10 bg-[#131313]/95 backdrop-blur border-b border-gray-800 relative">
          <div className="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
            <h1 className="text-xl font-bold">VibeTea Dashboard</h1>
            <div className="flex items-center gap-4">
              {/* User info section */}
              {user && (
                <div className="flex items-center gap-3">
                  {userAvatarUrl && (
                    <img
                      src={userAvatarUrl}
                      alt={`${userDisplayName}'s avatar`}
                      className="w-8 h-8 rounded-full border border-gray-600"
                    />
                  )}
                  <span className="text-sm text-gray-300 hidden sm:inline">
                    {userDisplayName}
                  </span>
                </div>
              )}

              <ConnectionStatus showLabel />

              {!isConnected && status === 'disconnected' && (
                <m.button
                  type="button"
                  onClick={connect}
                  className="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900"
                  whileHover={
                    prefersReducedMotion
                      ? undefined
                      : {
                          scale: 1.05,
                          boxShadow: '0 0 12px 2px rgba(59, 130, 246, 0.4)',
                          transition: SPRING_CONFIGS.gentle,
                        }
                  }
                  whileTap={prefersReducedMotion ? undefined : { scale: 0.95 }}
                >
                  Connect
                </m.button>
              )}

              {/* Sign out button */}
              <m.button
                type="button"
                onClick={handleSignOut}
                className="px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 focus:ring-offset-gray-900"
                whileHover={
                  prefersReducedMotion
                    ? undefined
                    : {
                        scale: 1.05,
                        transition: SPRING_CONFIGS.gentle,
                      }
                }
                whileTap={prefersReducedMotion ? undefined : { scale: 0.95 }}
              >
                Sign out
              </m.button>
            </div>
          </div>
        </header>

        {/* Main content */}
        <main className="max-w-7xl mx-auto p-4 relative z-10">
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Left column: Sessions and Heatmap */}
            <div className="lg:col-span-1 space-y-6">
              {/* Session Overview */}
              <section className="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
                <SessionOverview
                  onSessionClick={handleSessionClick}
                  selectedSessionId={filters.sessionId}
                />
              </section>

              {/* Activity Heatmap */}
              <section className="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
                <Heatmap onCellClick={handleHeatmapCellClick} />
              </section>

              {/* Activity Trends */}
              <section className="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
                <h2 className="text-lg font-semibold text-gray-100 mb-4">
                  Activity Trends
                </h2>
                <AnimationErrorBoundary>
                  <div style={{ height: 200 }}>
                    <ActivityGraph
                      events={events}
                      timeRange={timeRange}
                      onTimeRangeChange={setTimeRange}
                    />
                  </div>
                </AnimationErrorBoundary>
              </section>

              {/* Event Distribution */}
              <section className="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
                <h2 className="text-lg font-semibold text-gray-100 mb-4">
                  Event Distribution
                </h2>
                <AnimationErrorBoundary>
                  <div style={{ height: 200 }}>
                    <EventDistributionChart events={events} />
                  </div>
                </AnimationErrorBoundary>
              </section>

              {/* Token Management */}
              <section className="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
                <h2 className="text-lg font-semibold text-gray-100 mb-4">
                  Settings
                </h2>
                <TokenForm onTokenChange={handleTokenChange} />
              </section>
            </div>

            {/* Right column: Event Stream */}
            <div className="lg:col-span-2">
              <section className="bg-gray-800/50 rounded-lg border border-gray-700 overflow-hidden h-[calc(100vh-8rem)]">
                <div className="p-4 border-b border-gray-700">
                  <div className="flex items-center justify-between">
                    <h2 className="text-lg font-semibold text-gray-100">
                      Event Stream
                    </h2>
                    {filtersActive && (
                      <div className="flex items-center gap-2">
                        {filters.sessionId !== null && (
                          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs bg-purple-600/20 text-purple-400 border border-purple-500/30 rounded-md">
                            Session:{' '}
                            {sessions.get(filters.sessionId)?.project ??
                              filters.sessionId.slice(0, 8)}
                            <m.button
                              type="button"
                              onClick={() => setSessionFilter(null)}
                              className="ml-1 hover:text-purple-200 focus:outline-none focus:ring-1 focus:ring-purple-400 rounded-sm"
                              aria-label="Clear session filter"
                              whileHover={
                                prefersReducedMotion
                                  ? undefined
                                  : {
                                      scale: 1.2,
                                      transition: SPRING_CONFIGS.gentle,
                                    }
                              }
                              whileTap={
                                prefersReducedMotion
                                  ? undefined
                                  : { scale: 0.9 }
                              }
                            >
                              &times;
                            </m.button>
                          </span>
                        )}
                        {filters.timeRange !== null && (
                          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs bg-cyan-600/20 text-cyan-400 border border-cyan-500/30 rounded-md">
                            {filters.timeRange.start.toLocaleTimeString(
                              'en-US',
                              {
                                hour12: false,
                                hour: '2-digit',
                                minute: '2-digit',
                              }
                            )}{' '}
                            -{' '}
                            {filters.timeRange.end.toLocaleTimeString('en-US', {
                              hour12: false,
                              hour: '2-digit',
                              minute: '2-digit',
                            })}
                            <m.button
                              type="button"
                              onClick={() => setTimeRangeFilter(null)}
                              className="ml-1 hover:text-cyan-200 focus:outline-none focus:ring-1 focus:ring-cyan-400 rounded-sm"
                              aria-label="Clear time range filter"
                              whileHover={
                                prefersReducedMotion
                                  ? undefined
                                  : {
                                      scale: 1.2,
                                      transition: SPRING_CONFIGS.gentle,
                                    }
                              }
                              whileTap={
                                prefersReducedMotion
                                  ? undefined
                                  : { scale: 0.9 }
                              }
                            >
                              &times;
                            </m.button>
                          </span>
                        )}
                        <m.button
                          type="button"
                          onClick={clearFilters}
                          className="text-xs text-gray-400 hover:text-gray-200 focus:outline-none focus:ring-1 focus:ring-gray-400 rounded px-1"
                          whileHover={
                            prefersReducedMotion
                              ? undefined
                              : {
                                  scale: 1.05,
                                  transition: SPRING_CONFIGS.gentle,
                                }
                          }
                          whileTap={
                            prefersReducedMotion ? undefined : { scale: 0.95 }
                          }
                        >
                          Clear all
                        </m.button>
                      </div>
                    )}
                  </div>
                </div>
                <EventStream className="h-[calc(100%-4rem)]" />
              </section>
            </div>
          </div>
        </main>
      </div>
    </LazyMotion>
  );
}
