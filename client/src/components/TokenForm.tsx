/**
 * Token input form for WebSocket authentication.
 *
 * Allows users to enter, save, and clear their authentication token
 * which is stored in localStorage for WebSocket connection.
 */

import type { ChangeEvent, FormEvent } from 'react';
import { useCallback, useEffect, useState } from 'react';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** localStorage key for authentication token (must match useWebSocket) */
const TOKEN_STORAGE_KEY = 'vibetea_token';

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Props for the TokenForm component.
 */
interface TokenFormProps {
  /** Callback invoked when the token is saved or cleared. */
  readonly onTokenChange?: () => void;
  /** Additional CSS classes to apply to the container. */
  readonly className?: string;
}

/**
 * Token status for display purposes.
 */
type TokenStatus = 'saved' | 'not-saved';

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Check if a token exists in localStorage.
 *
 * @returns Whether a token is currently saved
 */
function hasStoredToken(): boolean {
  return localStorage.getItem(TOKEN_STORAGE_KEY) !== null;
}

// -----------------------------------------------------------------------------
// Component
// -----------------------------------------------------------------------------

/**
 * Form for managing the authentication token.
 *
 * Provides a password input for entering the token, with save and clear buttons.
 * Token is persisted to localStorage for use by the WebSocket connection.
 *
 * @example
 * ```tsx
 * function Settings() {
 *   const { connect } = useWebSocket();
 *
 *   return (
 *     <TokenForm
 *       onTokenChange={() => {
 *         // Reconnect when token changes
 *         connect();
 *       }}
 *     />
 *   );
 * }
 * ```
 */
export function TokenForm({ onTokenChange, className = '' }: TokenFormProps) {
  const [tokenInput, setTokenInput] = useState<string>('');
  const [status, setStatus] = useState<TokenStatus>(() =>
    hasStoredToken() ? 'saved' : 'not-saved'
  );

  // Update status when localStorage changes from another tab/window
  useEffect(() => {
    const handleStorageChange = (event: StorageEvent) => {
      if (event.key === TOKEN_STORAGE_KEY) {
        setStatus(event.newValue !== null ? 'saved' : 'not-saved');
      }
    };

    window.addEventListener('storage', handleStorageChange);
    return () => window.removeEventListener('storage', handleStorageChange);
  }, []);

  /**
   * Save the token to localStorage.
   */
  const handleSave = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();

      const trimmedToken = tokenInput.trim();
      if (trimmedToken === '') {
        return;
      }

      localStorage.setItem(TOKEN_STORAGE_KEY, trimmedToken);
      setStatus('saved');
      setTokenInput('');

      onTokenChange?.();
    },
    [tokenInput, onTokenChange]
  );

  /**
   * Clear the token from localStorage.
   */
  const handleClear = useCallback(() => {
    localStorage.removeItem(TOKEN_STORAGE_KEY);
    setStatus('not-saved');
    setTokenInput('');

    onTokenChange?.();
  }, [onTokenChange]);

  /**
   * Handle input changes.
   */
  const handleInputChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setTokenInput(event.target.value);
    },
    []
  );

  const isSaved = status === 'saved';
  const canSave = tokenInput.trim() !== '';

  return (
    <div className={`w-full max-w-md ${className}`}>
      <form onSubmit={handleSave} className="space-y-4">
        <div>
          <label
            htmlFor="token-input"
            className="block text-sm font-medium text-gray-200 mb-2"
          >
            Authentication Token
          </label>
          <input
            id="token-input"
            type="password"
            value={tokenInput}
            onChange={handleInputChange}
            placeholder={
              isSaved ? 'Enter new token to update' : 'Enter your token'
            }
            autoComplete="off"
            className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            aria-describedby="token-status"
          />
        </div>

        <div
          id="token-status"
          className="flex items-center gap-2"
          role="status"
          aria-live="polite"
        >
          <span
            className={`h-2 w-2 rounded-full ${
              isSaved ? 'bg-green-500' : 'bg-gray-500'
            }`}
            aria-hidden="true"
          />
          <span className="text-sm text-gray-400">
            {isSaved ? 'Token saved' : 'No token saved'}
          </span>
        </div>

        <div className="flex gap-3">
          <button
            type="submit"
            disabled={!canSave}
            className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900"
          >
            Save Token
          </button>
          <button
            type="button"
            onClick={handleClear}
            disabled={!isSaved}
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:text-gray-600 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 focus:ring-offset-gray-900"
          >
            Clear
          </button>
        </div>
      </form>
    </div>
  );
}
