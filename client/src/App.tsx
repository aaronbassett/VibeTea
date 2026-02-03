/**
 * VibeTea Dashboard - Main application component.
 *
 * Provides a real-time dashboard for monitoring Claude Code sessions
 * with event stream, session overview, activity heatmap, and connection status.
 */

import { useCallback, useEffect, useState } from 'react';

import { ConnectionStatus } from './components/ConnectionStatus';
import { EventStream } from './components/EventStream';
import { Heatmap } from './components/Heatmap';
import { SessionOverview } from './components/SessionOverview';
import { TokenForm } from './components/TokenForm';
import { useEventStore } from './hooks/useEventStore';
import { useSessionTimeouts } from './hooks/useSessionTimeouts';
import { useWebSocket } from './hooks/useWebSocket';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** localStorage key for authentication token (must match useWebSocket) */
const TOKEN_STORAGE_KEY = 'vibetea_token';

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
   * Handle session card click for filtering (placeholder for future feature).
   */
  const handleSessionClick = useCallback((sessionId: string) => {
    console.log(`Session clicked: ${sessionId}`);
    // Future: filter event stream to show only events from this session
  }, []);

  /**
   * Handle heatmap cell click for filtering (placeholder for future feature).
   */
  const handleHeatmapCellClick = useCallback(
    (startTime: Date, endTime: Date) => {
      console.log(`Heatmap cell clicked: ${startTime} - ${endTime}`);
      // Future: filter event stream to show only events in this time range
    },
    []
  );

  // Show token form if no token is saved
  if (!hasToken) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex flex-col items-center justify-center p-8">
        <div className="max-w-md w-full space-y-8">
          <div className="text-center">
            <h1 className="text-3xl font-bold mb-2">VibeTea Dashboard</h1>
            <p className="text-gray-400">
              Enter your authentication token to connect to the event stream.
            </p>
          </div>
          <TokenForm onTokenChange={handleTokenChange} />
        </div>
      </div>
    );
  }

  // Main dashboard layout
  return (
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
              <SessionOverview onSessionClick={handleSessionClick} />
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
                <h2 className="text-lg font-semibold text-gray-100">
                  Event Stream
                </h2>
              </div>
              <EventStream className="h-[calc(100%-4rem)]" />
            </section>
          </div>
        </div>
      </main>
    </div>
  );
}
