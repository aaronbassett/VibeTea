## Description

<!-- Provide a brief description of the changes in this PR -->

## Related Issues

<!-- Link any related issues using "Fixes #123" or "Relates to #123" -->

## Type of Change

<!-- Mark the appropriate option with an "x" -->

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] Test improvement

## Component(s) Affected

<!-- Mark all that apply -->

- [ ] Server (`server/`)
- [ ] Monitor (`monitor/`)
- [ ] Client (`client/`)
- [ ] Documentation
- [ ] CI/CD

## Testing

<!-- Describe the testing you've done -->

### Test Commands Run

```bash
# List the test commands you ran
cargo test --workspace -- --test-threads=1
cd client && pnpm test
```

### Manual Testing

<!-- Describe any manual testing performed -->

## Checklist

<!-- Mark completed items with an "x" -->

### Code Quality

- [ ] My code follows the project's style guidelines
- [ ] I have run `cargo fmt` and `cargo clippy` (for Rust changes)
- [ ] I have run `pnpm lint` and `pnpm typecheck` (for client changes)
- [ ] I have added/updated tests for my changes
- [ ] All new and existing tests pass

### Documentation

- [ ] I have updated the documentation accordingly
- [ ] I have added comments to complex or non-obvious code

### Security

- [ ] I have reviewed my changes for security implications
- [ ] Sensitive data is not exposed in logs or error messages
- [ ] Authentication/authorization is properly handled (if applicable)

### Privacy

<!-- VibeTea has strict privacy requirements for the Monitor -->

- [ ] No code, prompts, or file contents are transmitted (if Monitor changes)
- [ ] Only structural metadata is exposed (event types, timestamps, tool categories)

## Screenshots

<!-- If applicable, add screenshots to demonstrate the changes -->

## Additional Notes

<!-- Any additional information that reviewers should know -->
