# Changelog

All notable changes to this repository should be documented here.

This project follows the spirit of Keep a Changelog: write entries for people who need to understand behavior, compatibility, and operational impact. Do not paste raw git history.

## [Unreleased]

### Added

- Chain-ladder workbench notebook smoke test now runs the mapped custom-column triangle through `ChainLadder`.
- Python `ReserveResult.audit_trail()` for result summaries, diagnostics, and source-to-canonical input lineage.
- Python `ReserveResult.to_pandas()` for origin-level summary export as a pandas DataFrame.
- Python `ReserveResult.to_arrow()` for origin-level summary export as a PyArrow table.
- Python `ReserveResult.diagnostics()` for selected factors, CDFs, tail factor, basis, and origin diagnostics.
- Python `ReserveResult.summary()` for origin-level latest, ultimate, and reserve output.
- Python `ChainLadder.fit_predict` support for mapped `Triangle.from_frame(...)` inputs.
- Public Python `ChainLadder` class for dense canonical triangle runs.
- Low-level PyO3 chain-ladder binding for canonical dense triangles.
- TriangleDefinition contract language for `portfolio_id`, ordered segments, and derived display paths.
- Shared chain-ladder golden fixture coverage with documented `1e-9` absolute tolerance policy.
- Chain-ladder origin diagnostics for latest observed values, CDF components, ultimates, and reserves.
- Typed CDF diagnostics by development age, including remaining factor product and tail factor.
- Typed fixed tail factor interface with positive finite validation and optional rationale.
- Auditable selected-factor overrides and link-ratio exclusions with required rationales.
- Typed simple-average development factor selections from individual link ratios.
- Typed volume-weighted development factor selections with supporting aggregates.
- Typed cumulative link-ratio diagnostics by origin and development interval.
- Typed Rust latest-diagonal extraction with origin and development labels.
- Deterministic cumulative and incremental Rust triangle conversion with preserved axes and missing cells.
- Canonical Rust `Triangle` axes, cumulative/incremental basis, and construction validation.
- Validated custom-column paid claims CSV and matching YAML mapping fixtures.
- Executable chain-ladder input-review notebook for custom-column claims mapping and audit metadata.
- JSON-safe `ModelRunMetadata` mapping snapshots on canonical Python triangles.
- Source- and canonical-aware `ColumnMappingError` messages for invalid claims mappings.
- Canonical claims-field normalization for mapped `Triangle` inputs.
- PyArrow table conversion for pandas, Polars, PyArrow, and record-sequence triangle inputs.
- Reusable `ClaimsMapping` support in `Triangle.from_frame`.
- Expanded `Triangle.from_frame` named mapping parameters to cover the complete claims column-mapping contract.
- Frozen Python `ClaimsMapping` and `ExposureMapping` objects for reusable canonical column mapping configuration.
- Starter monorepo scaffold for the Rust actuarial core, Python bindings, Go services, SvelteKit workbench, contracts, docs, infra, and agent workflows.
- Repository governance documents for contribution, release, security, roadmap, model governance, data contracts, and engineering principles.
- Scoped Go service and SvelteKit UI agent skills, Svelte MCP setup examples, and Go quality tooling hooks.
- Pinned tool-version files for Rust, Go, Node, pnpm, Python, and uv.
- Starter dev container and VS Code extension recommendations for reproducible local development.
- Agent task brief and implementation report templates for non-trivial coding tasks.
- GitHub issue templates for features, bugs, and actuarial method changes.
- Stronger Rust verification gates for clippy, workspace tests, and rustdoc warnings.

### Changed

- Selected development-factor numerator and denominator diagnostics are now method-specific aggregates.
- Rust chain-ladder factor calculation now rejects incremental triangles until explicitly converted.

### Deprecated

### Removed

### Fixed

### Security

## Release entry checklist

When a PR changes user-visible behavior, public APIs, actuarial results, schemas, deployment behavior, or security posture, update the relevant section above before merging.

Use short, result-oriented bullets:

- Good: `Added Chain Ladder diagnostics for selected factors and CDFs.`
- Bad: `Updated files.`
