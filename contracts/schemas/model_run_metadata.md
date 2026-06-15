# Model run metadata placeholder

The Python binding records the mapping evidence available when a canonical
triangle is created. This payload is a placeholder for the fuller model-run
audit record that will be owned by workflow and result layers.

## Version 1

```json
{
  "canonical_schema": "claims_triangle",
  "canonical_schema_version": "1",
  "claims_mapping": {
    "origin": "AY",
    "development": "dev_month",
    "value": "paid_loss",
    "cumulative": true,
    "portfolio": "segment",
    "valuation_date": {"const": "2026-12-31"},
    "measure": {"const": "paid"},
    "currency": {"const": "CHF"},
    "origin_type": "accident_year",
    "development_unit": "months"
  }
}
```

Rules:

- `claims_mapping` is a detached snapshot of the mapping used for
  normalization.
- Date and datetime constants use ISO 8601 strings.
- Decimal constants use strings so persistence does not lose precision.
- This placeholder does not contain a model-run ID, source URI or hash,
  validation report, assumptions version, or engine version. Those fields
  belong to the later workflow/result audit record.
