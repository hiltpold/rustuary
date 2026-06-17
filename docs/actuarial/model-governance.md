# Actuarial model governance

This document records how actuarial methods, assumptions, diagnostics, and results are governed in the project.

## Scope

Covers deterministic reserving methods such as:

- Chain Ladder
- Bornhuetter-Ferguson
- Cape Cod
- A priori / expected loss
- Tail factor selection
- Accident-year method selection
- Weighted method blending

## Method implementation checklist

A new or changed method should include:

- Formula description.
- Required inputs.
- Validation rules.
- Diagnostics and intermediate values.
- Unit tests.
- Golden tests.
- Tolerance policy for floating-point output.
- Known limitations.
- Example Python usage once bindings exist.

## Assumption governance

Assumptions should be explicit and serializable:

- Selected development factors.
- Tail factors.
- Expected loss ratios.
- Exposure values and adjustments.
- Link-ratio exclusions.
- Origin-year selections.
- Weighted blends.
- Override reasons.

Assumptions must not be hidden in code paths or defaults that cannot be inspected.

## Diagnostics policy

Results should include enough intermediate values to explain the final selected reserve:

- Latest diagonal.
- Link ratios.
- Selected factors.
- CDFs.
- Reported/unreported percentages.
- Prior ultimate where applicable.
- Candidate ultimates by method.
- Selected ultimate and reserve.
- Selection components and rationale.

## Model change log

Record material model changes here or link to relevant ADRs and changelog entries.

| Date | Change | Impact | Validation |
|---|---|---|---|
| 2026-06-17 | Added Python result audit evidence | No formula or selection change; Python `ReserveResult` now exposes summary, diagnostics, and source-to-canonical input lineage for reproducibility review | Python audit-trail tests, notebook smoke test, and shared golden fixture comparison |
| 2026-06-16 | Documented golden fixture tolerance policy | No model formula change; shared chain-ladder golden fixture now covers every origin row and documents `1e-9` absolute tolerance | Fixture-backed Rust golden test |
| 2026-06-16 | Added chain-ladder ultimate, reserve, and origin diagnostics | Origin projections now retain labels, latest observed values, CDF components, selected ultimate, reserve, and reject non-finite projection arithmetic | Unit projection validation tests and expanded golden chain-ladder diagnostics with `1e-9` absolute tolerance |
| 2026-06-16 | Added typed CDF calculation including tail | CDFs now expose per-development-age diagnostics with remaining selected-factor product, fixed tail factor, and final CDF | Unit validation tests and golden CDF test with `1e-9` absolute tolerance |
| 2026-06-16 | Added fixed tail factor interface | Tail assumptions are now typed, positive finite values with optional non-blank rationale retained in diagnostics; default remains `1.0` for no tail beyond final observed age | Unit validation tests and golden fixed-tail projection test with `1e-9` absolute tolerance |
| 2026-06-15 | Added selected-factor overrides and link-ratio exclusions | Exclusions alter the observations included in factor calculation; overrides replace validated calculated factors. Both require explicit rationales and remain visible in diagnostics | Unit validation tests and golden adjusted-factor test with `1e-9` absolute tolerance |
| 2026-06-15 | Added simple-average development factor selection | Selects each age-to-age factor as the arithmetic mean of observed individual link ratios without exclusions or overrides | Unit and golden comparison tests with `1e-9` absolute tolerance |
| 2026-06-15 | Added volume-weighted development factor selection | Selects each age-to-age factor as the ratio of cumulative aggregate sums without exclusions or overrides | Unit and golden tests with `1e-9` absolute tolerance |
| 2026-06-15 | Added typed link-ratio diagnostics | Exposes each observed cumulative age-to-age ratio without selecting development factors | Unit and golden tests with `1e-9` absolute tolerance |
| 2026-06-15 | Added typed latest-diagonal extraction | Exposes origin and development labels for the last observed cell without changing values or basis | Unit and golden tests with `1e-9` absolute tolerance |
| 2026-06-15 | Added cumulative/incremental triangle conversion | Deterministic row-wise basis conversion; no reserve selection change | Unit and golden round-trip tests with `1e-9` absolute tolerance |
| 2026-06-14 | Initial governance scaffold | No production model impact | Documentation only |


## Data mapping and reproducibility

When input data uses non-canonical column names, the mapping from source columns to canonical fields is part of the model-run evidence. A reserving result is not reproducible unless the mapping, input source, canonical schema version, assumptions, engine version, and validation report are retained.

Every persisted triangle must be traceable to the `TriangleDefinition` that
produced it. That evidence includes the main `portfolio_id` reserving class,
ordered segment definitions, source mapping rules, measure, valuation context,
and validation report. Display paths are derived from `portfolio_id` plus
ordered segment values and are not independent canonical input.

The Python `Triangle` preserves the mapping and canonical schema version in a
model-run metadata placeholder. The Python `ReserveResult.audit_trail()`
combines that input evidence with result summaries, calculation diagnostics,
and source-to-canonical column lineage for notebook review. Workflow-owned
evidence such as source hashes, assumption versions, engine versions, and
validation reports must still be added before a persisted model run is
considered reproducible.

Mapping changes that alter calculated results must be treated as behavior changes and covered by tests, changelog notes, and review.
