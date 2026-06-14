---
name: rust-quality
description: Use when editing Rust crates, Rust APIs, PyO3 Rust modules, performance-sensitive calculations, or Rust tests.
---

Before editing Rust:

1. Read the nearest `AGENTS.md`.
2. Identify whether the code is library code, app code, Python-binding code, or experimental code.
3. Check existing tests and public API contracts before changing behavior.

Rust rules:

- Prioritize correctness, auditability, and simple data flow before optimization.
- Optimize deliberately: avoid obvious waste, but use benchmarks before adding SIMD, unsafe code, or complex parallelism.
- Use domain types/newtypes for actuarial concepts such as origin periods, development ages, factors, CDFs, ELRs, weights, and amounts.
- Public fallible APIs must return `Result<T, E>` with meaningful custom errors, preferably using `thiserror` in libraries.
- Do not use `.unwrap()` in library or production code. Use `.expect("...")` only for proven invariants with a specific message.
- Prefer borrowing over ownership. Avoid unnecessary allocation and cloning.
- Keep functions focused. Use config structs/builders when arguments would become hard to read.
- Keep public structs/enums/functions documented with Rust doc comments.
- Do not use `unsafe` unless explicitly justified, isolated, tested, and documented with safety invariants.
- Use `rayon` for CPU-bound parallel batch calculations when benchmarked or clearly beneficial.
- Use `tokio` only in async application crates, not in the pure actuarial core.
- Do not add dataframe, HTTP, database, Python, or UI dependencies to `engines/rustuary-core`.

Testing and verification:

- Add unit tests near new method code and golden/integration tests for actuarial outputs.
- Use tolerance-based assertions for floating-point values.
- Run the narrowest relevant checks first:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If only the core changed, prefer:

```bash
cargo fmt --all -- --check
cargo clippy -p rustuary-core --all-targets -- -D warnings
cargo test -p rustuary-core
```

See `references/rust-code-quality.md` for the longer rationale and optional guidance.
