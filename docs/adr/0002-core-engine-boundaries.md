# ADR 0002: Keep the actuarial core pure

Date: 2026-06-14

## Status

Accepted

## Context

The core engine must be fast, deterministic, testable, and usable from Python, backend services, CLI tools, and future execution environments.

## Decision

The Rust core must not depend on Python, web frameworks, databases, UI frameworks, or object-storage SDKs. It exposes domain types and calculation functions. IO and orchestration live outside the core.

## Consequences

- The core is easier to test with golden fixtures.
- Bindings and services can evolve without rewriting actuarial methods.
- Some convenience code must live in adapters rather than the core.
