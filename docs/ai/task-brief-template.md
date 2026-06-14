# Agent Task brief template

Use this before starting a non-trivial implementation task. Keep it short. Delete sections that do not apply.

## Goal

What user-visible or system-visible behavior should exist after this task?

## Non-goals

What should not be changed in this task?

## Affected areas

- Rust core:
- Python binding:
- Go service:
- SvelteKit UI:
- Contracts/data schemas:
- Docs/governance:

## Inputs and assumptions

List any actuarial assumptions, source references, fixtures, API constraints, or product decisions. If an assumption is missing, say so instead of inventing one.

## Plan

1.
2.
3.

## Tests and checks

Commands expected to run:

```bash
./scripts/check.sh
```

Add narrower commands when applicable.

## Documentation updates

Mark each item as `required`, `not applicable`, or `deferred with reason`.

- `CHANGELOG.md`:
- ADR:
- `contracts/DATA_CONTRACTS.md`:
- `docs/actuarial/model-governance.md`:
- Examples/golden fixtures:
- Runbooks:

## Acceptance criteria

- [ ] Behavior implemented.
- [ ] Relevant tests or golden fixtures added/updated.
- [ ] Documentation updates completed or explicitly marked not applicable.
- [ ] Checks run and results reported.
