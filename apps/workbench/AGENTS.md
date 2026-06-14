# Workbench UI agent instructions

The UI is for actuarial review and workflow, not hidden calculation.

## Required context

- Read root `AGENTS.md`.
- Read `.agents/skills/sveltekit-ui/SKILL.md` before editing Svelte/SvelteKit code.
- Use the official Svelte MCP server when available. Setup examples are in `docs/ai/mcp-setup.md`.

## Svelte MCP workflow

When MCP is available:

- Use `list-sections` first for Svelte/SvelteKit work.
- Use `get-documentation` for relevant sections.
- Use `svelte-autofixer` after editing Svelte code and repeat until clean.
- Do not create playground links for code committed to this repository.

## Rules

- Show assumptions, selected factors, tails, method candidates, selections, and rationale.
- Do not recompute actuarial results in the browser except for trivial display transforms.
- Favor accessibility, keyboard navigation, and clear tables over flashy dashboards.
- Keep destructive actions explicit and confirmable.
- Use generated clients from `contracts/` once available.
- Keep secrets and protected claims data out of client-side state, logs, and URLs.

## Commands

```bash
pnpm install
pnpm check
pnpm lint
pnpm test
```
