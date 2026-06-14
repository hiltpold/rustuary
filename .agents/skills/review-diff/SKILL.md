---
name: review-diff
description: Use before finalizing any non-trivial code change or when asked to review a pull request or local diff.
---

Review steps:

1. List files changed.
2. Check for scope creep.
3. Check tests, changelog, ADRs, contracts, governance docs, examples, and runbooks impacted by the change.
4. Check error handling and edge cases.
5. For actuarial code, check formulas, assumptions, diagnostics, and floating-point tolerances.
6. For platform code, check authorization, auditability, idempotency, and data boundaries.
7. Run or mentally apply `.agents/skills/change-hygiene/SKILL.md`.
8. Report exact commands run and results.

Do not approve your own change without noting residual risks.
