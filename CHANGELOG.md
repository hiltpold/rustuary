# Changelog

All notable changes to this repository should be documented here.

This project follows the spirit of Keep a Changelog: write entries for people who need to understand behavior, compatibility, and operational impact. Do not paste raw git history.

## [Unreleased]

### Added

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

### Deprecated

### Removed

### Fixed

### Security

## Release entry checklist

When a PR changes user-visible behavior, public APIs, actuarial results, schemas, deployment behavior, or security posture, update the relevant section above before merging.

Use short, result-oriented bullets:

- Good: `Added Chain Ladder diagnostics for selected factors and CDFs.`
- Bad: `Updated files.`
