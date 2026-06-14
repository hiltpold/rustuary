# Agent instruction map

Rustuary uses scoped instructions rather than one large all-purpose prompt.

## Source of truth

- `AGENTS.md`: repository-wide agent behavior, architecture boundaries, and verification expectations.
- Nested `AGENTS.md`: folder-specific rules that override or narrow the root guidance.
- `.agents/skills/*/SKILL.md`: reusable task workflows for Codex-style agents.
- `CLAUDE.md`: compatibility pointer for Claude-style agents; it should redirect to `AGENTS.md` and not become a second source of truth.

## Where technology-specific guidance belongs

| Guidance | Location |
|---|---|
| General repository behavior | root `AGENTS.md` |
| Rust coding standards | `.agents/skills/rust-quality/SKILL.md` and `engines/rustuary-core/AGENTS.md` |
| PyO3/maturin/Python binding workflow | `bindings/python/AGENTS.md` and `.agents/skills/python-binding/SKILL.md` |
| Go backend rules | `services/api/AGENTS.md` |
| SvelteKit UI rules | `apps/workbench/AGENTS.md` |
| Actuarial method implementation | `.agents/skills/actuarial-method/SKILL.md` |
| Engineering principles | `docs/standards/engineering-principles.md` |
| Change/documentation update policy | `CONTRIBUTING.md`, `.github/pull_request_template.md`, `docs/standards/change-management.md`, `.agents/skills/change-hygiene/SKILL.md` |
| Actuarial model governance | `docs/actuarial/model-governance.md` |
| Data contracts | `contracts/DATA_CONTRACTS.md` and `contracts/` schemas |
| Long rationale/reference material | skill `references/` files or `docs/ai/` |
| Task scoping | `docs/ai/task-brief-template.md` |
| Task completion reporting | `docs/ai/implementation-report-template.md` |
| Runtime/tool versions | `rust-toolchain.toml`, `.mise.toml`, `.tool-versions`, `.node-version`, `.python-version`, `.devcontainer/devcontainer.json` |

## Rule of thumb

Keep `AGENTS.md` files short and always-on. Put longer, task-specific workflows in skills. Put background rationale in references.


## Language and framework skills

- Rust core changes: `.agents/skills/rust-quality/SKILL.md` and `engines/rustuary-core/AGENTS.md`
- Python binding changes: `.agents/skills/python-binding/SKILL.md` and `bindings/python/AGENTS.md`
- Go service changes: `.agents/skills/go-service/SKILL.md` and `services/api/AGENTS.md`
- SvelteKit UI changes: `.agents/skills/sveltekit-ui/SKILL.md` and `apps/workbench/AGENTS.md`

## MCP

MCP setup lives in `docs/ai/mcp-setup.md`, `.codex/config.example.toml`, and `.mcp/`. The Svelte MCP server is recommended for workbench UI tasks when available.
