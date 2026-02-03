# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of VibeTea seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please send an email to: **aaronbassett@gmail.com**

Include the following information in your report:

- Type of vulnerability (e.g., authentication bypass, information disclosure, injection)
- Full paths of source file(s) related to the vulnerability
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact assessment of the vulnerability

### What to Expect

- **Acknowledgment:** You will receive an acknowledgment of your report within 48 hours
- **Updates:** We will provide updates on the status of your report at least every 7 days
- **Resolution:** We aim to resolve critical vulnerabilities within 30 days
- **Credit:** If you wish, we will credit you in the security advisory and release notes

### Scope

The following are in scope for security reports:

- VibeTea Server (authentication, authorization, WebSocket handling)
- VibeTea Monitor (data handling, privacy controls)
- VibeTea Client (XSS, CSRF, sensitive data exposure)
- Dependencies with known vulnerabilities affecting VibeTea

### Out of Scope

- Issues in third-party services
- Social engineering attacks
- Physical attacks
- Denial of service attacks (unless they reveal a deeper vulnerability)

## Security Best Practices

### For Operators

1. **Use TLS:** Always deploy the server behind HTTPS/WSS
2. **Strong Tokens:** Use cryptographically secure tokens for `VIBETEA_AUTH_TOKEN`
3. **Network Security:** Restrict access to the server to trusted networks where possible
4. **Keep Updated:** Regularly update to the latest version

### For Developers

When contributing to VibeTea, please follow these security practices:

1. **Token Comparison:** Use constant-time comparison (`subtle::ConstantTimeEq`) for authentication tokens to prevent timing attacks
2. **Signature Verification:** Use `verify_strict()` for RFC 8032 compliant Ed25519 signature verification
3. **Input Validation:** Validate and sanitize all user input
4. **Error Messages:** Avoid exposing sensitive information in error messages
5. **Dependencies:** Keep dependencies updated and audit for known vulnerabilities

## Privacy Commitment

VibeTea is designed with privacy as a core principle. The Monitor component implements strict privacy controls to ensure:

- No code, prompts, or file contents are ever transmitted
- Only structural metadata (event types, timestamps, tool categories) is shared
- Full file paths are stripped to basenames only

See the [README](README.md#privacy) for full privacy details.

## Security Updates

Security updates will be released as:

1. Patch releases for the affected version(s)
2. Security advisories on the GitHub repository
3. Announcements in release notes

## Acknowledgments

We thank the security researchers and community members who help keep VibeTea secure through responsible disclosure.
