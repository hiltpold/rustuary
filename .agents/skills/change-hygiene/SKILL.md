---
name: change-hygiene
description: Use before finalizing implementation changes to decide which governance, changelog, ADR, contract, model-governance, runbook, or example files must be updated.
---

Use this skill after code changes and before reporting completion.

Checklist:

1. Did user-visible behavior change?
   - Update `CHANGELOG.md`.
2. Did a major architecture, stack, storage, or interface decision change?
   - Add or update an ADR under `docs/adr/`.
3. Did an actuarial formula, method selection rule, assumption, diagnostic, or result semantic change?
   - Update golden tests and `docs/actuarial/model-governance.md`.
4. Did a data shape, wire shape, API endpoint, or schema change?
   - Update `contracts/`, `contracts/DATA_CONTRACTS.md`, and examples.
5. Did deployment, operations, jobs, migrations, or recovery behavior change?
   - Update `docs/runbooks/` and `RELEASE.md` if release-affecting.
6. Did security, secrets, PII, RBAC, audit, export, or data-retention behavior change?
   - Update `SECURITY.md` or a security/design note.
7. Did public usage change?
   - Update README, examples, or notebook snippets.

If none apply, state why in the PR summary or final response.

Verification:

```bash
./scripts/check_repo_hygiene.sh
```
