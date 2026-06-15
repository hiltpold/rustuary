import pytest
import pyarrow as pa

from rustuary import ClaimsMapping, Triangle


def test_triangle_from_frame_stores_required_mapping():
    triangle = Triangle.from_frame(
        data=[{"accident_year": 2024, "development_month": 12, "paid": 100.0}],
        origin="accident_year",
        development="development_month",
        value="paid",
    )

    assert triangle.origin == "accident_year"
    assert triangle.development == "development_month"
    assert triangle.value == "paid"
    assert isinstance(triangle.data, pa.Table)
    assert triangle.data.to_pylist() == [
        {"accident_year": 2024, "development_month": 12, "paid": 100.0}
    ]


def test_triangle_from_frame_stores_complete_mapping():
    claims = [{"AY": 2024, "dev_month": 12, "paid_loss": 100.0}]

    triangle = Triangle.from_frame(
        claims,
        origin="AY",
        development="dev_month",
        value="paid_loss",
        cumulative="is_cumulative",
        portfolio="segment",
        valuation_date={"const": "2026-12-31"},
        measure={"const": "paid"},
        currency={"const": "CHF"},
        origin_type="accident_year",
        development_unit="months",
    )

    assert triangle.data.to_pylist() == claims
    assert triangle.cumulative == "is_cumulative"
    assert triangle.portfolio == "segment"
    assert triangle.valuation_date == {"const": "2026-12-31"}
    assert triangle.measure == {"const": "paid"}
    assert triangle.currency == {"const": "CHF"}
    assert triangle.origin_type == "accident_year"
    assert triangle.development_unit == "months"


def test_triangle_from_frame_uses_claims_mapping_validation():
    with pytest.raises(ValueError, match="development_unit"):
        Triangle.from_frame(
            [],
            origin="AY",
            development="dev_month",
            value="paid_loss",
            development_unit="weeks",
        )


def test_triangle_from_frame_accepts_reusable_mapping():
    claims = [{"AY": 2024, "dev_month": 12, "paid_loss": 100.0}]
    mapping = ClaimsMapping(
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

    triangle = Triangle.from_frame(claims, mapping=mapping)

    assert triangle.data.to_pylist() == claims
    assert triangle.origin == mapping.origin
    assert triangle.development == mapping.development
    assert triangle.value == mapping.value
    assert triangle.cumulative == mapping.cumulative
    assert triangle.portfolio == mapping.portfolio
    assert triangle.valuation_date == mapping.valuation_date
    assert triangle.measure == mapping.measure
    assert triangle.currency == mapping.currency
    assert triangle.origin_type == mapping.origin_type
    assert triangle.development_unit == mapping.development_unit


def test_triangle_from_frame_rejects_mixed_mapping_forms():
    mapping = ClaimsMapping(origin="AY", development="dev_month", value="paid_loss")

    with pytest.raises(TypeError, match="cannot be combined.*origin"):
        Triangle.from_frame([], mapping=mapping, origin="AY")


def test_triangle_from_frame_reports_all_missing_required_named_fields():
    with pytest.raises(TypeError, match="origin, development, value"):
        Triangle.from_frame([])


def test_triangle_from_frame_rejects_wrong_mapping_type():
    with pytest.raises(TypeError, match="mapping must be a ClaimsMapping"):
        Triangle.from_frame([], mapping="claims")


def test_triangle_from_frame_preserves_pyarrow_table():
    table = pa.table({"AY": [2024], "dev_month": [12], "paid_loss": [100.0]})

    triangle = Triangle.from_frame(
        table,
        origin="AY",
        development="dev_month",
        value="paid_loss",
    )

    assert triangle.data is table


def test_triangle_from_frame_converts_pyarrow_record_batch():
    batch = pa.record_batch(
        [[2024], [12], [100.0]],
        names=["AY", "dev_month", "paid_loss"],
    )

    triangle = Triangle.from_frame(
        batch,
        origin="AY",
        development="dev_month",
        value="paid_loss",
    )

    assert isinstance(triangle.data, pa.Table)
    assert triangle.data.to_pylist() == [{"AY": 2024, "dev_month": 12, "paid_loss": 100.0}]


def test_triangle_from_frame_converts_pyarrow_record_batch_reader():
    schema = pa.schema(
        [
            ("AY", pa.int64()),
            ("dev_month", pa.int64()),
            ("paid_loss", pa.float64()),
        ]
    )
    reader = pa.RecordBatchReader.from_batches(
        schema,
        [
            pa.record_batch(
                [[2024], [12], [100.0]],
                schema=schema,
            )
        ],
    )

    triangle = Triangle.from_frame(
        reader,
        origin="AY",
        development="dev_month",
        value="paid_loss",
    )

    assert triangle.data.to_pylist() == [{"AY": 2024, "dev_month": 12, "paid_loss": 100.0}]


def test_triangle_from_frame_converts_pandas_dataframe_without_index():
    pandas = pytest.importorskip("pandas")
    frame = pandas.DataFrame(
        {"AY": [2024], "dev_month": [12], "paid_loss": [100.0]},
        index=["row-1"],
    )

    triangle = Triangle.from_frame(
        frame,
        origin="AY",
        development="dev_month",
        value="paid_loss",
    )

    assert triangle.data.column_names == ["AY", "dev_month", "paid_loss"]
    assert triangle.data.schema.metadata is None
    assert triangle.data.to_pylist() == [{"AY": 2024, "dev_month": 12, "paid_loss": 100.0}]


def test_triangle_from_frame_converts_polars_dataframe():
    polars = pytest.importorskip("polars")
    frame = polars.DataFrame({"AY": [2024], "dev_month": [12], "paid_loss": [100.0]})

    triangle = Triangle.from_frame(
        frame,
        origin="AY",
        development="dev_month",
        value="paid_loss",
    )

    assert isinstance(triangle.data, pa.Table)
    assert triangle.data.to_pylist() == [{"AY": 2024, "dev_month": 12, "paid_loss": 100.0}]


def test_triangle_from_frame_rejects_unsupported_data():
    with pytest.raises(TypeError, match="data must be"):
        Triangle.from_frame(
            object(),
            origin="AY",
            development="dev_month",
            value="paid_loss",
        )
