# AGENTS.md

## Purpose

This file is the repository-level operating manual for AI coding agents. Keep it concise, accurate, and enforceable. Add rules only after repeated mistakes or recurring review feedback.

## Repository shape

- `engines/rustuary-core/`: pure Rust actuarial engine. No Python, web, DB, or UI dependencies.
- `bindings/python/`: PyO3/maturin Python package that wraps the Rust engine and provides business-friendly APIs.
- `services/api/`: Go API service for auth, RBAC, workflow, audit, metadata, exports.
- `services/worker/`: job orchestration and background execution.
- `apps/workbench/`: SvelteKit reserving workbench.
- `contracts/`: OpenAPI, protobuf, and logical schemas.
- `data/`: tiny synthetic examples and golden fixtures only. Never commit client data.
- `docs/`: architecture, ADRs, standards, runbooks, actuarial governance, and product notes.

## Working agreements

- Read before editing. Inspect the nearest README/AGENTS.md and relevant tests first.
- Make the smallest useful change. Do not refactor unrelated code, rename files, reformat whole directories, or “clean up” nearby code unless asked.
- Prefer boring, typed, deterministic code over clever abstractions.
- Prefer pure functions, immutable inputs, and explicit outputs for calculation code. Use SOLID principles as heuristics, not ceremony.
- Keep actuarial formulas explicit and auditable. Store intermediate values when they explain a result.
- Do not hide assumptions. If a method uses selected factors, tail factors, ELRs, weights, exclusions, or overrides, expose them in diagnostics.
- Never invent financial, actuarial, legal, or regulatory facts. Add a TODO or ask for a source when domain assumptions are missing.
- Do not commit secrets, credentials, production data, or personally identifiable information.
- Avoid adding production dependencies without a short rationale in the PR description.
- When behavior changes, update the corresponding changelog, ADR, contract, governance doc, example, or runbook. If no doc update is needed, state why in the PR.


## Design style

- Follow `docs/standards/engineering-principles.md`.
- Favor functional-style data transformations in the actuarial core: validate input, calculate diagnostics, return explicit results.
- Use interfaces/traits/ports to isolate IO, database, HTTP, object storage, and UI concerns from domain logic.
- Do not force object-oriented patterns when plain functions, enums, traits, or data types are clearer.

## Scoped instructions

- Keep root guidance technology-neutral. Put language/framework-specific rules in nested `AGENTS.md` files or `.agents/skills/`.
- For Rust work, use `.agents/skills/rust-quality/SKILL.md` and the nearest Rust crate `AGENTS.md`.
- For Python binding work, use `.agents/skills/python-binding/SKILL.md` and `bindings/python/AGENTS.md`.
- For Go service work, use `.agents/skills/go-service/SKILL.md` and the nearest service `AGENTS.md`.
- For SvelteKit UI work, use `.agents/skills/sveltekit-ui/SKILL.md`, `apps/workbench/AGENTS.md`, and the Svelte MCP server when available.
- Before finalizing implementation changes, use `.agents/skills/change-hygiene/SKILL.md`.
- MCP setup examples live in `.codex/config.example.toml`, `.mcp/`, and `docs/ai/mcp-setup.md`.
- For longer rationale, prefer skill `references/` files or `docs/ai/`; do not turn root `AGENTS.md` into a full style guide.
- For non-trivial implementation tasks, start from `docs/ai/task-brief-template.md` and finish from `docs/ai/implementation-report-template.md`.
- Tool versions are pinned in `rust-toolchain.toml`, `.mise.toml`, `.tool-versions`, `.node-version`, and `.python-version`.
- `CLAUDE.md` is a compatibility pointer only. `AGENTS.md` remains the source of truth.

## Verification commands

Run the narrowest relevant checks first, then broader checks if time allows.

```bash
# Whole repo smoke check
./scripts/check.sh

# Governance/doc hygiene only
./scripts/check_repo_hygiene.sh

# Rust core
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Go API
(cd services/api && test -z "$(gofmt -l .)" && go test ./... && go vet ./...)
# Optional if installed
(cd services/api && golangci-lint run)
(cd services/api && govulncheck ./...)

# UI placeholder, once dependencies are installed
(cd apps/workbench && pnpm check && pnpm lint)
```

## Definition of done

A change is not done until:

- The intended behavior is implemented.
- Relevant tests or golden fixtures are added or updated.
- The nearest checks pass, or failures are clearly reported with the exact command and error.
- Public APIs and assumptions are documented.
- Relevant changelog, ADRs, data contracts, model-governance notes, examples, or runbooks are updated, or marked not applicable.
- For non-trivial tasks, an implementation report can be produced from `docs/ai/implementation-report-template.md`.
- The diff is limited to the requested scope.

## Actuarial engine rules

- `rustuary-core` must remain framework-free and IO-light.
- Prefer domain types over unstructured maps or dataframes in the core.
- Calculations should return diagnostics, not just final reserves.
- Floating-point tests must use tolerances, not exact equality, unless testing integer/control behavior.
- Keep candidate method results separate from selected/booked results.
- Method selection by origin period and weighted blends are first-class concepts.

## Agent behavior rules inspired by strong coding-agent practice

- Build a short plan for non-trivial tasks, then execute it.
- Maintain local coherence: update tests, docs, and contracts that are directly affected by the code change.
- Be conservative with scope: no drive-by features, speculative architecture, or dependency churn.
- Verify with commands, not confidence.
- When blocked, report the blocker, what was tried, and the smallest next action.
- When finished, include a short suggested commit message in the final response.

## Review checklist

Before finalizing a task, check:

- Could this change alter actuarial results? If yes, are golden tests updated?
- Could this change affect auditability? If yes, are assumptions and diagnostics preserved?
- Could this change affect API compatibility? If yes, are contracts and examples updated?
- Could this change affect user-visible behavior? If yes, is `CHANGELOG.md` updated?
- Could this change introduce or revise a major decision? If yes, is an ADR added or updated?
- Could this change leak data? If yes, stop and remove sensitive material.
