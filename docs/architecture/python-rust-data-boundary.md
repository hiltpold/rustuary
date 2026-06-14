# Python to Rust data boundary

## Purpose

Rustuary uses a strict boundary between business-friendly dataframe inputs and the Rust actuarial core.
The Python interface accepts messy user dataframes. The Rust engine receives canonical, validated domain types.

## Principle

```text
user dataframe / CSV / parquet
        ↓
Python adapter with column mapping
        ↓
canonical Arrow table or canonical Triangle object
        ↓
Rust binding boundary
        ↓
Rust domain types
        ↓
actuarial calculation
```

The Rust core must not depend on pandas, Polars, or user-specific column names.

## Where column mapping belongs

Column mapping belongs at every ingestion boundary:

| Boundary | Needs mapping? | Reason |
|---|---:|---|
| Python `Triangle.from_frame` | yes | Actuaries will use local pandas, Polars, or PyArrow tables with custom names. |
| Python `Exposure.from_frame` | yes | Exposure/premium columns vary by company and line of business. |
| YAML run configs | yes | Production runs must be reproducible without notebook-only code. |
| CLI file import | yes | CSV/Parquet files will not always use canonical names. |
| Go backend import jobs | yes | Uploaded files need server-side validation and metadata capture. |
| SvelteKit import wizard | yes | Business users need to map columns visually before submitting a run. |
| OpenAPI/protobuf contracts | yes | Job submissions need to carry mapping metadata. |
| Model-run metadata | yes | Reproducibility requires knowing how external data became canonical data. |
| Rust core methods | no | Core methods should calculate on already-validated canonical domain types. |

## Python API shape

Support both explicit named arguments and reusable mapping objects.

```python
triangle = ry.Triangle.from_frame(
    claims,
    origin="AY",
    development="dev_month",
    value="paid_loss",
    cumulative=True,
    portfolio="segment",
    valuation_date="valuation_dt",
    measure="paid",
)
```

Equivalent reusable mapping:

```python
mapping = ry.ClaimsMapping(
    origin="AY",
    development="dev_month",
    value="paid_loss",
    cumulative=True,
    portfolio="segment",
    valuation_date="valuation_dt",
    measure="paid",
)

triangle = ry.Triangle.from_frame(claims, mapping=mapping)
```

## Config shape

```yaml
triangle:
  input: data/examples/custom_claims.csv
  format: csv
  mapping:
    origin: AY
    development: dev_month
    value: paid_loss
    portfolio: segment
    valuation_date: valuation_dt
    measure: paid
    cumulative: true
    currency: CHF
```

## Implementation guidance

1. Normalize first: select/rename source columns into canonical names.
2. Cast second: parse dates, periods, integers, booleans, and numeric values.
3. Validate third: check required fields, duplicates, monotonic cumulative values, and missing cells.
4. Convert fourth: create canonical `Triangle`/`Exposure` Rust-domain inputs.
5. Calculate last: call Rust methods only after validation succeeds.

## Error style

Prefer errors that show both the user-facing source field and canonical field:

```text
ColumnMappingError: required canonical field `origin_period` is mapped from source column `AY`,
but `AY` is not present in the dataframe. Available columns: accident_year, dev_month, paid_loss.
```

## Non-goals

- Do not make Rust core methods accept arbitrary dataframe schemas.
- Do not duplicate actuarial calculations in Python.
- Do not hide mapping choices when persisting model runs.
