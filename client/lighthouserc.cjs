/**
 * Lighthouse CI Configuration
 *
 * Configures automated Lighthouse audits for CI pipeline.
 * Asserts performance budgets and quality gates.
 *
 * @see https://github.com/GoogleChrome/lighthouse-ci
 */

module.exports = {
  ci: {
    collect: {
      // Serve static files from the built dist directory
      staticDistDir: './dist',
      // Number of runs to perform (averaged for more stable results)
      numberOfRuns: 1,
      // URL paths to audit (relative to staticDistDir)
      url: ['http://localhost/'],
    },
    assert: {
      // Assertions for performance budgets
      assertions: {
        // Performance score must be >= 60 (allows for CI environment variability)
        // Note: Login page with animated background scores lower in CI
        'categories:performance': ['error', { minScore: 0.60 }],
        // Accessibility score must be >= 80
        'categories:accessibility': ['error', { minScore: 0.8 }],
        // Best practices score must be >= 80
        'categories:best-practices': ['error', { minScore: 0.8 }],
        // SEO score must be >= 80
        'categories:seo': ['error', { minScore: 0.8 }],
      },
    },
    upload: {
      // Don't upload results (local only for now)
      target: 'temporary-public-storage',
    },
  },
};
