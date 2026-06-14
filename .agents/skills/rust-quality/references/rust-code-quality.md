# Rust code-quality reference

This reference distills a Rust-specific Claude/Codex instruction file into guidance that fits the Rustuary monorepo. The source document was intentionally not copied verbatim because some rules are useful only for Rust crates, while others conflict with the planned Go backend and SvelteKit frontend.

## Keep from the source

- Use `cargo` for Rust project management, builds, tests, and dependency handling.
- Use `serde`/`serde_json` for structured config and JSON where needed.
- Use `thiserror` for library errors and `anyhow` for application-level errors.
- Use `tracing`/`log` in app crates instead of committed debug `println!` or `dbg!` output.
- Use PyO3 and `maturin` for Python bindings; rebuild the Python extension after Rust binding changes.
- Use `uv` and a local `.venv` for Python development in `bindings/python`.
- Use meaningful names, idiomatic Rust naming, rustfmt, clippy, and clear module boundaries.
- Document public Rust items.
- Avoid `.unwrap()` in library or production code.
- Prefer newtypes over plain primitives for semantically different quantities.
- Prefer `Option<T>` over sentinel values.
- Keep functions focused; use builders/config structs for complex construction.
- Derive common traits such as `Debug`, `Clone`, and `PartialEq` where appropriate.
- Use `Vec::with_capacity()` where size is known and allocation matters.
- Use `rayon` for CPU-bound parallelism when it improves real workloads.
- Do not commit secrets, commented-out code, debug macros, or credentials.

## Adapt for Rustuary

- Do not require every change to be "fully optimized" before handoff. Rustuary prioritizes correctness, auditability, and benchmark-driven optimization.
- Do not add Polars to the pure Rust core. Polars can be used in Python/data adapters, not in `engines/rustuary-core`.
- Do not adopt Axum as a default because the planned backend is Go. Keep Axum guidance only for future Rust service experiments.
- Do not copy frontend rules that ban component frameworks. Rustuary currently targets SvelteKit for the workbench.
- Do not require WASM rebuilds unless a future Rust/WASM crate actually exists.

## Rust review checklist

- Does the change alter actuarial results? If yes, update golden tests and diagnostics.
- Are all assumptions explicit and serializable where relevant?
- Are fallible paths represented as typed errors?
- Are public APIs documented?
- Are floats compared with tolerances?
- Are allocations and clones intentional?
- Are dependencies justified and scoped to the right crate?
- Were relevant commands run, and were failures reported exactly?
