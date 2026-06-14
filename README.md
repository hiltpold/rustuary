# Rustuary

Rustuary is a monorepo starter for a high-performance actuarial reserving platform.

It is intentionally split into:

- **Core engine**: deterministic actuarial calculations in Rust.
- **Python bindings**: notebook/business interface for actuaries.
- **Backend services**: API, auth, RBAC, workflow, jobs, audit, metadata, exports.
- **Workbench UI**: reserving workbench for review, selection, diagnostics, and approval.
- **Contracts and data formats**: OpenAPI, protobuf, Arrow/Parquet schemas, examples.
- **AI-agent guidance**: `AGENTS.md`, nested overrides, skills, review checklists, and repeatable prompts.
- **Governance docs**: changelog, ADRs, release process, security policy, data contracts, model governance, and engineering principles.

The name is a placeholder. Rename the package, crate, and services once the product name is chosen.

## Target architecture

```text
                           ┌────────────────────────┐
                           │      SvelteKit UI       │
                           │ reserving workbench     │
                           └───────────┬────────────┘
                                       │
                              HTTPS / REST / gRPC
                                       │
┌──────────────────────────────────────▼──────────────────────────────────────┐
│                               Go Backend                                    │
│ API, auth, RBAC, workflow, job orchestration, audit, metadata, exports       │
└──────────────┬──────────────────────┬──────────────────────┬───────────────┘
               │                      │                      │
        ┌──────▼──────┐       ┌───────▼────────┐      ┌──────▼──────┐
        │ PostgreSQL  │       │ Job Queue /    │      │ Object Store │
        │ OLTP store  │       │ Workflow engine│      │ S3/Blob/GCS  │
        └─────────────┘       └───────┬────────┘      └──────┬──────┘
                                       │                      │
                               ┌───────▼────────┐             │
                               │ Rust Engine    │◄────────────┘
                               │ calculations   │
                               └───────┬────────┘
                                       │
                               ┌───────▼────────┐
                               │ Results store  │
                               │ Parquet/Arrow  │
                               └────────────────┘
```

## Repository map

```text
apps/                  User-facing applications
  workbench/           SvelteKit reserving workbench placeholder

services/              Backend services
  api/                 Go API service: auth, RBAC, metadata, audit, exports
  worker/              Go worker/job orchestration placeholder

engines/               Calculation engines
  rustuary-core/       Pure Rust actuarial calculation crate

bindings/              Language bindings
  python/              PyO3/maturin Python package skeleton

contracts/             API and wire contracts
  openapi/             REST API contract
  proto/               gRPC/protobuf contract
  schemas/             Arrow/Parquet logical schemas

data/                  Small non-sensitive examples and golden test fixtures
  examples/            Toy claims triangles and exposures
  golden/              Golden outputs used by tests

docs/                  Architecture, ADRs, runbooks, standards, product notes
infra/                 Local and deployable infrastructure skeletons
scripts/               Developer scripts used by humans and agents
.agents/skills/        Codex skills for repeatable repo workflows
.codex/prompts/        Reusable task prompts for Codex sessions
```

## Quick start

```bash
# Check repo shape and run available tests.
./scripts/check.sh

# Rust core only.
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Go services once implemented.
(cd services/api && go test ./...)

# UI once dependencies are installed.
(cd apps/workbench && pnpm install && pnpm check)
```

## Development sequence

1. Build `engines/rustuary-core` until chain ladder, BF, Cape Cod, a priori, tail factors, and accident-year selections are correct.
2. Add Arrow/Parquet adapters around the core without polluting the core with IO concerns.
3. Add `bindings/python` with a friendly actuarial API: `Triangle`, `ChainLadder`, `ReservingWorkflow`, `ReserveResult`.
4. Add backend orchestration only after the local library workflow is validated with business users.
5. Add UI when the workflow, diagnostics, and audit model are stable enough to review visually.

## Design principles

- Rust core owns actuarial math and deterministic diagnostics.
- Python owns user ergonomics, notebooks, plotting, Excel, and business workflows.
- Backend owns identity, permissions, workflow state, audit, jobs, and exports.
- UI owns review, comparison, selection, approval, and explainability.
- Arrow is the in-memory interchange target; Parquet is the durable analytical result format.
- PostgreSQL stores metadata and workflow state, not large triangle/result matrices.
- Object storage stores raw inputs, normalized datasets, Arrow/Parquet outputs, and export artifacts.

## AI-agent entry points

Read these first when using Codex or another coding agent:

1. `AGENTS.md`
2. `docs/ai/instruction-map.md`
3. `docs/ai/agent-playbook.md`
4. `docs/standards/definition_of_done.md`
5. The nearest nested `AGENTS.md` in the folder you are editing
6. The relevant skill in `.agents/skills/`

Agents should make small, verifiable changes and run the nearest tests before reporting success. For non-trivial implementation tasks, use `docs/ai/task-brief-template.md` before coding and `docs/ai/implementation-report-template.md` before handoff.

## Governance and change tracking

Tool versions are pinned in `rust-toolchain.toml`, `.mise.toml`, `.tool-versions`, `.node-version`, and `.python-version`. A starter dev container lives in `.devcontainer/devcontainer.json`.

Important root documents:

- `CHANGELOG.md`: user-visible changes and release notes.
- `CONTRIBUTING.md`: contribution workflow and documentation update rule.
- `RELEASE.md`: release checklist and artifact expectations.
- `SECURITY.md`: data, secrets, and vulnerability policy.
- `GOVERNANCE.md`: review ownership and decision types.
- `ROADMAP.md`: high-level direction.
- `docs/adr/`: architecture decision records.
- `docs/standards/engineering-principles.md`: SOLID/functional-programming guidance and layering principles.
- `docs/actuarial/model-governance.md`: actuarial method and assumption governance.
- `contracts/DATA_CONTRACTS.md`: canonical logical data schemas.


## AI and MCP tooling

This repository is configured for scoped agent workflows:

- Root behavior: `AGENTS.md`
- Rust quality: `.agents/skills/rust-quality/SKILL.md`
- Go services: `.agents/skills/go-service/SKILL.md`
- SvelteKit UI: `.agents/skills/sveltekit-ui/SKILL.md`
- Python bindings: `.agents/skills/python-binding/SKILL.md`
- MCP setup examples: `docs/ai/mcp-setup.md`, `.codex/config.example.toml`, `.mcp/`

For SvelteKit work, configure the official Svelte MCP server if your client supports MCP. For Go work, prefer the standard Go toolchain plus `gopls`, `govulncheck`, and optional `golangci-lint`.


## First implementation slices

The project starts with a dataframe-to-Rust boundary before adding more reserving methods.

1. Canonical data contracts and Python column mapping.
2. Rust chain ladder core with tail factor and diagnostics.
3. PyO3 binding and notebook workbench.
4. Additional deterministic methods: a priori, Bornhuetter-Ferguson, Cape Cod.
5. Method selection, weighted blending, and platform integration.

The Rust core consumes canonical domain types only. User-specific dataframe columns are mapped in the Python adapter, CLI, backend import jobs, or UI import wizard.
