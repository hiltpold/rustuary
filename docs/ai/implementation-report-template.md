# Agent Implementation report template

Use this at the end of a non-trivial task or PR.

## Summary

Briefly describe what changed and why.

## Files changed

Group by area, not by every tiny file.

- Rust core:
- Python binding:
- Go service:
- SvelteKit UI:
- Contracts/data schemas:
- Docs/governance:

## Behavior changes

Describe user-visible, API-visible, model-visible, or schema-visible behavior changes.

## Actuarial impact

- Does this alter actuarial calculations? yes/no
- If yes, which methods/diagnostics are affected?
- Were golden fixtures updated?

## Documentation updates

- `CHANGELOG.md`:
- ADR:
- `contracts/DATA_CONTRACTS.md`:
- `docs/actuarial/model-governance.md`:
- Examples/golden fixtures:
- Runbooks:

## Verification

Commands run and outcomes:

```bash
# command
# outcome
```

## Known gaps or follow-ups

List only concrete follow-ups. Do not hide failures.
