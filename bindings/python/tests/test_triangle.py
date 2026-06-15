import pytest
import pyarrow as pa

from rustuary import ClaimsMapping, ColumnMappingError, Triangle


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
        {
            "origin_period": 2024,
            "development_age": 12,
            "amount": 100.0,
            "is_cumulative": True,
        }
    ]


def test_triangle_from_frame_stores_complete_mapping():
    claims = [
        {
            "AY": 2024,
            "dev_month": 12,
            "paid_loss": 100.0,
            "is_cumulative": True,
        }
    ]

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

    assert triangle.data.to_pylist() == [
        {
            "portfolio_id": "segment",
            "valuation_date": "2026-12-31",
            "origin_period": 2024,
            "development_age": 12,
            "measure": "paid",
            "amount": 100.0,
            "currency": "CHF",
            "is_cumulative": True,
        }
    ]
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

    assert triangle.data.to_pylist() == [
        {
            "portfolio_id": "segment",
            "valuation_date": "2026-12-31",
            "origin_period": 2024,
            "development_age": 12,
            "measure": "paid",
            "amount": 100.0,
            "currency": "CHF",
            "is_cumulative": True,
        }
    ]
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


def test_triangle_from_frame_normalizes_pyarrow_table():
    table = pa.table({"AY": [2024], "dev_month": [12], "paid_loss": [100.0]})

    triangle = Triangle.from_frame(
        table,
        origin="AY",
        development="dev_month",
        value="paid_loss",
    )

    assert triangle.data.column_names == [
        "origin_period",
        "development_age",
        "amount",
        "is_cumulative",
    ]


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
    assert triangle.data.to_pylist() == [
        {
            "origin_period": 2024,
            "development_age": 12,
            "amount": 100.0,
            "is_cumulative": True,
        }
    ]


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

    assert triangle.data.to_pylist() == [
        {
            "origin_period": 2024,
            "development_age": 12,
            "amount": 100.0,
            "is_cumulative": True,
        }
    ]


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

    assert triangle.data.column_names == [
        "origin_period",
        "development_age",
        "amount",
        "is_cumulative",
    ]
    assert triangle.data.schema.metadata is None
    assert triangle.data.to_pylist() == [
        {
            "origin_period": 2024,
            "development_age": 12,
            "amount": 100.0,
            "is_cumulative": True,
        }
    ]


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
    assert triangle.data.to_pylist() == [
        {
            "origin_period": 2024,
            "development_age": 12,
            "amount": 100.0,
            "is_cumulative": True,
        }
    ]


def test_triangle_from_frame_rejects_unsupported_data():
    with pytest.raises(TypeError, match="data must be"):
        Triangle.from_frame(
            object(),
            origin="AY",
            development="dev_month",
            value="paid_loss",
        )


def test_triangle_from_frame_normalizes_source_columns_and_drops_extras():
    claims = pa.table(
        {
            "segment": ["Motor"],
            "valuation_dt": ["2026-12-31"],
            "AY": [2024],
            "dev_month": [12],
            "loss_type": ["paid"],
            "paid_loss": [100.0],
            "currency_code": ["CHF"],
            "cumulative_flag": [True],
            "unused": ["drop me"],
        }
    )

    triangle = Triangle.from_frame(
        claims,
        origin="AY",
        development="dev_month",
        value="paid_loss",
        cumulative="cumulative_flag",
        portfolio="segment",
        valuation_date="valuation_dt",
        measure="loss_type",
        currency="currency_code",
    )

    assert triangle.data.column_names == [
        "portfolio_id",
        "valuation_date",
        "origin_period",
        "development_age",
        "measure",
        "amount",
        "currency",
        "is_cumulative",
    ]
    assert triangle.data.to_pylist() == [
        {
            "portfolio_id": "Motor",
            "valuation_date": "2026-12-31",
            "origin_period": 2024,
            "development_age": 12,
            "measure": "paid",
            "amount": 100.0,
            "currency": "CHF",
            "is_cumulative": True,
        }
    ]


def test_triangle_from_frame_materializes_plain_scalar_constants():
    triangle = Triangle.from_frame(
        [{"AY": 2024, "dev_month": 12, "paid_loss": 100.0}],
        origin="AY",
        development="dev_month",
        value="paid_loss",
        portfolio="Motor",
        valuation_date="2026-12-31",
        measure="paid",
        currency="CHF",
    )

    assert triangle.data.to_pylist() == [
        {
            "portfolio_id": "Motor",
            "valuation_date": "2026-12-31",
            "origin_period": 2024,
            "development_age": 12,
            "measure": "paid",
            "amount": 100.0,
            "currency": "CHF",
            "is_cumulative": True,
        }
    ]


def test_triangle_from_frame_explicit_constant_overrides_matching_source_column():
    triangle = Triangle.from_frame(
        [
            {
                "AY": 2024,
                "dev_month": 12,
                "paid_loss": 100.0,
                "measure": "incurred",
            }
        ],
        origin="AY",
        development="dev_month",
        value="paid_loss",
        measure={"const": "paid"},
    )

    assert triangle.data["measure"].to_pylist() == ["paid"]


def test_triangle_from_frame_rejects_malformed_constant_mapping():
    with pytest.raises(ValueError, match="only a `const` field"):
        Triangle.from_frame(
            [{"AY": 2024, "dev_month": 12, "paid_loss": 100.0}],
            origin="AY",
            development="dev_month",
            value="paid_loss",
            measure={"value": "paid"},
        )


@pytest.mark.parametrize(
    ("mapping_field", "source_column", "canonical_field"),
    [
        ("origin", "AY", "origin_period"),
        ("development", "dev_month", "development_age"),
        ("value", "paid_loss", "amount"),
        ("cumulative", "cumulative_flag", "is_cumulative"),
    ],
)
def test_triangle_from_frame_reports_source_and_canonical_for_missing_columns(
    mapping_field,
    source_column,
    canonical_field,
):
    mapping = {
        "origin": "origin",
        "development": "development",
        "value": "value",
        "cumulative": True,
    }
    mapping[mapping_field] = source_column

    with pytest.raises(ColumnMappingError) as exc_info:
        Triangle.from_frame(
            [{"origin": 2024, "development": 12, "value": 100.0}],
            **mapping,
        )

    error = exc_info.value
    assert error.source_column == source_column
    assert error.canonical_field == canonical_field
    assert error.available_columns == ("origin", "development", "value")
    assert f"canonical field `{canonical_field}`" in str(error)
    assert f"source column `{source_column}`" in str(error)
    assert "Available columns: origin, development, value." in str(error)


def test_triangle_from_frame_reports_ambiguous_source_column():
    claims = pa.Table.from_arrays(
        [
            pa.array([2024]),
            pa.array([2025]),
            pa.array([12]),
            pa.array([100.0]),
        ],
        names=["AY", "AY", "dev_month", "paid_loss"],
    )

    with pytest.raises(ColumnMappingError, match="appears 2 times") as exc_info:
        Triangle.from_frame(
            claims,
            origin="AY",
            development="dev_month",
            value="paid_loss",
        )

    assert exc_info.value.canonical_field == "origin_period"
    assert exc_info.value.source_column == "AY"


def test_triangle_from_frame_reports_ambiguous_optional_source_column():
    claims = pa.Table.from_arrays(
        [
            pa.array(["Motor"]),
            pa.array(["Property"]),
            pa.array([2024]),
            pa.array([12]),
            pa.array([100.0]),
        ],
        names=["segment", "segment", "AY", "dev_month", "paid_loss"],
    )

    with pytest.raises(ColumnMappingError, match="appears 2 times") as exc_info:
        Triangle.from_frame(
            claims,
            origin="AY",
            development="dev_month",
            value="paid_loss",
            portfolio="segment",
        )

    assert exc_info.value.canonical_field == "portfolio_id"
    assert exc_info.value.source_column == "segment"
