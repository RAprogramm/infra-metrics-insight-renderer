<!--
SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>

SPDX-License-Identifier: MIT
-->

# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in IMIR, please report it by emailing **andrey.rozanov.vl@gmail.com**.

**Please do not open a public issue for security vulnerabilities.**

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if available)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity (critical: 7 days, high: 14 days, medium: 30 days)

## Security Measures

IMIR implements the following security practices:

- **Dependency Auditing**: Automated security audits via `cargo audit` in CI
- **License Compliance**: REUSE specification for clear licensing
- **Supply Chain**: Dependabot for automated dependency updates
- **Code Quality**: Clippy with `-D warnings` enforces security best practices
- **Memory Safety**: Rust's memory safety guarantees prevent common vulnerabilities

## Disclosure Policy

Once a vulnerability is fixed:

1. Security advisory published on GitHub
2. CVE requested if applicable
3. Notification sent to users via GitHub release notes
4. Credit given to reporter (if desired)
