# Implement actuarial method

Use this prompt for Codex:

Implement the requested actuarial method in `engines/rustuary-core`.

Constraints:

- Read root `AGENTS.md` and `engines/rustuary-core/AGENTS.md` first.
- Keep the core independent from Python, services, and UI.
- Expose intermediate diagnostics.
- Add tests with synthetic data.
- Run `cargo test -p rustuary-core`.
- Summarize formulas, assumptions, diagnostics, and checks run.
