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
            className="block text-sm font-medium text-[#f5f5f5] mb-2"
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
            className="w-full px-4 py-2 bg-[#1a1a1a] border border-[#2a2a2a] rounded-lg text-[#f5f5f5] placeholder-[#6b6b6b] focus:outline-none focus:ring-2 focus:ring-[#d97757] focus:border-transparent"
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
              isSaved ? 'bg-[#4ade80]' : 'bg-[#6b6b6b]'
            }`}
            aria-hidden="true"
          />
          <span className="text-sm text-[#a0a0a0]">
            {isSaved ? 'Token saved' : 'No token saved'}
          </span>
        </div>

        <div className="flex gap-3">
          <button
            type="submit"
            disabled={!canSave}
            className="flex-1 px-4 py-2 bg-[#d97757] hover:bg-[#e89a7a] disabled:bg-[#242424] disabled:text-[#6b6b6b] disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-[#d97757] focus:ring-offset-2 focus:ring-offset-[#131313]"
          >
            Save Token
          </button>
          <button
            type="button"
            onClick={handleClear}
            disabled={!isSaved}
            className="px-4 py-2 bg-[#242424] hover:bg-[#2a2a2a] disabled:bg-[#1a1a1a] disabled:text-[#6b6b6b] disabled:cursor-not-allowed text-[#f5f5f5] font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-[#d97757] focus:ring-offset-2 focus:ring-offset-[#131313]"
          >
            Clear
          </button>
        </div>
      </form>
    </div>
  );
}
