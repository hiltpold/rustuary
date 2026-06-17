import pytest
import pyarrow as pa

from rustuary import ColumnMappingError, SegmentDefinition, TriangleBuilder, TriangleDefinition


def test_triangle_builder_stores_definition_and_required_source_columns():
    definition = TriangleDefinition(
        triangle_definition_id="paid-claims-v1",
        origin_date="accident_date",
        development_date="payment_date",
        amount="paid_loss",
        measure={"const": "paid"},
        portfolio_id="reserving_class",
        segments=[
            SegmentDefinition(name="country", source="country"),
            SegmentDefinition(name="coverage", source={"const": "MTPL"}),
        ],
        valuation_date={"const": "2026-12-31"},
        currency="currency",
    )

    builder = TriangleBuilder(definition)

    assert builder.definition == definition
    assert builder.required_source_columns() == (
        "accident_date",
        "payment_date",
        "paid_loss",
        "reserving_class",
        "country",
        "currency",
    )


def test_triangle_builder_count_definition_does_not_require_amount_source():
    definition = TriangleDefinition(
        triangle_definition_id="reported-counts-v1",
        origin_date="accident_date",
        development_date="report_date",
        measure={"const": "reported_count"},
        portfolio_id="reserving_class",
        aggregation="count",
    )

    builder = TriangleBuilder(definition)

    assert builder.required_source_columns() == (
        "accident_date",
        "report_date",
        "reserving_class",
    )


def test_triangle_builder_validate_frame_returns_arrow_table():
    definition = TriangleDefinition(
        triangle_definition_id="paid-claims-v1",
        origin_date="accident_date",
        development_date="payment_date",
        amount="paid_loss",
        measure={"const": "paid"},
        portfolio_id="reserving_class",
    )
    builder = TriangleBuilder(definition)

    table = builder.validate_frame(
        [
            {
                "reserving_class": "Motor",
                "accident_date": "2024-01-15",
                "payment_date": "2024-03-10",
                "paid_loss": 100.0,
            }
        ]
    )

    assert isinstance(table, pa.Table)
    assert table.column_names == [
        "reserving_class",
        "accident_date",
        "payment_date",
        "paid_loss",
    ]


def test_triangle_builder_validate_frame_reports_missing_source_column():
    definition = TriangleDefinition(
        triangle_definition_id="paid-claims-v1",
        origin_date="accident_date",
        development_date="payment_date",
        amount="paid_loss",
        measure={"const": "paid"},
        portfolio_id="reserving_class",
    )
    builder = TriangleBuilder(definition)

    with pytest.raises(ColumnMappingError) as exc_info:
        builder.validate_frame(
            [
                {
                    "reserving_class": "Motor",
                    "accident_date": "2024-01-15",
                    "paid_loss": 100.0,
                }
            ]
        )

    assert exc_info.value.canonical_field == "development_age"
    assert exc_info.value.source_column == "payment_date"
    assert "source column `payment_date`" in str(exc_info.value)


def test_triangle_builder_validate_frame_reports_duplicate_source_column():
    definition = TriangleDefinition(
        triangle_definition_id="paid-claims-v1",
        origin_date="accident_date",
        development_date="payment_date",
        amount="paid_loss",
        measure={"const": "paid"},
        portfolio_id="reserving_class",
    )
    builder = TriangleBuilder(definition)
    table = pa.Table.from_arrays(
        [
            pa.array(["Motor"]),
            pa.array(["Property"]),
            pa.array(["2024-01-15"]),
            pa.array(["2024-03-10"]),
            pa.array([100.0]),
        ],
        names=[
            "reserving_class",
            "reserving_class",
            "accident_date",
            "payment_date",
            "paid_loss",
        ],
    )

    with pytest.raises(ColumnMappingError, match="appears 2 times") as exc_info:
        builder.validate_frame(table)

    assert exc_info.value.canonical_field == "portfolio_id"
    assert exc_info.value.source_column == "reserving_class"


def test_triangle_builder_rejects_wrong_definition_type():
    with pytest.raises(TypeError, match="TriangleDefinition"):
        TriangleBuilder(definition="paid-claims-v1")
