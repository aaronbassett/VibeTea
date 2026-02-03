/**
 * VibeTea Dashboard - Main application component.
 *
 * Provides a real-time dashboard for monitoring Claude Code sessions
 * with event stream, session overview, activity heatmap, and connection status.
 */

import { LazyMotion, domAnimation } from 'framer-motion';
import { useCallback, useEffect, useState } from 'react';

import { AnimatedBackground } from './components/animated/AnimatedBackground';
import { ASCIIHeader } from './components/animated/ASCIIHeader';
import { AnimationErrorBoundary } from './components/animated/ErrorBoundary';
import { SpringContainer } from './components/animated/SpringContainer';
import { ConnectionStatus } from './components/ConnectionStatus';
import { EventStream } from './components/EventStream';
import { Heatmap } from './components/Heatmap';
import { SessionOverview } from './components/SessionOverview';
import { TokenForm } from './components/TokenForm';
import { hasActiveFilters, useEventStore } from './hooks/useEventStore';
import { useSessionTimeouts } from './hooks/useSessionTimeouts';
import { TOKEN_STORAGE_KEY, useWebSocket } from './hooks/useWebSocket';

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

export default function App() {
  // Initialize session timeout checking at root level
  useSessionTimeouts();

  // Track whether user has a token saved
  const [hasToken, setHasToken] = useState<boolean>(() => hasStoredToken());

  // WebSocket connection management
  const { connect, disconnect, isConnected } = useWebSocket();

  // Connection status from store
  const status = useEventStore((state) => state.status);

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

  // Show token form if no token is saved
  if (!hasToken) {
    return (
      <LazyMotion features={domAnimation}>
        <div className="min-h-screen bg-[#131313] text-[#f5f5f5] flex flex-col items-center justify-center p-8 relative">
          {/* Animated background layer with error boundary */}
          <AnimationErrorBoundary>
            <AnimatedBackground showGrid showParticles />
          </AnimationErrorBoundary>

          {/* Content layer */}
          <div className="max-w-md w-full space-y-8 relative z-10">
            {/* ASCII Header with spring entrance and error boundary */}
            <div className="text-center">
              <AnimationErrorBoundary
                fallback={
                  <h1 className="text-3xl font-bold text-[#f5f5f5]">VibeTea</h1>
                }
              >
                <ASCIIHeader />
              </AnimationErrorBoundary>
              <SpringContainer springType="gentle" delay={0.2}>
                <p className="text-[#a0a0a0] mt-4">
                  Enter your authentication token to connect to the event
                  stream.
                </p>
              </SpringContainer>
            </div>

            {/* Token form with spring entrance */}
            <SpringContainer springType="standard" delay={0.4}>
              <TokenForm onTokenChange={handleTokenChange} />
            </SpringContainer>
          </div>
        </div>
      </LazyMotion>
    );
  }

  // Main dashboard layout
  return (
    <LazyMotion features={domAnimation}>
      <div className="min-h-screen bg-gray-900 text-white">
        {/* Header */}
        <header className="sticky top-0 z-10 bg-gray-900/95 backdrop-blur border-b border-gray-800">
          <div className="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
            <h1 className="text-xl font-bold">VibeTea Dashboard</h1>
            <div className="flex items-center gap-4">
              <ConnectionStatus showLabel />
              {!isConnected && status === 'disconnected' && (
                <button
                  type="button"
                  onClick={connect}
                  className="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900"
                >
                  Connect
                </button>
              )}
            </div>
          </div>
        </header>

        {/* Main content */}
        <main className="max-w-7xl mx-auto p-4">
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
                            <button
                              type="button"
                              onClick={() => setSessionFilter(null)}
                              className="ml-1 hover:text-purple-200"
                              aria-label="Clear session filter"
                            >
                              &times;
                            </button>
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
                            <button
                              type="button"
                              onClick={() => setTimeRangeFilter(null)}
                              className="ml-1 hover:text-cyan-200"
                              aria-label="Clear time range filter"
                            >
                              &times;
                            </button>
                          </span>
                        )}
                        <button
                          type="button"
                          onClick={clearFilters}
                          className="text-xs text-gray-400 hover:text-gray-200"
                        >
                          Clear all
                        </button>
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
