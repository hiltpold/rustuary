---
name: api-contract
description: Use when editing OpenAPI, protobuf, backend DTOs, API routes, job contracts, or service boundaries.
---

Rules:

- Contracts must be versioned under `contracts/`.
- Do not expose internal engine structs directly as public API payloads.
- Include request IDs and model run IDs where useful for auditability.
- Large tabular data should move through object-store references or Arrow/Parquet artifacts, not giant JSON payloads.
- Update examples when a contract changes.

Verify:

- Run service tests if backend code changes.
- Check that UI and Python SDK expectations are still coherent.
