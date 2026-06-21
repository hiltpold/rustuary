# 0007 — Use browser WASM only for playground calculations

Status: Accepted

Date: 2026-06-21

## Context

The SvelteKit workbench needs an early visual playground while the actuary
notebook workflow is still being reviewed. Browser-side Rust/WASM can make a
demo feel responsive, but production reserving runs need controlled inputs,
server-side orchestration, durable audit metadata, and repeatable job records.

## Decision

Rustuary may use browser WASM only for playground and demo calculations.
Playground output must be clearly labeled as non-production and not
audit-controlled.

Production calculations must run server-side through the backend and job
engine. The backend/job engine is responsible for invoking approved calculation
artifacts, preserving run metadata, enforcing authorization, and making results
available for review, approval, and export.

## Consequences

- The workbench can show early interaction designs before the full platform is
  ready.
- Browser playground code cannot become the production execution path without a
  new architecture decision.
- Production workflows keep calculation lineage, permissions, and audit state
  outside browser-only state.
