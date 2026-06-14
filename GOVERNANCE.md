# Governance

This document defines lightweight decision and review rules for the project. It should stay practical.

## Decision types

| Decision type | Where recorded | Who should review |
|---|---|---|
| Architecture or stack choice | `docs/adr/*.md` | Architecture/platform owners |
| Actuarial method or result semantics | `docs/actuarial/model-governance.md` and tests | Actuarial method owners |
| Public API or schema change | `contracts/` and changelog | API/platform owners |
| Release process change | `RELEASE.md` | Maintainers |
| Security/data policy change | `SECURITY.md` | Security/data owners |

## Approval principles

- Calculation changes require actuarial review.
- Public interface changes require consumer-impact review.
- Security-sensitive changes require explicit security review.
- Large refactors require a short design note or ADR before implementation.

## AI-agent governance

Agents may propose changes, but humans own final accountability for:

- Actuarial assumptions.
- Production releases.
- Security posture.
- Data governance.
- Breaking public interfaces.
