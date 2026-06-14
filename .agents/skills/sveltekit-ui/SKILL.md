# SvelteKit UI skill

Use this skill when changing `apps/workbench`, Svelte, SvelteKit, TypeScript UI code, frontend tests, routes, load functions, form actions, or generated API clients used by the workbench.

## Goal

Build a transparent actuarial workbench: assumptions, diagnostics, selections, candidate methods, rationale, and audit state must be visible and understandable.

## Svelte MCP usage

When an MCP client has the Svelte MCP server configured:

1. For Svelte/SvelteKit tasks, use the `svelte-task` prompt if the client supports MCP prompts.
2. Use `list-sections` first to find relevant docs.
3. Use `get-documentation` for all relevant sections before implementing unfamiliar Svelte/SvelteKit features.
4. Run `svelte-autofixer` after writing or editing Svelte code. Repeat until it returns no issues or suggestions.
5. Do not use `playground-link` for code written to files in this repository.

If the MCP server is unavailable, proceed using local project checks and mention that MCP autofix was not available.

## UI rules

- Use Svelte 5 and SvelteKit conventions.
- Prefer server-side loading and API calls through generated clients.
- Do not recompute actuarial results in the browser except trivial display transforms.
- Make state explicit and serializable where practical.
- Keep components focused: view components render, route/load/action modules fetch or mutate, shared utilities transform presentation data.
- Favor accessibility, keyboard navigation, readable tables, and clear error messages over flashy dashboards.
- Never expose secrets or protected raw claims data in browser logs, URLs, or client-side stores.
- Confirm destructive workflow actions.
- Update contracts/examples when UI expectations require API shape changes.

## Commands

```bash
cd apps/workbench
pnpm install
pnpm check
pnpm test
pnpm lint
```

Run only the commands available in the current environment and report skipped checks.
