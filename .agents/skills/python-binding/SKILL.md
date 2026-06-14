---
name: python-binding
description: Use when adding or changing Python APIs, PyO3 bindings, maturin packaging, pandas/pyarrow adapters, or notebook-facing actuarial workflows.
---

Before editing:

1. Read `bindings/python/AGENTS.md`.
2. Keep the public Python API friendly and stable.
3. Keep `_rust` low-level details hidden from business users.

Design rules:

Column mapping rules:

- Accept pandas, Polars, and PyArrow inputs through a dataframe adapter.
- Normalize user columns into canonical contracts before calling Rust.
- Prefer `Triangle.from_frame(..., origin=..., development=..., value=...)` for simple use and `ClaimsMapping` for reusable workflows.
- Persist or return mapping metadata for reproducible model runs.
- Report validation errors using both source column names and canonical field names.


- Use `uv` and a local `.venv` for Python development.
- Rebuild PyO3 bindings with `maturin develop` after Rust binding changes.
- Type public Python APIs and avoid mutable default arguments.

- Python expresses actuarial intent; Rust executes actuarial math.
- Prefer named parameters over positional arguments.
- Return rich result objects, not naked arrays.
- Expose `.summary()`, `.diagnostics()`, `.to_pandas()`, and `.to_arrow()` where applicable.
- Do not make pandas a hard dependency of the Rust core.

Completion:

- Add a minimal Python example or doctest-style snippet.
- Explain any API compatibility impact.
