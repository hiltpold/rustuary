# Contributing

This repository is designed for humans and coding agents working together. Keep changes small, verified, and easy to audit.

## First reads

Before editing, read:

1. `AGENTS.md`
2. `docs/ai/instruction-map.md`
3. The nearest nested `AGENTS.md` in the folder you are editing
4. The relevant `.agents/skills/*/SKILL.md`
5. `docs/standards/definition_of_done.md`

## Development environment

Tool versions are pinned in `rust-toolchain.toml`, `.mise.toml`, `.tool-versions`, `.node-version`, and `.python-version`. Use `mise install`, `asdf install`, the provided `.devcontainer/devcontainer.json`, or equivalent local tooling to align with the repo before making changes.

## Development workflow

1. Create a small branch for one change.
2. For non-trivial implementation work, prepare a short task brief from `docs/ai/task-brief-template.md`.
3. Inspect existing tests and examples before editing.
4. Update code, tests, docs, contracts, and fixtures together when behavior changes.
5. Run the narrowest relevant checks.
6. For non-trivial work, prepare a short implementation report from `docs/ai/implementation-report-template.md`.
7. Open a PR with clear context, verification output, and any unresolved risks.

## Commit style

Prefer Conventional Commit style:

```text
feat(core): add Bornhuetter-Ferguson ultimate calculation
fix(python): preserve origin order in Triangle.from_frame
docs(adr): record Arrow and Parquet interchange decision
test(core): add golden chain ladder fixture
```

## Documentation update rule

Every implementation change must answer this question:

> Did this change alter behavior, assumptions, contracts, workflow, deployment, security, or user-facing output?

If yes, update one or more of:

- `CHANGELOG.md`
- `docs/adr/*.md`
- `contracts/DATA_CONTRACTS.md`
- `contracts/openapi/*`
- `contracts/proto/*`
- `docs/actuarial/model-governance.md`
- `docs/standards/*`
- `docs/runbooks/*`
- examples or golden fixtures under `data/`

If no, say why in the PR description.

## Testing expectations

- Calculation changes need unit tests and golden tests when outputs change.
- Contract changes need examples and compatibility notes.
- Python API changes need typed public signatures and tests.
- Backend changes need API/service tests and migration notes where applicable.
- UI changes need user-flow checks and accessibility consideration.

## Data policy

Never commit real client data, credentials, PII, production logs, or confidential reports. Use tiny synthetic fixtures only.
