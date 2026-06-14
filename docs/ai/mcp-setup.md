# MCP setup

This repository keeps MCP configuration examples in version control, but most clients require users to opt in locally.

## Svelte MCP

Use the official Svelte MCP server when working on `apps/workbench`.

Recommended options:

- Remote MCP: `https://mcp.svelte.dev/mcp`
- Local MCP: `npx -y @sveltejs/mcp`

Config examples:

- `.codex/config.example.toml` for Codex CLI
- `.mcp/svelte-remote.json` for clients that accept JSON MCP server definitions
- `.mcp/svelte-local.json` for clients that prefer local stdio servers

When available, agents should use the Svelte MCP tools as described in `.agents/skills/sveltekit-ui/SKILL.md`.

## Go tooling

There is no official Go MCP server configured in this repository. Prefer official/static tooling for Go work:

- `gopls` for language-server support
- `gofmt` for formatting
- `go test` and `go vet` for correctness checks
- `govulncheck` for vulnerability checks
- `golangci-lint` as the optional aggregate linter

Do not add a third-party Go MCP server without an ADR, a security review, and a clear reason it improves agent output beyond the standard Go toolchain.
