# Architecture Decision Records

ADRs record important decisions that would be expensive or confusing to rediscover later.

Use an ADR when a change introduces or reverses a meaningful decision about architecture, technology stack, data contracts, public interfaces, deployment, security, or actuarial semantics.

## Index

| ID | Title | Status |
|---|---|---|
| 0001 | Use a monorepo | Accepted |
| 0002 | Core engine boundaries | Accepted |
| 0003 | Data formats | Accepted |
| 0004 | Use governance documents and scoped agent instructions | Accepted |
| 0005 | Use scoped AI tooling for Go and SvelteKit | Accepted |
| 0006 | Keep column mapping outside the Rust core | Accepted |
| 0007 | Use browser WASM only for playground calculations | Accepted |

## Template

Use `scripts/new_adr.sh "Decision title"` or copy `docs/templates/adr-template.md`.
