# Code review guide

## General

- Is the change limited to the requested scope?
- Are tests or examples updated where behavior changed?
- Are errors actionable and understandable?
- Are new dependencies justified?
- Are public APIs documented?
- Are relevant governance docs, changelog entries, ADRs, contracts, examples, or runbooks updated?

## Actuarial calculations

- Are formulas visible and auditable?
- Are assumptions explicit?
- Are diagnostics preserved?
- Are candidate and selected results separated?
- Are floating-point tests tolerant but strict enough?
- Are golden fixtures updated intentionally?
- Is `docs/actuarial/model-governance.md` updated for method semantics or assumption changes?

## Platform

- Are authorization boundaries clear?
- Does metadata stay in PostgreSQL and large data stay in object storage?
- Are migrations reversible or at least safely forward-only?
- Are audit events appended rather than overwritten?

## UI

- Are workflows understandable to actuaries?
- Are selected assumptions and overrides visible?
- Is accessibility considered?
- Are destructive actions confirmed?
