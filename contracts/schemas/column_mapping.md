# Column mapping schema

Column mappings adapt external user data into Rustuary canonical contracts.

## ClaimsMapping

| Field | Type | Required | Description |
|---|---|---:|---|
| `origin` | string | yes | Source column for canonical `origin_period`. |
| `development` | string | yes | Source column for canonical `development_age`. |
| `value` | string | yes | Source column for canonical `amount`. |
| `portfolio_id` | string/object | no | Source column or constant for canonical `portfolio_id`, the main reserving class / actuarial reserving unit. |
| `segments` | ordered list | no | Optional ordered segment mappings below `portfolio_id`. |
| `portfolio` | string/object | no | Backward-compatible adapter alias for `portfolio_id`; prefer `portfolio_id` in new contracts. |
| `valuation_date` | string/object | no | Source column or constant for canonical `valuation_date`. |
| `measure` | string/object | no | Source column or constant for canonical `measure`. |
| `cumulative` | bool/string | yes | Constant boolean or source column for canonical `is_cumulative`. |
| `currency` | string/object | no | Source column or constant for canonical `currency`. |
| `origin_type` | string | no | Semantic origin basis. |
| `development_unit` | string | no | `months`, `quarters`, or `years`. |

Constant values may be represented in config as:

```yaml
measure:
  const: paid
valuation_date:
  const: 2026-12-31
```

For Python convenience, plain scalar constants are also allowed:

```python
ry.ClaimsMapping(measure="paid", valuation_date="2026-12-31")
```

Python resolves optional string mappings deterministically:

- If the string matches an input column, it is treated as a source column.
- Otherwise, it is treated as a constant.
- Use `{"const": value}` to force constant interpretation when a source column
  has the same name.

The Python triangle adapter always emits `origin_period`, `development_age`,
`amount`, and `is_cumulative`. It includes mapped context fields such as
`portfolio_id`, ordered `segments`, `valuation_date`, `measure`, and `currency`
when supplied.

Segment mappings are ordered and each item has:

| Field | Type | Required | Description |
|---|---|---:|---|
| `name` | string | yes | Canonical segment name, for example `country`, `channel`, or `coverage`. |
| `source` | string/object | yes | Source column or constant segment value. |

Do not map or persist `segment_path` as canonical input. Display paths are
derived from `portfolio_id` plus ordered segment values.

Required source columns must resolve to exactly one input column. Missing or
duplicate columns raise `ColumnMappingError` with the source column, canonical
field, and available input columns.

## ExposureMapping

| Field | Type | Required | Description |
|---|---|---:|---|
| `origin` | string | yes | Source column for canonical `origin_period`. |
| `value` | string | yes | Source column for canonical `amount`. |
| `portfolio_id` | string/object | no | Source column or constant for canonical `portfolio_id`. |
| `segments` | ordered list | no | Optional ordered segment mappings that must align with claims segmentation. |
| `portfolio` | string/object | no | Backward-compatible adapter alias for `portfolio_id`; prefer `portfolio_id` in new contracts. |
| `valuation_date` | string/object | no | Source column or constant for canonical `valuation_date`. |
| `exposure_measure` | string/object | yes | Source column or constant exposure measure. |
| `currency` | string/object | no | Source column or constant currency. |

## Persistence rule

Every model run that used non-canonical external input must persist:

- mapping config
- input source URI or file hash when available
- canonical schema version
- validation report

The Python `Triangle` currently persists the mapping config and canonical
schema version in the versioned
[`model_run_metadata.md`](model_run_metadata.md) placeholder. Workflow-owned
evidence is added when model-run orchestration is implemented.
