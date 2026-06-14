# Architecture overview

Rustuary is designed as a layered actuarial platform.

## Layers

1. **Actuarial core**: deterministic calculations, diagnostics, selections, and audit-friendly result structures.
2. **Language bindings**: Python first, because actuaries and analytics teams can use it directly.
3. **Platform backend**: workflow state, identity, RBAC, job orchestration, exports, and metadata.
4. **Workbench UI**: guided review of triangles, assumptions, candidates, selections, diagnostics, approvals.
5. **Data platform**: object storage for raw/normalized/result files, PostgreSQL for metadata, Arrow/Parquet for analytical data.

## Non-goals for the core

The Rust core should not know about users, permissions, HTTP, S3, PostgreSQL, Excel, notebooks, or UI state.

## Primary data flow

```text
raw claims/exposure data
  -> normalized Arrow/Parquet datasets
  -> validated Triangle / Exposure domain objects
  -> candidate actuarial methods
  -> selected reserve result
  -> Parquet result tables + metadata + audit trail
```

## Core calculation design

The engine should separate:

- candidate method calculation
- assumption selection
- accident-year method selection
- weighted blending
- final booked result
- diagnostics and audit trail

This separation is essential because actuaries need to compare methods and document judgment.
