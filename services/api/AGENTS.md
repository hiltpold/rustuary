# API service agent instructions

The API service owns application workflow concerns: auth, RBAC, metadata, audit, job submission, exports, and object-store references.

## Required context

- Read root `AGENTS.md`.
- Read `.agents/skills/go-service/SKILL.md` before editing Go service code.
- Read `contracts/` before changing API payloads.

## Rules

- Do not implement actuarial formulas in Go. Call the engine or read persisted engine outputs.
- Keep large tabular payloads out of JSON. Use object-store references to Arrow/Parquet artifacts.
- Add audit events for user-visible workflow changes.
- API handlers should be thin; business logic belongs in internal packages.
- Authorization checks must happen before reading or writing protected resources.
- Use `context.Context` for blocking, external, storage, and workflow operations.
- Prefer table-driven tests and `httptest` for handlers.

## Commands

```bash
gofmt -w .
go test ./...
go vet ./...
# Optional if installed:
golangci-lint run
govulncheck ./...
```
