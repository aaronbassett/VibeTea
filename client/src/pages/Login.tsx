/**
 * Login Page Component
 *
 * Provides GitHub OAuth authentication via Supabase for VibeTea.
 * Implements FR-001 (redirect unauthenticated users) and FR-002 (GitHub OAuth).
 *
 * @see spec.md for full requirements (SC-001: auth under 30 seconds)
 * @module pages/Login
 */

import { LazyMotion, domAnimation, m } from 'framer-motion';
import { useCallback, useState } from 'react';

import { AnimatedBackground } from '../components/animated/AnimatedBackground';
import { ASCIIHeader } from '../components/animated/ASCIIHeader';
import { AnimationErrorBoundary } from '../components/animated/ErrorBoundary';
import { SpringContainer } from '../components/animated/SpringContainer';
import { COLORS, SPRING_CONFIGS } from '../constants/design-tokens';
import { useAuth } from '../hooks/useAuth';
import { useReducedMotion } from '../hooks/useReducedMotion';

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Props for the Login page component.
 */
export interface LoginProps {
  /** Optional callback when authentication succeeds (for routing) */
  readonly onAuthSuccess?: () => void;
}

// -----------------------------------------------------------------------------
// Component
// -----------------------------------------------------------------------------

/**
 * Login page for GitHub OAuth authentication.
 *
 * Displays the VibeTea ASCII header with animated background and a "Sign in with GitHub"
 * button. Respects user's reduced motion preference and shows appropriate loading
 * and error states.
 *
 * @example
 * ```tsx
 * // Basic usage
 * <Login />
 *
 * // With auth success callback
 * <Login onAuthSuccess={() => navigate('/dashboard')} />
 * ```
 */
export function Login({ onAuthSuccess }: LoginProps) {
  const { loading, signInWithGitHub } = useAuth();
  const prefersReducedMotion = useReducedMotion();

  // Local state for sign-in process
  const [isSigningIn, setIsSigningIn] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Handle GitHub sign-in button click.
   * Initiates OAuth flow and manages loading/error states.
   */
  const handleSignIn = useCallback(async () => {
    setError(null);
    setIsSigningIn(true);

    try {
      await signInWithGitHub();
      onAuthSuccess?.();
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to sign in with GitHub';
      setError(message);
    } finally {
      setIsSigningIn(false);
    }
  }, [signInWithGitHub, onAuthSuccess]);

  // Spring-based hover animations (consistent with TokenForm)
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

  const isButtonDisabled = loading || isSigningIn;

  // Show loading state while checking initial auth
  if (loading) {
    return (
      <LazyMotion features={domAnimation}>
        <div
          className="min-h-screen bg-[#131313] text-[#f5f5f5] flex flex-col items-center justify-center p-8 relative"
          role="status"
          aria-live="polite"
          aria-label="Checking authentication status"
        >
          {/* Animated background layer with error boundary */}
          <AnimationErrorBoundary>
            <AnimatedBackground showGrid showParticles />
          </AnimationErrorBoundary>

          {/* Loading indicator */}
          <div className="relative z-10 text-center">
            <div
              className="inline-block w-8 h-8 border-2 border-[#d97757] border-t-transparent rounded-full animate-spin"
              aria-hidden="true"
            />
            <p className="mt-4 text-[#a0a0a0]">Checking authentication...</p>
          </div>
        </div>
      </LazyMotion>
    );
  }

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
                Sign in with GitHub to access the VibeTea dashboard.
              </p>
            </SpringContainer>
          </div>

          {/* Sign in button with spring entrance */}
          <SpringContainer springType="standard" delay={0.4}>
            <div className="space-y-4">
              <m.button
                type="button"
                onClick={handleSignIn}
                disabled={isButtonDisabled}
                className="w-full px-4 py-3 bg-[#d97757] hover:bg-[#e89a7a] disabled:bg-[#242424] disabled:text-[#6b6b6b] disabled:cursor-not-allowed text-white font-medium rounded-lg focus:outline-none focus:ring-2 focus:ring-[#d97757] focus:ring-offset-2 focus:ring-offset-[#131313] flex items-center justify-center gap-3"
                whileHover={getButtonHoverProps(isButtonDisabled)}
                whileTap={getButtonTapProps(isButtonDisabled)}
                aria-describedby={error ? 'login-error' : undefined}
              >
                {/* GitHub icon */}
                <svg
                  className="w-5 h-5"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                  aria-hidden="true"
                >
                  <path
                    fillRule="evenodd"
                    d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                    clipRule="evenodd"
                  />
                </svg>
                {isSigningIn ? 'Signing in...' : 'Sign in with GitHub'}
              </m.button>

              {/* Error message */}
              {error && (
                <m.div
                  id="login-error"
                  role="alert"
                  aria-live="assertive"
                  initial={
                    prefersReducedMotion ? undefined : { opacity: 0, y: -10 }
                  }
                  animate={{ opacity: 1, y: 0 }}
                  transition={SPRING_CONFIGS.gentle}
                  className="text-center p-3 bg-[#ef4444]/10 border border-[#ef4444]/30 rounded-lg"
                >
                  <p className="text-sm" style={{ color: COLORS.status.error }}>
                    {error}
                  </p>
                </m.div>
              )}
            </div>
          </SpringContainer>

          {/* Footer text */}
          <SpringContainer springType="gentle" delay={0.6}>
            <p className="text-center text-sm text-[#6b6b6b]">
              Any GitHub account can access the dashboard.
            </p>
          </SpringContainer>
        </div>
      </div>
    </LazyMotion>
  );
}

export default Login;
