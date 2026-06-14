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

The Python layer accepts user-specific dataframe schemas and maps them into Rustuary canonical contracts before calling Rust.

```python
triangle = ry.Triangle.from_frame(
    claims,
    origin="AY",
    development="dev_month",
    value="paid_loss",
    cumulative=True,
    portfolio="segment",
    valuation_date="2026-12-31",
    measure="paid",
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
    valuation_date="2026-12-31",
    measure="paid",
)

triangle = ry.Triangle.from_frame(claims, mapping=mapping)
```

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
