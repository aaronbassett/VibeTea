/**
 * Persistence feature detection utilities.
 *
 * Provides helpers for detecting whether Supabase persistence is enabled
 * based on environment configuration.
 */

// -----------------------------------------------------------------------------
// Environment Detection
// -----------------------------------------------------------------------------

/**
 * Check if Supabase persistence is enabled.
 *
 * Persistence is considered enabled when VITE_SUPABASE_URL is set to a
 * non-empty string. This controls whether the heatmap component should
 * fetch and display historic data.
 *
 * @returns true if persistence is configured, false otherwise
 *
 * @example
 * ```tsx
 * if (isPersistenceEnabled()) {
 *   // Fetch historic data from Supabase
 * } else {
 *   // Hide heatmap or show real-time only
 * }
 * ```
 */
export function isPersistenceEnabled(): boolean {
  const supabaseUrl = import.meta.env.VITE_SUPABASE_URL as string | undefined;
  return supabaseUrl !== undefined && supabaseUrl !== '';
}

/**
 * Get the configured Supabase URL.
 *
 * Returns the URL if configured, or null if not.
 *
 * @returns Supabase URL string or null
 */
export function getSupabaseUrl(): string | null {
  const supabaseUrl = import.meta.env.VITE_SUPABASE_URL as string | undefined;
  if (supabaseUrl === undefined || supabaseUrl === '') {
    return null;
  }
  return supabaseUrl;
}

/**
 * Check if Supabase auth token is configured.
 *
 * @returns true if token is configured, false otherwise
 */
export function isAuthTokenConfigured(): boolean {
  const token = import.meta.env.VITE_SUPABASE_TOKEN as string | undefined;
  return token !== undefined && token !== '';
}

/**
 * Get the persistence configuration status.
 *
 * Returns a detailed status object indicating what is configured
 * and what is missing.
 *
 * @returns Configuration status object
 */
export interface PersistenceStatus {
  /** Whether persistence is fully enabled (URL + token) */
  readonly enabled: boolean;
  /** Whether the Supabase URL is configured */
  readonly hasUrl: boolean;
  /** Whether the auth token is configured */
  readonly hasToken: boolean;
  /** Human-readable status message */
  readonly message: string;
}

export function getPersistenceStatus(): PersistenceStatus {
  const hasUrl = isPersistenceEnabled();
  const hasToken = isAuthTokenConfigured();
  const enabled = hasUrl && hasToken;

  let message: string;
  if (enabled) {
    message = 'Persistence enabled';
  } else if (!hasUrl && !hasToken) {
    message = 'Persistence not configured';
  } else if (!hasUrl) {
    message = 'Missing VITE_SUPABASE_URL';
  } else {
    message = 'Missing VITE_SUPABASE_TOKEN';
  }

  return { enabled, hasUrl, hasToken, message };
}
