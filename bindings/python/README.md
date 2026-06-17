# Python bindings

This package provides the actuary-facing Python API around the Rust core.

Current input preparation:

```python
import rustuary as ry

mapping = ry.ClaimsMapping(
    origin="accident_year",
    development="development_month",
    value="paid_claims",
    cumulative=True,
)

triangle = ry.Triangle.from_frame(
    claims,
    mapping=mapping,
)

triangle.data
triangle.model_run_metadata.to_dict()
```

Available objects:

- `Triangle`
- `TriangleDefinition`
- `SegmentDefinition`
- `ChainLadder`
- `ClaimsMapping`
- `ExposureMapping`
- `ModelRunMetadata`
- `ReserveResult`

Chain ladder on a mapped triangle:

```python
triangle = ry.Triangle.from_frame(
    claims,
    origin="AY",
    development="dev_month",
    value="paid_loss",
    cumulative=True,
)

model = ry.ChainLadder(tail_factor=1.0)
result = model.fit_predict(triangle)

result.summary()
result.to_arrow()
result.to_pandas()  # requires the pandas extra
result.diagnostics()["selected_factors"]
result.audit_trail()["input"]["column_lineage"]
```

The lower-level dense form remains available for already-shaped triangle data:

```python
result = ry.ChainLadder().fit_predict(
    origin_periods=[2020, 2021, 2022],
    development_ages=[12, 24, 36],
    rows=[
        [100.0, 180.0, 240.0],
        [120.0, 210.0, None],
        [150.0, None, None],
    ],
)

result.summary()
result.to_arrow()
result.to_pandas()  # requires the pandas extra
result.diagnostics()["selected_factors"]
result.audit_trail()["input"]["column_lineage"]
```

Internal binding:

- `rustuary._rust.chain_ladder(...)` accepts canonical dense triangle axes and
  rows, delegates calculation to `rustuary-core`, and returns plain Python
  diagnostics. It is intentionally low-level; business users should use
  `ChainLadder`.

Planned objects:

- `Exposure`
- `BornhuetterFerguson`
- `CapeCod`
- `ExpectedLoss`
- `TriangleBuilder`
- `TriangleSet`
- `ReservingWorkflow`

Raw claim/event triangle definitions:

```python
definition = ry.TriangleDefinition(
    triangle_definition_id="paid-claims-v1",
    origin_date="accident_date",
    development_date="payment_date",
    amount="paid_loss",
    measure={"const": "paid"},
    aggregation="sum",
    bucket_months=12,
    output_kind="cumulative",
    portfolio_id="reserving_class",
    segments=[
        ry.SegmentDefinition(name="country", source="country"),
        ry.SegmentDefinition(name="coverage", source="coverage"),
    ],
)

definition.to_dict()
```

For `aggregation="sum"`, `amount` is required. For `aggregation="count"`,
omit `amount`; each input row contributes one event to the cell count.

The deterministic input-review workflow is available in
[`notebooks/01_chain_ladder_workbench.ipynb`](../../notebooks/01_chain_ladder_workbench.ipynb).
It loads custom-column claims data, reviews the canonical `Triangle`, runs
`ChainLadder`, and captures result diagnostics and audit evidence.

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

For reusable workflows, use a mapping object:

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

The repository example keeps the same mapping in a reviewable YAML file. Install
the `yaml` extra and load it with a safe YAML parser:

```python
from pathlib import Path

import yaml

mapping_document = yaml.safe_load(
    Path("contracts/examples/claims_mapping.yaml").read_text()
)
mapping = ry.ClaimsMapping(**mapping_document["claims_mapping"])
```

Use either `mapping=` or the individual named mapping fields in a call, not both.
For optional fields, a string matching an input column reads that column;
otherwise the string is treated as a constant. Use `{"const": value}` to force
a constant when an input column has the same name.

Missing or duplicate required source columns raise `ColumnMappingError`. The
error identifies both the source column and canonical field and lists the input
columns available for correction.

Every triangle retains a detached, JSON-safe mapping snapshot for later audit
and model-run persistence:

```python
triangle.model_run_metadata.to_dict()
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
