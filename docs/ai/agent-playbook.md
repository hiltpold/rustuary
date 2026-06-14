# Agent playbook

## Default loop

1. Read the issue/request and restate the target behavior.
2. For non-trivial implementation tasks, sketch a brief using `docs/ai/task-brief-template.md`.
3. Inspect the nearest `AGENTS.md`, README, tests, and affected code.
4. Make a short plan for non-trivial tasks.
5. Implement the smallest useful change.
6. Add or update tests/golden fixtures.
7. Update changelog, ADRs, contracts, governance docs, examples, or runbooks when behavior or interfaces change.
8. Run the narrowest relevant verification command.
9. Review the diff for accidental scope creep.
10. For non-trivial tasks, summarize with `docs/ai/implementation-report-template.md`.
11. Report what changed and which checks ran.

## What agents should avoid

- Broad rewrites without a specific request.
- Moving files just to improve aesthetics.
- Adding dependencies when a small local implementation is enough.
- Making actuarial assumptions silently.
- Changing public contracts without examples and documentation.
- Changing behavior without updating `CHANGELOG.md` or explicitly marking it not applicable.
- Changing model semantics without updating model governance notes and golden tests.
- Marking work as complete without running or explaining tests.

## Useful task prompts

- `.codex/prompts/implement-actuarial-method.md`
- `.codex/prompts/review-actuarial-change.md`
- `.codex/prompts/add-python-binding.md`
- `.codex/prompts/update-api-contract.md`
- `docs/ai/task-brief-template.md`
- `docs/ai/implementation-report-template.md`

## Retrospective rule

When an agent makes the same mistake twice, update the closest relevant `AGENTS.md` with a short enforceable rule.


## Framework-specific agent tools

### SvelteKit

For SvelteKit work, use the Svelte MCP server when available. Start with documentation discovery, fetch relevant sections, and run the Svelte autofixer after editing Svelte files. If MCP is unavailable, run `pnpm check` and `pnpm lint` and report that MCP autofix was not available.

### Go

For Go work, rely on the standard toolchain first: `gofmt`, `go test`, `go vet`, `gopls`, `govulncheck`, and optional `golangci-lint`. Do not add third-party Go MCP servers without an ADR and security review.
