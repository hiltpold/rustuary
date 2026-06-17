# TriangleDefinition Schema

`TriangleDefinition` describes how raw claim or event records are grouped and
transformed into one or more canonical actuarial triangles.

The Rust engine consumes validated canonical triangles and, for raw
triangle-building workflows, typed canonical claim/event build records after
adapters have resolved source columns and constants. It does not read
dataframes or external source column names directly.

Adapters pass Rust a validated build request that mirrors the build-semantic
fields from `TriangleDefinition`: `triangle_definition_id`, `schema_version`,
`aggregation`, `bucket_months`, `output_kind`, and ordered segment names. Source
column names remain adapter metadata.

## Required Concepts

| Field | Type | Required | Notes |
|---|---|---:|---|
| `triangle_definition_id` | string | yes | Stable identifier for audit and reproducibility. |
| `schema_version` | string | yes | Version of this logical schema. |
| `portfolio_id` | string/object | yes | Source column or constant for the main reserving class / actuarial reserving unit. |
| `segments` | ordered list | no | Optional ordered drill-down dimensions below `portfolio_id`. Empty list means no segments. |
| `origin_date` | string | yes | Source date column used to derive canonical `origin_period`. |
| `development_date` | string | yes | Source date column used with `origin_date` to derive canonical `development_age`. |
| `amount` | string | no | Source amount column for canonical `amount`. Required for `sum`; omitted for simple row-count triangles. |
| `measure` | string/object | yes | Source column or constant measure such as `paid` or `incurred`. |
| `aggregation` | string | yes | Aggregation used to build cells. MVP values: `sum`, `count`. `count` means one input row contributes one event. |
| `bucket_months` | integer | yes | Development bucket size in months, between `1` and `12`. |
| `output_kind` | string | yes | `incremental` or `cumulative` output triangle. |
| `valuation_date` | string/object | no | Source column or constant valuation date. |
| `currency` | string/object | no | Source column or constant currency. Required when monetary data can cross currencies. |

Segment entries are ordered:

| Field | Type | Required | Notes |
|---|---|---:|---|
| `name` | string | yes | Canonical segment name, for example `country`, `channel`, or `coverage`. Names must be non-empty and unique within a definition. |
| `source` | string/object | yes | Source column or constant segment value. |

## Grouping Key

The canonical grouping key is:

```text
portfolio_id + ordered segment values + measure
```

`portfolio_id` is the main reserving class / actuarial reserving unit.
`segments` are optional ordered drill-down dimensions. The order in this schema
is the order used for deterministic grouping keys and display paths.

`segment_path` is not canonical input. UI folder paths are derived from
structured values:

```text
portfolio_id / segment_1_value / segment_2_value / ...
```

## Traceability

Every persisted triangle must retain enough metadata to trace it back to:

- `triangle_definition_id`
- `schema_version`
- source artifact identifier or hash when available
- column mapping or source rules
- ordered segment definitions
- validation report

Changing segment order changes display-tree order and deterministic grouping
metadata. It must be reviewed as a contract change even when the source records
are unchanged.
