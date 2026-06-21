# Reserving workbench

SvelteKit application for the business-facing reserving workbench.

The current `/` and `/playground` routes render a non-production playground shell
for early workflow review. It uses Tailwind CSS for layout/styling and AG Grid
Community for dense preview grids.

The default playground sample generates a synthetic 500-development-month
triangle so the grid can be reviewed under a more realistic horizontal scrolling
and rendering load before production data is connected.

## Current scope

This app is intentionally small while the actuarial notebook workflow is still
being reviewed. The playground is for interaction design only:

- sample outputs only
- deterministic synthetic stress data only
- no production calculations
- no audit-controlled runs
- no protected claims data in browser state, logs, or URLs

Production calculations must run server-side through the backend/job engine.
See [`docs/adr/0007-use-browser-wasm-only-for-playground.md`](../../docs/adr/0007-use-browser-wasm-only-for-playground.md).

## Routes

- `/` - playground shell
- `/playground` - explicit playground route

## UI stack

- SvelteKit and Svelte 5
- Tailwind CSS for application layout and utility styling
- AG Grid Community for triangle and result preview grids
- Component-scoped Svelte code for route-specific behavior

## Core screens

- portfolio and valuation selection
- data quality review
- triangle view
- factor and tail selection
- candidate method comparison
- accident-year method selection and blending
- selected reserves
- approval and export
- audit history

## Development

Install dependencies from the repository root or this app directory:

```bash
pnpm install
```

Run the local workbench:

```bash
pnpm dev
```

Useful checks:

```bash
pnpm check
pnpm lint
pnpm test
pnpm exec vite build
```

## Design notes

- Keep actuarial assumptions, diagnostics, selections, and rationale visible.
- Prefer accessible controls and readable grids over dashboard decoration.
- Do not recompute production actuarial results in the browser.
- Use generated API clients from `contracts/` once backend contracts are ready.
