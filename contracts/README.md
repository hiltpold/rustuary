# Contracts

Shared API and data contracts.

- `openapi/`: REST contracts used by UI and external clients.
- `proto/`: gRPC contracts for internal services or engine workers.
- `schemas/`: logical table schemas for Arrow/Parquet artifacts.
- `examples/`: small synthetic mapping and triangle-definition examples.

Contract rules:

- Do not expose internal engine structs directly.
- Large tabular inputs and outputs should be referenced by artifact URI.
- Every model run should have a stable ID, valuation date, assumption version, engine version, and audit metadata.
