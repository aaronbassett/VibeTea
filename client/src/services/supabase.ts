import { createClient, Session } from '@supabase/supabase-js';

/**
 * Supabase URL from environment variables.
 * Required for all Supabase operations.
 */
const supabaseUrl = import.meta.env.VITE_SUPABASE_URL;

/**
 * Supabase anonymous key from environment variables.
 * Used for client-side authentication.
 */
const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY;

if (!supabaseUrl || !supabaseAnonKey) {
  throw new Error(
    'Missing Supabase environment variables. ' +
    'Please set VITE_SUPABASE_URL and VITE_SUPABASE_ANON_KEY in your .env file.'
  );
}

/**
 * Supabase client instance configured with project credentials.
 * Use this client for all Supabase operations including authentication.
 */
export const supabase = createClient(supabaseUrl, supabaseAnonKey);

/**
 * Retrieves the current user session from Supabase.
 * Returns null if no active session exists.
 *
 * @returns The current session or null if not authenticated
 */
export async function getSession(): Promise<Session | null> {
  const { data, error } = await supabase.auth.getSession();

  if (error) {
    console.error('Failed to get session:', error.message);
    return null;
  }

  return data.session;
}
