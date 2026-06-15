# Python bindings

This package will provide the actuary-facing Python API around the Rust core.

Target usage:

```python
import rustuary as ry

triangle = ry.Triangle.from_frame(
    claims,
    origin="accident_year",
    development="development_month",
    value="paid_claims",
    cumulative=True,
)

result = ry.ChainLadder(tail_factor=1.03).fit_predict(triangle)
result.summary()
```

Planned objects:

- `Triangle`
- `Exposure`
- `ChainLadder`
- `BornhuetterFerguson`
- `CapeCod`
- `ExpectedLoss`
- `ReservingWorkflow`
- `ReserveResult`


## Column mapping

The Python layer accepts pandas, Polars, PyArrow, or record-sequence inputs and
converts them to a PyArrow table. Dataframe indexes are not treated as claims
columns. Column mapping then selects and renames source fields into canonical
claims columns such as `origin_period`, `development_age`, `amount`, and
`is_cumulative`. Unmapped source columns are not retained.

Those four calculation fields are always present. Context fields such as
`portfolio_id`, `valuation_date`, `measure`, and `currency` are included when
their mappings are supplied.

```python
triangle = ry.Triangle.from_frame(
    claims,
    origin="AY",
    development="dev_month",
    value="paid_loss",
    cumulative=True,
    portfolio="segment",
    valuation_date={"const": "2026-12-31"},
    measure={"const": "paid"},
    currency={"const": "CHF"},
    origin_type="accident_year",
    development_unit="months",
)
```

For reusable workflows, use a mapping object or YAML config:

```python
mapping = ry.ClaimsMapping(
    origin="AY",
    development="dev_month",
    value="paid_loss",
    cumulative=True,
    portfolio="segment",
    valuation_date={"const": "2026-12-31"},
    measure={"const": "paid"},
)

triangle = ry.Triangle.from_frame(claims, mapping=mapping)
```

Use either `mapping=` or the individual named mapping fields in a call, not both.
For optional fields, a string matching an input column reads that column;
otherwise the string is treated as a constant. Use `{"const": value}` to force
a constant when an input column has the same name.

Mapping belongs in the Python adapter, CLI, backend import job, and UI import wizard. The Rust core receives canonical validated inputs only.

Exposure mappings use the same source-column or constant convention:

```python
mapping = ry.ExposureMapping(
    origin="AY",
    value="earned_premium",
    exposure_measure={"const": "earned_premium"},
    portfolio="segment",
    valuation_date={"const": "2026-12-31"},
    currency={"const": "CHF"},
)
```
