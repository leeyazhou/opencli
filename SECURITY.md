# Security Policy

## Supported Versions

Security fixes are intended for the latest development state and the latest tagged release.

## Reporting a Vulnerability

Do not open public GitHub issues for sensitive vulnerabilities.

Report security issues privately to the maintainers with:

- a clear description of the issue
- affected commands or modules
- reproduction steps if available
- potential impact
- suggested mitigation if known

When reporting issues related to provider auth, shell execution, path traversal, audit data, or approval bypasses, include the relevant config values with secrets redacted.

## Areas of Special Concern

- Shell command approval and execution
- Workspace path restriction enforcement
- Tool policy bypasses
- Provider credential handling
- Audit log leakage of sensitive data

## Disclosure Process

1. Maintainers acknowledge receipt.
2. Issue is reproduced and impact assessed.
3. Fix is prepared and validated.
4. Release notes document the security fix when appropriate.
