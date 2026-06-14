# 0005: Use scoped AI tooling for Go and SvelteKit

## Status

Accepted

## Context

The repository will contain a Go backend and a SvelteKit workbench in addition to the Rust actuarial core and Python bindings. AI-agent guidance must help contributors use the right tools for each technology without turning root `AGENTS.md` into a large, conflicting style guide.

Svelte has an official MCP server and AI instructions designed to improve generated Svelte and SvelteKit code. Go has strong official tooling through the standard toolchain, `gopls`, and vulnerability tooling, but no official Go MCP server has been selected for this repository.

## Decision

We will keep root `AGENTS.md` technology-neutral and add scoped skills:

- `.agents/skills/go-service/SKILL.md` for Go service work
- `.agents/skills/sveltekit-ui/SKILL.md` for SvelteKit UI work

We will add Svelte MCP setup examples under `.codex/`, `.mcp/`, and `docs/ai/mcp-setup.md`.

For Go, we will rely on:

- `gofmt`
- `go test`
- `go vet`
- `gopls`
- `govulncheck`
- optional `golangci-lint`

We will not add third-party Go MCP servers without a separate ADR and security review.

## Consequences

- Agents get precise instructions for frontend and backend work without polluting global guidance.
- SvelteKit changes can use official Svelte MCP documentation and autofix loops when available.
- Go changes remain grounded in official and widely adopted static tooling.
- Future MCP additions require explicit justification, which reduces supply-chain and prompt-injection risk.
