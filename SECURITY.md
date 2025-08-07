# Security Policy

We take security seriously. If you discover a vulnerability in this project, please follow the steps below.

## Reporting a Vulnerability

- Do NOT open a public issue. Instead, email: security@yourdomain.example
- Provide a detailed description, steps to reproduce, and potential impact.
- We will acknowledge your report within 48 hours and provide a timeline for a fix.

## Supported Versions

We aim to support the latest main branch. Security fixes will be backported on a best-effort basis.

## Best Practices Followed

- PDA-based account derivations
- Strict signer and ownership checks
- Gateway caller validation using instruction sysvar
- Clear error codes and messages
- Minimal account data and rent-exempt sizing
- Continuous Integration (CI) with clippy and rustfmt

## Scope

- On-chain program code in `programs/universal-nft`
- Tests and scripts in `tests` and `scripts`

## Out of Scope

- Wallet integrations and third-party dependencies
- Off-chain services and infrastructure

## Coordinated Disclosure

We prefer responsible disclosure and will credit researchers upon request once the issue is fixed.

