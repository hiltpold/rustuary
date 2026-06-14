# GitHub Copilot instructions

Use `AGENTS.md` as the source of truth for repository conventions.

Key rules:

- Keep the Rust actuarial core independent from Python, databases, UI, and web frameworks.
- Make small, testable changes.
- Expose actuarial assumptions and diagnostics.
- Never use real client data in examples or tests.


Scoped guidance:

- For Go service work, read `.agents/skills/go-service/SKILL.md` and `services/api/AGENTS.md`.
- For SvelteKit UI work, read `.agents/skills/sveltekit-ui/SKILL.md` and `apps/workbench/AGENTS.md`; use the official Svelte MCP server when configured.
- For MCP setup, see `docs/ai/mcp-setup.md`.
