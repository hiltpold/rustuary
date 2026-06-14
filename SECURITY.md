# Security policy

This project may eventually process claims, premium, exposure, user, and audit data. Treat all real business data as sensitive by default.

## Reporting a vulnerability

For now, report security issues privately to the repository maintainers. Do not open public issues for vulnerabilities, secrets, PII exposure, or exploitable behavior.

## Data handling rules

- Never commit client data, PII, credentials, tokens, production logs, or secrets.
- Use synthetic examples only under `data/`.
- Store secrets in environment variables or secret managers, never in source code.
- Do not log passwords, tokens, PII, claim identifiers, or full raw payloads.
- Prefer audit-safe identifiers and redacted samples in docs and tests.

## Dependency rules

- Add production dependencies only with a short rationale.
- Prefer mature, maintained libraries.
- Keep dependency changes scoped to the component that needs them.
- Triage security advisories before release.

## Access-control expectations

Backend and UI work must consider:

- Authentication.
- RBAC.
- Tenant/data boundaries.
- Audit logging.
- Safe export behavior.
- Least-privilege service credentials.
