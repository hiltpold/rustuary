# 0006 — Keep column mapping outside the Rust core

Status: Accepted

Date: 2026-06-14

## Context

Actuaries and business users will provide claims triangles, exposure data, and assumptions with organization-specific column names. The Rust engine needs standardized contracts for correctness, testing, and reproducibility.

## Decision

Rustuary will keep column mapping in adapter layers. Python bindings, CLI
import commands, Go backend import jobs, and the SvelteKit import wizard may
map external columns into canonical Rustuary contracts. The Rust actuarial core
will consume canonical validated domain types only, including dense
`Triangle`s, typed claim/event build records, and validated build requests
after source columns and constants have been resolved by an adapter.

## Consequences

- Python APIs must support both convenience column arguments and reusable mapping objects.
- Run configs and model-run metadata must persist mappings when external schemas are used.
- OpenAPI/protobuf contracts must represent mapping metadata for backend-submitted runs.
- Rust core tests can remain focused on canonical triangles, canonical
  claim/event build records, build requests, and deterministic actuarial
  behavior.
- UI work can include an import wizard without changing calculation code.
