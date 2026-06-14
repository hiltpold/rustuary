# ADR 0001: Use a monorepo

Date: 2026-06-14

## Status

Accepted

## Context

The product will include a Rust actuarial engine, Python bindings, backend services, UI, contracts, infrastructure, and documentation. These parts must evolve together, especially while APIs and actuarial workflows are still unstable.

## Decision

Use a monorepo with clear top-level ownership boundaries:

- `engines/` for calculation engines
- `bindings/` for language bindings
- `services/` for backend services
- `apps/` for user-facing applications
- `contracts/` for shared contracts and schemas
- `docs/` for architecture and standards
- `.agents/` for coding-agent skills

## Consequences

- Cross-cutting changes are easier to coordinate.
- CI can validate engine, backend, and contracts in one place.
- Repository guidance for AI agents can be centralized and scoped by directory.
- The repo must aggressively avoid accidental coupling between layers.
