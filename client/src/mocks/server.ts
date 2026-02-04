/**
 * MSW server setup for Vitest integration.
 *
 * Provides a pre-configured MSW server instance for use in tests.
 * Import and use with Vitest's beforeAll/afterAll hooks.
 *
 * @example
 * ```ts
 * import { server } from '../mocks/server';
 *
 * beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
 * afterEach(() => server.resetHandlers());
 * afterAll(() => server.close());
 * ```
 */

import { setupServer } from 'msw/node';
import { queryHandlers } from './handlers';

/**
 * MSW server instance configured with all VibeTea handlers.
 *
 * Use this server in test files to intercept HTTP requests
 * and return mock responses.
 */
export const server = setupServer(...queryHandlers);
