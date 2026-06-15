# Initial implementation backlog

This backlog is ordered for the first vertical slice. Complete tasks in order unless an ADR changes the plan.

## Slice 0 — Canonical contracts, column mapping, and notebook workbench

Goal: allow actuaries to load their own dataframe columns while keeping the Rust engine canonical and deterministic.

- [x] Define Python `ClaimsMapping` and `ExposureMapping` objects.
- [x] Implement `Triangle.from_frame(data, *, origin, development, value, cumulative, ...)` convenience API.
- [x] Implement `Triangle.from_frame(data, mapping=ClaimsMapping(...))` reusable API.
- [x] Accept pandas, Polars, and PyArrow inputs by converting to a canonical PyArrow table.
- [x] Normalize claims input into canonical fields from `contracts/DATA_CONTRACTS.md`.
- [x] Add validation errors that reference both source and canonical column names.
- [x] Persist mapping metadata in `ReserveResult.audit_trail()` or model-run metadata placeholder.
- [x] Add `notebooks/01_chain_ladder_workbench.ipynb` for actuary review sessions.
- [x] Add example custom-column CSV and YAML mapping fixture.

## Slice 1 — Rust chain ladder core

Goal: one reliable deterministic calculation path.

- [x] Finalize canonical Rust `Triangle` domain type.
- [x] Add cumulative/incremental conversion.
- [x] Add latest diagonal extraction.
- [ ] Add link-ratio calculation.
- [ ] Add volume-weighted development factor selection.
- [ ] Add simple-average development factor selection.
- [ ] Add selected factor overrides and exclusions.
- [ ] Add fixed tail factor interface.
- [ ] Add CDF calculation including tail.
- [ ] Add chain ladder ultimate, reserve, and diagnostics.
- [ ] Add golden fixture tests with documented tolerance policy.

## Slice 2 — Python binding for chain ladder

Goal: actuaries can run the Rust calculation from Python notebooks.

- [ ] Add PyO3 binding for canonical chain ladder calculation.
- [ ] Add Python `ChainLadder` class.
- [ ] Add `ReserveResult.summary()`, `.diagnostics()`, `.to_arrow()`, `.to_pandas()`.
- [ ] Add notebook smoke test that loads custom-column data and runs chain ladder.
- [ ] Add Python tests for mapping, validation, and result shape.
- [ ] Update `CHANGELOG.md`, `contracts/DATA_CONTRACTS.md`, and model-governance docs if behavior changes.

## Slice 3 — Additional deterministic reserving methods

- [ ] Add a priori / expected loss.
- [ ] Add Bornhuetter-Ferguson.
- [ ] Add Cape Cod.
- [ ] Add exposure vectors and expected loss ratios.
- [ ] Add candidate-vs-selected result model.
- [ ] Add accident-year method selection and weighted blending.

## Slice 4 — Platform integration

- [ ] Finalize OpenAPI run-submission contract, including column mapping metadata.
- [ ] Define metadata tables for model runs, assumptions, audit events.
- [ ] Define object-store path conventions.
- [ ] Define job lifecycle and retry semantics.
- [ ] Add SvelteKit import wizard design for mapping user columns to canonical fields.
