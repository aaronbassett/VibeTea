/**
 * VibeTea Application Root - Authentication routing.
 *
 * Routes users to either the Login or Dashboard page based on Supabase
 * authentication state. Implements FR-001 (redirect unauthenticated users
 * to login page).
 */

import { LazyMotion, domAnimation } from 'framer-motion';

import { AnimatedBackground } from './components/animated/AnimatedBackground';
import { AnimationErrorBoundary } from './components/animated/ErrorBoundary';
import { useAuth } from './hooks/useAuth';
import { useReducedMotion } from './hooks/useReducedMotion';
import { Dashboard } from './pages/Dashboard';
import { Login } from './pages/Login';

// -----------------------------------------------------------------------------
// Main Component
// -----------------------------------------------------------------------------

/**
 * Application root component with authentication routing.
 *
 * Displays the appropriate page based on Supabase auth state:
 * - Loading state during initial auth check
 * - Login page when not authenticated (FR-001)
 * - Dashboard page when authenticated
 */
export default function App() {
  const { user, loading } = useAuth();
  const prefersReducedMotion = useReducedMotion();

  // Show loading state during initial auth check
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
          {!prefersReducedMotion && (
            <AnimationErrorBoundary>
              <AnimatedBackground showGrid showParticles />
            </AnimationErrorBoundary>
          )}

          {/* Loading indicator */}
          <div className="relative z-10 text-center">
            <div
              className="inline-block w-8 h-8 border-2 border-[#d97757] border-t-transparent rounded-full animate-spin"
              aria-hidden="true"
            />
            <p className="mt-4 text-[#a0a0a0]">Loading...</p>
          </div>
        </div>
      </LazyMotion>
    );
  }

  // Show login page if not authenticated (FR-001)
  if (!user) {
    return <Login />;
  }

  // Show dashboard for authenticated users
  return <Dashboard />;
}
