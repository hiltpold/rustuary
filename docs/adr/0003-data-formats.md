# ADR 0003: Use Arrow and Parquet as analytical data formats

Date: 2026-06-14

## Status

Proposed

## Context

The platform needs efficient interchange between Rust, Python, backend services, batch jobs, and object storage.

## Decision

Use Arrow as the preferred in-memory interchange representation and Parquet as the durable analytical storage format. Use JSON/YAML for small human-reviewed assumption/configuration documents.

## Consequences

- Python, Rust, and data tools can exchange tabular results efficiently.
- Results can be stored in object storage and queried later.
- The core engine should still operate on typed domain objects, not arbitrary dataframes.
