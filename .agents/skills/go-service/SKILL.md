# Go service skill

Use this skill when changing Go code under `services/api`, `services/worker`, or future Go services.

## Goal

Keep Go services boring, observable, testable, and thin around the actuarial engine. Go owns API, auth, RBAC, workflow, audit, metadata, job orchestration, and exports. It must not reimplement actuarial formulas.

## Before editing

- Read root `AGENTS.md` and the nearest service `AGENTS.md`.
- Inspect package boundaries before adding new packages.
- Check `contracts/` before changing request/response shapes.
- Check `docs/standards/engineering-principles.md` for design preferences.

## Design rules

- Keep HTTP handlers thin: parse, authorize, validate, call application service, map response.
- Put business workflow in `internal/<domain>` packages, not in `cmd/`.
- Accept `context.Context` as the first parameter for operations that may block, call external systems, or touch storage.
- Prefer small interfaces owned by consumers.
- Avoid global mutable state. Inject dependencies through structs.
- Return errors with context. Do not swallow errors.
- Log structured events; never log secrets, tokens, claims-level PII, or raw client data.
- Use object-store references for large Arrow/Parquet payloads instead of embedding large tables in JSON.
- Update OpenAPI/protobuf contracts and examples when API behavior changes.

## Testing rules

- Prefer table-driven tests for validation, mapping, and workflow cases.
- Use `httptest` for handler behavior.
- Mock external systems behind small interfaces.
- Add integration tests only when unit tests cannot prove the behavior.
- Consider Go fuzz tests for parsers, schema normalization, and boundary-heavy validation.

## Commands

Run the narrowest relevant checks first:

```bash
cd services/api
gofmt -w .
go test ./...
go vet ./...
```

If installed, also run:

```bash
golangci-lint run
govulncheck ./...
```

Report skipped optional checks explicitly.
