# Testing strategy

## Test pyramid

- Unit tests for small pure functions and validators.
- Integration tests for method workflows and adapters.
- Golden tests for actuarial outputs.
- Contract tests for service/API boundaries.
- End-to-end tests only for critical user workflows.

## Actuarial golden tests

Golden tests should include:

- Input triangle or exposure fixture.
- Assumptions and selections.
- Expected outputs with tolerance policy.
- Diagnostics necessary to explain the result.
- A note when an expected value changes.

## Floating-point policy

Use tolerances for floating-point assertions. Document whether the tolerance is absolute, relative, or both.

## Agent expectations

Agents should run the narrowest relevant test first and report exact commands and failures.
