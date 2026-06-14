---
name: actuarial-method
description: Use when implementing or changing reserving methods such as chain ladder, Bornhuetter-Ferguson, Cape Cod, expected loss, tail factors, selections, or blends.
---

Before editing:

1. Read `engines/rustuary-core/AGENTS.md`.
2. Locate existing method, type, and diagnostic structures.
3. Confirm whether the change affects candidate results, selected results, or both.

Implementation rules:

- Keep formulas explicit.
- Return diagnostics for intermediate values.
- Do not silently select assumptions.
- Add or update golden tests.
- Use tolerance-based assertions for floating-point results.
- Preserve separation between calculation and IO.

Completion:

- Run `cargo test -p rustuary-core`.
- Report the affected formulas and diagnostics.
