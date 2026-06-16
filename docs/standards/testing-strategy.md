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

Use tolerances for floating-point assertions. Rust actuarial golden tests use
an absolute tolerance of `1e-9` and no relative tolerance unless a test states
otherwise. Expected values should carry enough decimal precision to reproduce
the hand calculation. If an expected value changes, document whether the change
is a correction to the fixture or an intentional behavior change.

## Agent expectations

Agents should run the narrowest relevant test first and report exact commands and failures.
