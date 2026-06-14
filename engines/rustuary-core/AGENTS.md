# Engine-specific agent instructions

This crate is the pure actuarial calculation core.

## Hard boundaries

- Do not add Python, pandas, Polars, HTTP, database, object-store, or UI dependencies here.
- Do not read files inside core calculations.
- Keep IO in adapters outside the core.
- Keep calculations deterministic.

## Coding conventions

- Use domain types for origin period, development age, factors, and results.
- Make invalid triangles explicit errors.
- Keep candidate method results separate from selected results.
- Add diagnostics for all intermediate actuarial quantities that a reviewer may need.
- Prefer clear formulas over generic abstractions.

## Rust quality rules

- Also follow `.agents/skills/rust-quality/SKILL.md` for Rust edits.
- Use `thiserror` for library error types.
- Do not use `.unwrap()` in library code; use typed errors or `expect` only for named invariants.
- Document public Rust items with doc comments.
- Prefer newtypes for actuarial quantities that share primitive representations.
- Avoid unnecessary allocation and cloning, but benchmark before adding complex optimization.
- Do not use `unsafe` without an explicit design note and safety invariants.

## Tests

- Add unit tests near the method implementation.
- Add integration/golden tests for method outputs.
- Use absolute or relative tolerances for floats.

## Commands

```bash
cargo fmt --all -- --check
cargo test -p rustuary-core
```
