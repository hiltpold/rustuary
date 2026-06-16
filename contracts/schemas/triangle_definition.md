# TriangleDefinition Schema

`TriangleDefinition` describes how raw claim or event records are grouped and
transformed into one or more canonical actuarial triangles.

The Rust engine consumes validated canonical triangles. It does not read raw
records or infer portfolio and segmentation semantics. Adapters, import jobs,
and workflow services use `TriangleDefinition` before invoking the engine.

## Required Concepts

| Field | Type | Required | Notes |
|---|---|---:|---|
| `triangle_definition_id` | string | yes | Stable identifier for audit and reproducibility. |
| `schema_version` | string | yes | Version of this logical schema. |
| `portfolio_id` | string/object | yes | Source column or constant for the main reserving class / actuarial reserving unit. |
| `segments` | ordered list | no | Optional ordered drill-down dimensions below `portfolio_id`. Empty list means no segments. |
| `origin` | string/object | yes | Source column or rule for canonical `origin_period`. |
| `development` | string/object | yes | Source column or rule for canonical `development_age`. |
| `value` | string/object | yes | Source column or rule for canonical `amount`. |
| `measure` | string/object | yes | Source column or constant measure such as `paid` or `incurred`. |
| `cumulative` | bool/string/object | yes | Constant or source field for canonical `is_cumulative`. |
| `valuation_date` | string/object | no | Source column or constant valuation date. |
| `currency` | string/object | no | Source column or constant currency. Required when monetary data can cross currencies. |
| `origin_type` | string | no | `accident_year`, `underwriting_year`, `report_year`, etc. |
| `development_unit` | string | no | `months`, `quarters`, or `years`. |

Segment entries are ordered:

| Field | Type | Required | Notes |
|---|---|---:|---|
| `name` | string | yes | Canonical segment name, for example `country`, `channel`, or `coverage`. |
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
