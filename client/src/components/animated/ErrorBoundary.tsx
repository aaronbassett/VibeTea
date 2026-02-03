/**
 * AnimationErrorBoundary - Error boundary for animated components (FR-013)
 *
 * Catches errors in animated component trees and renders a static fallback,
 * ensuring the application remains functional even when animation errors occur.
 *
 * @module components/animated/ErrorBoundary
 */

import React, { Component, ErrorInfo, ReactNode } from 'react';

import { COLORS } from '../../constants/design-tokens';

/**
 * Props for the AnimationErrorBoundary component
 */
interface AnimationErrorBoundaryProps {
  /** Child components to render (typically animated components) */
  children: ReactNode;
  /** Optional custom fallback UI to render when an error occurs */
  fallback?: ReactNode;
  /** Optional callback invoked when an error is caught */
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

/**
 * Internal state for the AnimationErrorBoundary
 */
interface AnimationErrorBoundaryState {
  /** Whether an error has been caught */
  hasError: boolean;
  /** The error that was caught, if any */
  error: Error | null;
}

/**
 * Default fallback styles matching design tokens
 */
const defaultFallbackStyles: React.CSSProperties = {
  backgroundColor: COLORS.background.secondary,
  color: COLORS.text.muted,
  padding: '1rem',
  borderRadius: '0.5rem',
  fontFamily: 'monospace',
  fontSize: '0.875rem',
  textAlign: 'center',
};

/**
 * Error boundary component for catching and handling errors in animated components.
 *
 * This component implements the React error boundary pattern using a class component
 * (required by React for error boundaries). When an error occurs in any child component,
 * it catches the error, logs it for debugging, and renders a static fallback UI.
 *
 * @example
 * ```tsx
 * // Basic usage with default fallback
 * <AnimationErrorBoundary>
 *   <AnimatedComponent />
 * </AnimationErrorBoundary>
 *
 * // With custom fallback
 * <AnimationErrorBoundary fallback={<StaticVersion />}>
 *   <AnimatedComponent />
 * </AnimationErrorBoundary>
 *
 * // With error callback
 * <AnimationErrorBoundary onError={(error) => logToService(error)}>
 *   <AnimatedComponent />
 * </AnimationErrorBoundary>
 * ```
 */
export class AnimationErrorBoundary extends Component<
  AnimationErrorBoundaryProps,
  AnimationErrorBoundaryState
> {
  /**
   * Initialize state with no error
   */
  state: AnimationErrorBoundaryState = {
    hasError: false,
    error: null,
  };

  /**
   * Static lifecycle method called when an error is thrown in a descendant component.
   * Updates state to trigger fallback UI rendering.
   *
   * @param error - The error that was thrown
   * @returns Updated state with error information
   */
  static getDerivedStateFromError(error: Error): AnimationErrorBoundaryState {
    return {
      hasError: true,
      error,
    };
  }

  /**
   * Lifecycle method called after an error has been thrown by a descendant component.
   * Used for error logging and reporting.
   *
   * @param error - The error that was thrown
   * @param errorInfo - Object containing information about which component threw the error
   */
  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    // Log error details for debugging
    console.error('[AnimationErrorBoundary] Animation error caught:', {
      error: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
    });

    // Invoke optional error callback if provided
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
  }

  /**
   * Renders the default fallback UI when no custom fallback is provided.
   *
   * @returns A styled div indicating animation is unavailable
   */
  private renderDefaultFallback(): ReactNode {
    return (
      <div style={defaultFallbackStyles} role="alert" aria-live="polite">
        Animation unavailable
      </div>
    );
  }

  /**
   * Renders either the children or the fallback UI based on error state.
   *
   * @returns The children if no error, otherwise the fallback UI
   */
  render(): ReactNode {
    const { hasError } = this.state;
    const { children, fallback } = this.props;

    if (hasError) {
      return fallback ?? this.renderDefaultFallback();
    }

    return children;
  }
}

export default AnimationErrorBoundary;
