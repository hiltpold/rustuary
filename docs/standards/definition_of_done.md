# Definition of done

A task is done when:

- Behavior matches the request.
- Relevant tests pass.
- Public interfaces are documented.
- Actuarial assumptions and diagnostics are explicit.
- The diff is scoped and reviewable.
- No real client data or secrets are introduced.
- Known limitations are documented.
- Non-trivial agent tasks have a brief and implementation report when useful.
- Changelog, ADRs, contracts, governance docs, examples, or runbooks are updated when the change affects them.

For calculation changes, also include:

- Golden tests or examples.
- Tolerance rationale for floating-point assertions.
- Diagnostics that make intermediate calculations visible.
- `docs/actuarial/model-governance.md` updated when method semantics, assumptions, selections, or diagnostics change.
