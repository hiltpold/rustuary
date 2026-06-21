# Initial implementation backlog

This backlog is ordered for the first vertical slice. Complete tasks in order unless an ADR changes the plan.

## Guiding conventions

* The Rust engine uses canonical contracts and deterministic domain types.

* Python adapts user dataframes into canonical contracts through explicit mappings.

* Python `TriangleBuilder` owns dataframe adaptation and source-column
  validation; `rustuary-core` owns deterministic triangle build semantics after
  inputs are canonicalized.

* `portfolio_id` represents the main reserving class / actuarial reserving unit.

* `segments` represent optional drill-down dimensions below `portfolio_id`.

* Segment order is the order in which segments are defined in `TriangleDefinition`.

* Do not store `segment_path` as canonical input. Display/folder paths are derived from:

  ```text
  portfolio_id + ordered segment values
  ```

* The triangle grouping key is:

  ```text
  portfolio_id + segments + measure
  ```

* The UI may display a ResQ-like folder tree, but the canonical model remains structured data.

* Chain ladder, BF, Cape Cod, and other methods should operate on validated canonical triangles, not raw user dataframes.

* Raw claim/event records are converted into triangles through `TriangleDefinition` and `TriangleBuilder`.

---

## Slice 0 — Canonical contracts, column mapping, and notebook workbench

Goal: allow actuaries to load their own dataframe columns while keeping the Rust engine canonical and deterministic.

* [x] Define Python `ClaimsMapping` and `ExposureMapping` objects.
* [x] Implement `Triangle.from_frame(data, *, origin, development, value, cumulative, ...)` convenience API.
* [x] Implement `Triangle.from_frame(data, mapping=ClaimsMapping(...))` reusable API.
* [x] Accept pandas, Polars, and PyArrow inputs by converting to a canonical PyArrow table.
* [x] Normalize claims input into canonical fields from `contracts/DATA_CONTRACTS.md`.
* [x] Add validation errors that reference both source and canonical column names.
* [x] Persist mapping metadata in `ReserveResult.audit_trail()` or model-run metadata placeholder.
* [x] Add `notebooks/01_chain_ladder_workbench.ipynb` for actuary review sessions.
* [x] Add example custom-column CSV and YAML mapping fixture.

---

## Slice 0.1 — Refine language, portfolio, and segmentation contracts

Goal: align the canonical language with the current actuarial workflow before extending implementation.

* [x] Update `docs/actuarial/language-and-conventions.md`.
* [x] State clearly that `portfolio_id` is the reserving class / main actuarial reserving unit.
* [x] State clearly that `segments` are optional ordered drill-down dimensions.
* [x] Remove `segment_path` as a required canonical input field if currently documented.
* [x] Document that display paths are derived from `portfolio_id + ordered segments`.
* [x] Update `contracts/DATA_CONTRACTS.md` with the simplified grouping convention.
* [x] Update `contracts/schemas/column_mapping.md` to include `portfolio_id` and ordered `segments`.
* [x] Add or update `contracts/schemas/triangle_definition.md`.
* [x] Add example triangle definition using `portfolio_id="reservingClass"`.
* [x] Add example triangle definition with ordered segments such as `country`, `channel`, and `coverage`.
* [x] Add one example showing no segments, only `portfolio_id`.
* [x] Update model-governance docs to state that every triangle must be traceable to its `TriangleDefinition`.
* [x] Update `CHANGELOG.md` under `[Unreleased]`.

---

## Slice 1 — Rust chain ladder core

Goal: one reliable deterministic calculation path.

* [x] Finalize canonical Rust `Triangle` domain type.
* [x] Add cumulative/incremental conversion.
* [x] Add latest diagonal extraction.
* [x] Add link-ratio calculation.
* [x] Add volume-weighted development factor selection.
* [x] Add simple-average development factor selection.
* [x] Add selected factor overrides and exclusions.
* [x] Add fixed tail factor interface.
* [x] Add CDF calculation including tail.
* [x] Add chain ladder ultimate, reserve, and diagnostics.
* [x] Add golden fixture tests with documented tolerance policy.

---

## Slice 2 — Python binding for canonical chain ladder

Goal: actuaries can run the Rust calculation from Python notebooks on an already-built canonical triangle.

* [x] Add PyO3 binding for canonical chain ladder calculation.
* [x] Add Python `ChainLadder` class.
* [x] Ensure Python does not duplicate chain ladder math already implemented in Rust.
* [x] Accept a mapped `Triangle` created by `Triangle.from_frame(...)`.
* [x] Add `ReserveResult.summary()`.
* [x] Add `ReserveResult.diagnostics()`.
* [x] Add `ReserveResult.to_arrow()`.
* [x] Add `ReserveResult.to_pandas()`.
* [x] Add `ReserveResult.audit_trail()`.
* [x] Preserve mapping metadata and source/canonical column names in the audit trail.
* [x] Add Python tests for mapping, validation, and result shape.
* [x] Add Python test comparing result values to the Rust golden fixture.
* [x] Add notebook smoke test that loads custom-column triangle data and runs chain ladder.
* [x] Update `CHANGELOG.md`.
* [x] Update `contracts/DATA_CONTRACTS.md` if result schema changes.
* [x] Update `docs/actuarial/model-governance.md` if calculation behavior changes.

---

## Slice 2.1 — Rust-backed TriangleBuilder MVP

Goal: build actuarial `TriangleSet` objects from raw claim/event records, not
only from pre-shaped triangle data. Python adapts dataframes and source-column
mappings; `rustuary-core` owns deterministic date bucketing, grouping,
aggregation, and cumulative conversion.

### Completed Python configuration surface

* [x] Add Python `TriangleDefinition` concept.
* [x] Add Python `SegmentDefinition` concept for ordered drill-down dimensions.
* [x] Support `origin_date`.
* [x] Support `development_date`.
* [x] Support `amount`.
* [x] Support `measure`.
* [x] Support `aggregation`.
* [x] Support `bucket_months`.
* [x] Support `output_kind`.
* [x] Support `portfolio_id`.
* [x] Support ordered `segments`.
* [x] Validate `bucket_months` is an integer between `1` and `12` at the
  Python configuration boundary.
* [x] Add Python `TriangleBuilder` validation shell.
* [x] Add Python adapter validation errors that reference source column names
  and canonical field names.
* [x] Add Python tests for `TriangleDefinition`, `TriangleBuilder`, count
  definitions without `amount`, invalid `bucket_months`, and missing required
  mapped columns.

### Rust engine boundary

* [x] Define canonical Rust raw claim/event build-record input after Python
  dataframe adaptation.
* [x] Define the Rust build request from `TriangleDefinition`, or a validated
  Rust mirror of it.
* [x] Implement raw claim/event triangle construction in `rustuary-core`.
* [x] Return `TriangleSet`, `TriangleKey`, and build diagnostics from the Rust
  construction engine.
* [x] Expose the Rust triangle-construction engine through PyO3 for Python
  builders.
* [x] Keep Python `TriangleBuilder` as the dataframe adapter and bridge, not
  the owner of date bucketing, aggregation, grouping, or cumulative conversion
  semantics.
* [x] Design `TriangleSet` construction so per-key work can be parallelized
  inside `rustuary-core` later without changing the Python API.
* [x] Do not expose a Python parallel-execution option until profiling or an
  actuary workflow shows a need.

### Rust TriangleSet and keys

* [x] Add Rust `TriangleSet` concept for multiple triangles built from one
  dataset.
* [x] Add Rust `TriangleKey` or equivalent grouping key with:

  * [x] `portfolio_id`
  * [x] ordered `segments`
  * [x] `measure`

* [x] Preserve ordered segment metadata in `TriangleKey`.
* [x] Add a method such as `TriangleKey.display_path()` that derives a folder
  path from `portfolio_id + ordered segments`.
* [x] Do not persist `segment_path` as independent canonical truth.

### Date resolution in Rust

* [x] Support monthly triangles with `bucket_months=1`.
* [x] Support quarterly triangles with `bucket_months=3`.
* [x] Support half-year triangles with `bucket_months=6`.
* [x] Support annual triangles with `bucket_months=12`.
* [x] Return a clear unsupported-bucket error for other `bucket_months` values
  that pass Python's generic `1..12` configuration validation.
* [x] Calculate origin period from `origin_date`.
* [x] Calculate development age from `origin_date` and `development_date`.
* [x] Ensure negative development ages produce clear validation errors.
* [x] Ensure invalid Rust record dates produce clear validation errors.
* [x] Ensure missing adapter date values produce clear validation errors in the
  Python/PyO3 bridge.

### Aggregation and conversion in Rust

* [x] Support `sum` aggregation for monetary triangles.
* [x] Support `count` aggregation for simple count triangles.
* [x] Defer `count_distinct` unless needed by the first actuary workshop; no
  contract or API surface is introduced for it in the MVP.
* [x] Build incremental triangle cells from raw records.
* [x] Convert incremental output to cumulative output when requested.
* [x] Record whether cumulative conversion was applied.

### Portfolio and segments

* [x] Map source reserving-class values to canonical `portfolio_id` in the
  Python adapter before calling Rust.
* [x] Resolve source-column and constant values for `measure`, `portfolio_id`,
  ordered `segments`, `valuation_date`, and `currency`.
* [x] Support zero or more ordered segment mappings.
* [x] Build one triangle per `portfolio_id + segments + measure`.

### Python bridge API

* [x] Add Python `TriangleSet` wrapper around Rust output.
* [x] Add `TriangleBuilder.from_frame(data, definition=...)`.
* [x] Have `TriangleBuilder.from_frame(...)` return a `TriangleSet`.
* [x] Add `TriangleSet.keys()`.
* [x] Add `TriangleSet.get(...)`.
* [x] Add `TriangleSet.tree()` or equivalent display helper.
* [x] Add audit metadata showing the full `TriangleDefinition`.

### Tests and fixtures

* [x] Add fixture with raw claim/event records.
* [x] Add deterministic expected-output fixture for the Rust-built
  `TriangleSet`.
* [x] Add Rust test for `bucket_months=1`.
* [x] Add Rust test for `bucket_months=3`.
* [x] Add Rust test for `bucket_months=6`.
* [x] Add Rust test for `bucket_months=12`.
* [x] Add Rust test with only `portfolio_id` and no segments.
* [x] Add Rust test with ordered segments.
* [x] Add Rust or Python bridge test that display path is derived from segment
  order.
* [x] Add Rust or Python bridge test that changing segment order changes
  display tree order but not source data.
* [x] Add Rust test for `sum` aggregation.
* [x] Add Rust test for `count` aggregation.
* [x] Add Rust test for incremental output.
* [x] Add Rust test for cumulative output.
* [x] Add Rust test for invalid date ordering.
* [x] Add Python bridge test for `TriangleBuilder.from_frame(...)`.
* [x] Add Python bridge test for full `TriangleDefinition` audit metadata.

---

## Slice 2.2 — Actuary notebook workbench

Goal: validate the full workflow with actuaries before building the full application UI.

* [ ] Create or update `notebooks/01_triangle_building_workbench.ipynb`.
* [ ] Load example raw claims data with custom column names.
* [ ] Configure `TriangleDefinition`.
* [ ] Map `accident_date` or equivalent source column to `origin_date`.
* [ ] Map `reporting_date`, `payment_date`, or equivalent source column to `development_date`.
* [ ] Map reserving-class source column to `portfolio_id`.
* [ ] Configure optional ordered segments.
* [ ] Configure `bucket_months` as `1`, `3`, `6`, or `12`.
* [ ] Build a `TriangleSet`.
* [ ] Display available triangle keys.
* [ ] Display a derived folder/tree view from `portfolio_id + ordered segments`.
* [ ] Select one triangle.
* [ ] Show incremental triangle.
* [ ] Show cumulative triangle.
* [ ] Run chain ladder on the selected triangle.
* [ ] Display latest diagonal.
* [ ] Display link ratios.
* [ ] Display selected factors.
* [ ] Display CDFs including tail.
* [ ] Display ultimates and reserves.
* [ ] Display audit trail including mappings and triangle definition.
* [ ] Export summary and diagnostics to CSV or Excel.
* [ ] Capture actuary feedback in `docs/product/actuary-feedback.md`.

---

## Slice 2.3 — Minimal SvelteKit playground design

Goal: prepare a small visual playground, but do not build the full platform yet.

* [x] Add ADR: `docs/adr/0007-use-browser-wasm-only-for-playground.md`.
* [x] State that browser WASM is allowed only for playground/demo calculations.
* [x] State that production calculations must run server-side through the backend/job engine.
* [x] Design a simple `apps/workbench` playground route.
* [x] Add CSV upload design.
* [x] Add column-mapping UI design.
* [x] Add `portfolio_id` mapping control.
* [x] Add ordered segment selection control.
* [x] Add bucket size selector for `1`, `3`, `6`, and `12` months.
* [x] Add incremental/cumulative output toggle.
* [x] Add derived folder/tree preview.
* [x] Add triangle preview.
* [x] Add chain ladder result preview.
* [x] Mark the playground as non-production and not audit-controlled.

Implementation of this slice should wait until the notebook workflow has been reviewed with actuaries.

---

## Slice 3 — Additional deterministic reserving methods

Goal: extend the deterministic actuarial method library after the chain ladder workflow is trusted.

* [ ] Add a priori / expected loss method.
* [ ] Add exposure vectors.
* [ ] Add expected loss ratios.
* [ ] Add Bornhuetter-Ferguson.
* [ ] Add Cape Cod.
* [ ] Reuse development patterns from chain ladder where appropriate.
* [ ] Add candidate method result model.
* [ ] Add selected result model.
* [ ] Add accident-year / origin-period method selection.
* [ ] Add weighted blending between candidate methods.
* [ ] Preserve selection rationale in the result metadata.
* [ ] Add diagnostics for each method.
* [ ] Add golden fixture tests for each method.
* [ ] Update model-governance docs.
* [ ] Update `CHANGELOG.md`.

---

## Slice 4 — Optional SvelteKit playground implementation

Goal: give actuaries a simple visual playground after the notebook workflow is validated.

* [ ] Add a playground route in `apps/workbench`.
* [ ] Use Svelte MCP guidance when working on Svelte/SvelteKit code.
* [ ] Add CSV upload using browser-only state or temporary local state.
* [ ] Add column-mapping controls.
* [ ] Add `portfolio_id` selector.
* [ ] Add ordered segment selector.
* [ ] Add bucket size selector.
* [ ] Add cumulative/incremental toggle.
* [ ] Render derived folder/tree preview.
* [ ] Render incremental triangle preview.
* [ ] Render cumulative triangle preview.
* [ ] Run chain ladder through one of:

  * [ ] browser WASM wrapper, or
  * [ ] temporary local API.
* [ ] Show summary results.
* [ ] Show diagnostics.
* [ ] Clearly label playground output as non-production.
* [ ] Do not add auth, RBAC, job queue, PostgreSQL, or object storage in this slice.

---

## Slice 5 — Platform integration

Goal: prepare the production architecture after the actuarial workflow is validated.

* [ ] Finalize OpenAPI run-submission contract, including column mapping metadata.
* [ ] Include `TriangleDefinition` in run-submission contract.
* [ ] Include `portfolio_id` and ordered `segments` in result metadata.
* [ ] Define metadata tables for model runs.
* [ ] Define metadata tables for assumptions.
* [ ] Define metadata tables for audit events.
* [ ] Define object-store path conventions.
* [ ] Define job lifecycle and retry semantics.
* [ ] Define backend validation flow for uploaded data.
* [ ] Define persisted triangle-definition versions.
* [ ] Define result storage in Parquet/Arrow.
* [ ] Define export workflow.
* [ ] Define SvelteKit import wizard implementation plan.
* [ ] Define Go backend orchestration boundaries.
* [ ] Define Rust engine execution boundary.
* [ ] Update ADRs as technology choices are finalized.

---

## Non-goals for the first vertical slice

Do not implement these until the notebook workflow and chain ladder binding are validated:

* [ ] Full Go backend.
* [ ] PostgreSQL model-run store.
* [ ] Job queue.
* [ ] Object-store integration.
* [ ] Auth and RBAC.
* [ ] Full SvelteKit workbench.
* [ ] IFRS 17 engine.
* [ ] Stochastic reserving.
* [ ] Production audit workflow.
* [ ] Multi-user review and approval workflow.
* [ ] Full ResQ replacement features.

---

## First usable milestone

The first business-usable milestone is complete when:

* [ ] An actuary can open a notebook.
* [ ] Load raw claim/event data.
* [ ] Map custom columns.
* [ ] Choose `portfolio_id`.
* [ ] Choose optional ordered segments.
* [ ] Choose development resolution with `bucket_months`.
* [ ] Build incremental and cumulative triangles.
* [ ] Select one triangle from the generated `TriangleSet`.
* [ ] Run chain ladder through the Rust engine via Python.
* [ ] Inspect summary and diagnostics.
* [ ] Export results.
* [ ] Review the audit trail showing mappings, triangle definition, and assumptions.

At that point, start collecting actuary feedback before building the full UI or backend platform.
