# ADR 0004: Use governance documents and scoped agent instructions

Date: 2026-06-14

## Status

Accepted

## Context

The repository will contain a calculation engine, Python bindings, services, UI, contracts, infrastructure, and domain documentation. Humans and AI coding agents need a predictable way to know which files must change when behavior changes.

A single large instruction file would become noisy and technology-specific. A single `DECISION.md` would not scale well once decisions accumulate.

## Decision

Use a small set of root governance files plus scoped documentation:

- `CHANGELOG.md` for user-visible changes.
- `docs/adr/*.md` for durable architecture decisions.
- `CONTRIBUTING.md` for contribution flow and documentation update rules.
- `RELEASE.md` for release checklist and artifacts.
- `SECURITY.md` for security and data-handling policy.
- `GOVERNANCE.md` for review ownership and decision types.
- `docs/standards/engineering-principles.md` for SOLID/functional-style/layering guidance.
- `docs/actuarial/model-governance.md` for actuarial method and assumption governance.
- `contracts/DATA_CONTRACTS.md` for shared logical data schemas.
- Root and nested `AGENTS.md` files plus `.agents/skills/` for AI-agent workflow instructions.

## Consequences

- Contributors and agents have clear places to update after implementation changes.
- Technology-specific instructions remain scoped to the relevant folder or skill.
- Important decisions are indexed and reviewable.
- PRs must explicitly update or mark governance files as not applicable.
