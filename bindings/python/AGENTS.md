# Python binding agent instructions

This folder exposes the Rust actuarial engine to actuaries and analysts.

## Rules

- Keep the public Python API business-friendly.
- Hide `_rust` implementation details.
- Prefer named parameters and rich result objects.
- Python can depend on pandas/pyarrow/polars adapters; the Rust core must not.
- Provide notebook-friendly summaries and diagnostics.
- Do not duplicate actuarial formulas in Python unless it is a display/validation helper.

## PyO3 and Python tooling

- Use `uv` and a local `.venv` for Python development. Do not rely on the system Python.
- Keep `.venv` ignored.
- Use PyO3/maturin for compiled Rust bindings.
- Rebuild with `uv run maturin develop` after Rust binding changes.
- Use type hints for public Python functions and avoid mutable default arguments.
- Prefer `T | None` for nullable values.
- Run pytest and type checks when Python API code changes.

## Commands

```bash
uv sync --extra dev
uv run pytest
```

When PyO3 is implemented:

```bash
uv run maturin develop
uv run pytest
```
