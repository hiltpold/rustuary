---
name: golden-tests
description: Use when adding or updating deterministic examples, expected actuarial outputs, or regression fixtures.
---

Rules:

- Golden data must be synthetic and small.
- Store inputs in `data/examples/` or crate-local test fixtures.
- Store expected outputs in `data/golden/` when shared across languages.
- Include a short note explaining the source of expected values.
- Use tolerances for floating-point comparisons.
- Never overwrite golden results without explaining why the expected behavior changed.
