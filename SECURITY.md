# Security Policy

This is a prototype game that turns QR codes into fictional artifacts. It runs offline,
stores no personal data, and makes no network requests at runtime.

## Reporting

Please report security issues via a **private GitHub Security Advisory** (Security → Report a
vulnerability) or a GitHub issue for non-sensitive reports.

## Notes

- Dependency vulnerabilities are checked in CI with `cargo audit`.
- QR content is treated as untrusted input: it is hashed/normalized for artifact generation
  and never executed, and scanned links are **not** auto-opened.
