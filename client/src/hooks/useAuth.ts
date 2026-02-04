/**
 * Authentication hook for VibeTea client.
 *
 * Provides GitHub OAuth authentication via Supabase, managing user sessions
 * and auth state changes. Implements FR-001 through FR-004 from spec.md.
 *
 * @see https://supabase.com/docs/guides/auth/social-login/auth-github
 */

import { useCallback, useEffect, useState } from 'react';
import type { Session, User } from '@supabase/supabase-js';

import { supabase } from '../services/supabase';

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Return type for the useAuth hook.
 */
export interface UseAuthReturn {
  /** Current authenticated user (null if not authenticated) */
  readonly user: User | null;
  /** Current Supabase session (null if not authenticated) */
  readonly session: Session | null;
  /** Loading state during initial auth check */
  readonly loading: boolean;
  /** Initiates GitHub OAuth flow */
  readonly signInWithGitHub: () => Promise<void>;
  /** Signs out the current user */
  readonly signOut: () => Promise<void>;
}

// -----------------------------------------------------------------------------
// Hook Implementation
// -----------------------------------------------------------------------------

/**
 * Authentication hook for GitHub OAuth via Supabase.
 *
 * Manages authentication state including user, session, and loading states.
 * Automatically subscribes to auth state changes and handles session persistence
 * through Supabase's built-in session management (FR-004).
 *
 * @returns Object with user, session, loading state, and auth methods
 *
 * @example
 * ```tsx
 * function App() {
 *   const { user, session, loading, signInWithGitHub, signOut } = useAuth();
 *
 *   if (loading) {
 *     return <div>Loading...</div>;
 *   }
 *
 *   if (!user) {
 *     return <button onClick={signInWithGitHub}>Sign in with GitHub</button>;
 *   }
 *
 *   return (
 *     <div>
 *       <p>Welcome, {user.email}</p>
 *       <button onClick={signOut}>Sign out</button>
 *     </div>
 *   );
 * }
 * ```
 */
export function useAuth(): UseAuthReturn {
  const [user, setUser] = useState<User | null>(null);
  const [session, setSession] = useState<Session | null>(null);
  const [loading, setLoading] = useState<boolean>(true);

  // Subscribe to auth state changes
  useEffect(() => {
    // Get initial session
    const initializeAuth = async (): Promise<void> => {
      try {
        const { data, error } = await supabase.auth.getSession();

        if (error) {
          console.error('[useAuth] Failed to get session:', error.message);
          setUser(null);
          setSession(null);
        } else {
          setSession(data.session);
          setUser(data.session?.user ?? null);
        }
      } catch (err) {
        console.error('[useAuth] Unexpected error during initialization:', err);
        setUser(null);
        setSession(null);
      } finally {
        setLoading(false);
      }
    };

    // Set up auth state listener
    const {
      data: { subscription },
    } = supabase.auth.onAuthStateChange((_event, newSession) => {
      setSession(newSession);
      setUser(newSession?.user ?? null);
      setLoading(false);
    });

    // Initialize auth state
    void initializeAuth();

    // Cleanup subscription on unmount
    return () => {
      subscription.unsubscribe();
    };
  }, []);

  /**
   * Initiate GitHub OAuth sign-in flow.
   *
   * Redirects the user to GitHub for authentication (FR-002).
   * Any authenticated GitHub user can access the dashboard (FR-003).
   */
  const signInWithGitHub = useCallback(async (): Promise<void> => {
    try {
      const { error } = await supabase.auth.signInWithOAuth({
        provider: 'github',
      });

      if (error) {
        console.error('[useAuth] GitHub sign-in failed:', error.message);
        throw error;
      }
    } catch (err) {
      console.error('[useAuth] Unexpected error during sign-in:', err);
      throw err;
    }
  }, []);

  /**
   * Sign out the current user.
   *
   * Clears the session from Supabase and local state.
   */
  const signOut = useCallback(async (): Promise<void> => {
    try {
      const { error } = await supabase.auth.signOut();

      if (error) {
        console.error('[useAuth] Sign-out failed:', error.message);
        throw error;
      }

      // State will be updated by onAuthStateChange listener
    } catch (err) {
      console.error('[useAuth] Unexpected error during sign-out:', err);
      throw err;
    }
  }, []);

  return {
    user,
    session,
    loading,
    signInWithGitHub,
    signOut,
  };
}
