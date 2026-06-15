# Data contracts

This document describes canonical logical schemas shared by the Rust engine, Python bindings, backend services, UI, and storage layers.

The schemas are intentionally logical. Physical representations may be Arrow, Parquet, JSON, CSV, protobuf, or database tables.

## Claims triangle input, long format

Preferred input format for claims data:

| Field | Type | Required | Notes |
|---|---|---:|---|
| `portfolio_id` | string | yes | Reserving segment or portfolio. |
| `valuation_date` | date | yes | Date of the run or data cut. |
| `origin_period` | string/int/date | yes | Accident year, underwriting year, report year, etc. |
| `development_age` | int | yes | Development age in months or agreed unit. |
| `calendar_period` | string/int/date | no | Optional derived calendar period. |
| `measure` | string | yes | `paid`, `incurred`, `reported_count`, etc. |
| `amount` | decimal/float | yes | Claim amount or count. |
| `currency` | string | no | Required for monetary cross-currency data. |
| `is_cumulative` | bool | yes | Whether `amount` is cumulative or incremental. |

### Canonical Rust triangle

Adapters group the long-format claims contract into one homogeneous calculation
slice before constructing the Rust `Triangle`. The core representation contains:

- unique, strictly increasing `OriginPeriod` row labels
- unique, strictly increasing `DevelopmentAge` column labels
- one explicit cumulative or incremental basis
- a rectangular row-major amount matrix with finite observed values
- trailing unobserved cells only; gaps between observed cells are invalid

Portfolio, valuation date, measure, and currency remain workflow context for the
homogeneous slice rather than being repeated in every dense matrix cell.

Basis conversion is row-wise and preserves the canonical axes and trailing
unobserved cells:

- cumulative from incremental: `C[j] = sum(X[k], k = 0..j)`
- incremental from cumulative: `X[0] = C[0]` and `X[j] = C[j] - C[j - 1]`

Negative incremental values are valid because recoveries and corrections may
reduce cumulative claims. Conversion fails if floating-point arithmetic would
produce a non-finite value.

Latest-diagonal extraction returns one entry per origin period in origin-axis
order. Each entry includes the origin period, development age, their zero-based
matrix indices, and the latest observed value. The value remains in the
triangle's current basis; callers requiring cumulative latest values must
convert the triangle first.

Link-ratio calculation emits one diagnostic for each origin row where adjacent
development cells are both observed. For cumulative values `C`, the ratio is
`C[i, j + 1] / C[i, j]`. Diagnostics include origin and development labels,
matrix indices, both source values, and the ratio. Incremental triangles must be
converted explicitly. Zero denominators are errors; finite negative ratios are
retained for review rather than silently excluded.

## Exposure input

| Field | Type | Required | Notes |
|---|---|---:|---|
| `portfolio_id` | string | yes | Must align with claims. |
| `valuation_date` | date | yes | Must align with claims. |
| `origin_period` | string/int/date | yes | Must align with triangle origin. |
| `exposure_measure` | string | yes | `earned_premium`, `onlevel_premium`, `policy_count`, etc. |
| `amount` | decimal/float | yes | Exposure amount. |
| `currency` | string | no | Required for monetary exposure. |

## Assumption input

| Field | Type | Required | Notes |
|---|---|---:|---|
| `assumption_set_id` | string | yes | Stable ID for reproducibility. |
| `method` | string | yes | Method name or candidate ID. |
| `origin_period` | string/int/date | no | Optional origin-specific assumption. |
| `development_age` | int | no | Optional age-specific assumption. |
| `name` | string | yes | Example: `tail_factor`, `expected_loss_ratio`. |
| `value` | scalar/json | yes | Assumption value. |
| `rationale` | string | no | Human explanation. |

## Result output, by origin period

| Field | Type | Required | Notes |
|---|---|---:|---|
| `model_run_id` | string | yes | Unique run ID. |
| `portfolio_id` | string | yes | Reserving segment. |
| `valuation_date` | date | yes | Run date. |
| `origin_period` | string/int/date | yes | Origin period. |
| `latest_observed` | decimal/float | yes | Latest cumulative value. |
| `selected_ultimate` | decimal/float | yes | Final selected ultimate. |
| `selected_reserve` | decimal/float | yes | Ultimate less latest observed. |
| `selected_method` | string | yes | Method or blend label. |
| `selection_rationale` | string | no | Human explanation. |
| `diagnostics` | json | yes | Auditable intermediate values. |

## Column mapping contract

Rustuary has canonical logical schemas, but user data rarely arrives with canonical column names.
Column mapping is therefore part of the adapter layer contract, not the actuarial core.

The canonical claims fields remain:

```text
portfolio_id
valuation_date
origin_period
development_age
measure
amount
is_cumulative
```

A user-facing mapping describes how external columns map into canonical fields:

| Mapping field | Type | Required | Notes |
|---|---|---:|---|
| `origin` | string | yes | Source column for accident/underwriting/report year. |
| `development` | string | yes | Source column for development age. |
| `value` | string | yes | Source column for claim amount/count. |
| `portfolio` | string | no | Source column or constant portfolio value. |
| `valuation_date` | string | no | Source column or constant valuation date. |
| `measure` | string | no | Source column or constant measure, for example `paid`. |
| `cumulative` | bool/string | yes | Constant or source column indicating cumulative vs incremental. |
| `currency` | string | no | Source column or constant currency. |
| `origin_type` | string | no | `accident_year`, `underwriting_year`, `report_year`, etc. |
| `development_unit` | string | no | `months`, `quarters`, `years`. |

Example YAML mapping:

```yaml
claims_mapping:
  origin: AY
  development: dev_month
  value: paid_loss
  portfolio: segment
  valuation_date:
    const: 2026-12-31
  measure:
    const: paid
  cumulative: true
  currency:
    const: CHF
  origin_type: accident_year
  development_unit: months
```

The executable example pair is
[`data/examples/paid_claims_custom_columns.csv`](../data/examples/paid_claims_custom_columns.csv)
and
[`contracts/examples/claims_mapping.yaml`](examples/claims_mapping.yaml).

Rules:

- Rust core methods consume canonical `Triangle`, `Exposure`, and assumption types only.
- Python, CLI, backend import jobs, and UI import wizards may apply column mappings.
- Python optional string mappings use a matching input column when present and
  otherwise materialize the string as a constant. `{"const": value}` always
  means a constant.
- Mapping must be included in model-run metadata when it affects a persisted run.
- Mapping errors must be reported using user column names and canonical field names.
- Mapped data must be validated before calculation.


## Compatibility rule

Any change to these logical fields must update:

- This file.
- Relevant OpenAPI/protobuf/schema files.
- Example datasets if affected.
- Changelog if visible to users or downstream systems.
