/**
 * Token management form for WebSocket authentication.
 *
 * Shows Supabase session status and allows manual token entry for WebSocket
 * connection. The session token exchange (automatic token acquisition) will
 * be implemented in Phase 4.
 */

import type { ChangeEvent, FormEvent } from 'react';
import { useCallback, useEffect, useState } from 'react';
import { LazyMotion, domAnimation, m } from 'framer-motion';

import { SPRING_CONFIGS } from '../constants/design-tokens';
import { useAuth } from '../hooks/useAuth';
import { useReducedMotion } from '../hooks/useReducedMotion';

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
 * Form for managing the WebSocket authentication token.
 *
 * Displays Supabase session status and provides token management.
 * When authenticated via Supabase, shows session info. Token management
 * allows saving/clearing the WebSocket connection token.
 *
 * Note: Automatic session token exchange will be added in Phase 4.
 * For now, manual token entry is required for WebSocket connections.
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
  const { user, session } = useAuth();
  const [tokenInput, setTokenInput] = useState<string>('');
  const [status, setStatus] = useState<TokenStatus>(() =>
    hasStoredToken() ? 'saved' : 'not-saved'
  );

  // Respect user's reduced motion preference (FR-008)
  const prefersReducedMotion = useReducedMotion();

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

  // Spring-based hover animations (FR-007)
  const getButtonHoverProps = (isDisabled: boolean) =>
    prefersReducedMotion || isDisabled
      ? undefined
      : {
          scale: 1.02,
          boxShadow: '0 0 12px 2px rgba(217, 119, 87, 0.3)',
          transition: SPRING_CONFIGS.gentle,
        };

  const getButtonTapProps = (isDisabled: boolean) =>
    prefersReducedMotion || isDisabled ? undefined : { scale: 0.98 };

  // Extract user display info
  const userDisplayName =
    user?.user_metadata?.full_name ??
    user?.user_metadata?.name ??
    user?.email ??
    null;

  return (
    <LazyMotion features={domAnimation}>
      <div className={`w-full max-w-md ${className}`}>
        {/* Supabase session info */}
        {session && (
          <div className="mb-4 p-3 bg-[#1a1a1a] border border-[#2a2a2a] rounded-lg">
            <div className="flex items-center gap-2 mb-2">
              <span
                className="h-2 w-2 rounded-full bg-[#4ade80]"
                aria-hidden="true"
              />
              <span className="text-sm font-medium text-[#f5f5f5]">
                Authenticated via GitHub
              </span>
            </div>
            {userDisplayName && (
              <p className="text-xs text-[#a0a0a0]">
                Signed in as {userDisplayName}
              </p>
            )}
          </div>
        )}

        {/* Manual token form */}
        <form onSubmit={handleSave} className="space-y-4">
          <div>
            <label
              htmlFor="token-input"
              className="block text-sm font-medium text-[#f5f5f5] mb-2"
            >
              WebSocket Token
            </label>
            <p className="text-xs text-[#6b6b6b] mb-2">
              {session
                ? 'Token will be auto-managed in a future update. For now, enter manually.'
                : 'Enter your server-issued token for WebSocket connection.'}
            </p>
            <input
              id="token-input"
              type="password"
              value={tokenInput}
              onChange={handleInputChange}
              placeholder={
                isSaved ? 'Enter new token to update' : 'Enter your token'
              }
              autoComplete="off"
              className="w-full px-4 py-2 bg-[#1a1a1a] border border-[#2a2a2a] rounded-lg text-[#f5f5f5] placeholder-[#6b6b6b] hover:border-[#d97757]/50 focus:outline-none focus:ring-2 focus:ring-[#d97757] focus:border-transparent transition-colors"
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
              {isSaved ? 'WebSocket token saved' : 'No WebSocket token saved'}
            </span>
          </div>

          <div className="flex gap-3">
            <m.button
              type="submit"
              disabled={!canSave}
              className="flex-1 px-4 py-2 bg-[#d97757] hover:bg-[#e89a7a] disabled:bg-[#242424] disabled:text-[#6b6b6b] disabled:cursor-not-allowed text-white font-medium rounded-lg focus:outline-none focus:ring-2 focus:ring-[#d97757] focus:ring-offset-2 focus:ring-offset-[#131313]"
              whileHover={getButtonHoverProps(!canSave)}
              whileTap={getButtonTapProps(!canSave)}
            >
              Save Token
            </m.button>
            <m.button
              type="button"
              onClick={handleClear}
              disabled={!isSaved}
              className="px-4 py-2 bg-[#242424] hover:bg-[#2a2a2a] disabled:bg-[#1a1a1a] disabled:text-[#6b6b6b] disabled:cursor-not-allowed text-[#f5f5f5] font-medium rounded-lg focus:outline-none focus:ring-2 focus:ring-[#d97757] focus:ring-offset-2 focus:ring-offset-[#131313]"
              whileHover={getButtonHoverProps(!isSaved)}
              whileTap={getButtonTapProps(!isSaved)}
            >
              Clear
            </m.button>
          </div>
        </form>
      </div>
    </LazyMotion>
  );
}
