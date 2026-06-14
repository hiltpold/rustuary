# Change management

This repo changes both software behavior and actuarial results. Treat both as reviewable outputs.

## When to update which file

| Change | Required updates |
|---|---|
| User-visible behavior | `CHANGELOG.md`, tests/examples |
| Actuarial formula or result semantics | Golden tests, diagnostics, `docs/actuarial/model-governance.md` |
| Major architecture or stack choice | New or updated ADR |
| Public Python API | Python tests, examples, changelog |
| REST/gRPC/schema contract | `contracts/`, generated examples, changelog |
| Data model or storage format | `contracts/DATA_CONTRACTS.md`, ADR if strategic |
| Deployment or operational behavior | `docs/runbooks/`, release notes |
| Security/data handling | `SECURITY.md`, threat notes if applicable |

## PR expectation

Every PR should either update the relevant docs/contracts or explicitly mark them as not applicable. For non-trivial agent work, use `docs/ai/task-brief-template.md` to scope the change and `docs/ai/implementation-report-template.md` to summarize the handoff.

## Breaking changes

Breaking changes require:

- Clear changelog entry.
- Migration note or compatibility statement.
- Contract/schema versioning note.
- Golden-output explanation if actuarial results change.
