import pytest
import pyarrow as pa

import rustuary.triangle_builder as triangle_builder_module
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


def test_triangle_builder_build_payload_adapts_records_and_delegates_to_rust(monkeypatch):
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
    calls = []

    class FakeRust:
        def build_triangle_set(self, request, records):
            calls.append((request, records))
            return {"diagnostics": {"source_record_count": len(records)}, "triangles": []}

    monkeypatch.setattr(triangle_builder_module, "_load_rust_extension", lambda: FakeRust())

    payload = builder._build_payload(
        [
            {
                "reserving_class": "Motor",
                "country": "CH",
                "currency": "CHF",
                "accident_date": "2024-01-15",
                "payment_date": "2024-03-10",
                "paid_loss": 100.0,
            }
        ]
    )

    assert payload == {"diagnostics": {"source_record_count": 1}, "triangles": []}
    assert calls == [
        (
            {
                "triangle_definition_id": "paid-claims-v1",
                "schema_version": "1",
                "aggregation": "sum",
                "bucket_months": 12,
                "output_kind": "cumulative",
                "segment_names": ["country", "coverage"],
            },
            [
                {
                    "origin_date": "2024-01-15",
                    "development_date": "2024-03-10",
                    "amount": 100.0,
                    "portfolio_id": "Motor",
                    "segments": [
                        {"name": "country", "value": "CH"},
                        {"name": "coverage", "value": "MTPL"},
                    ],
                    "measure": "paid",
                    "valuation_date": "2026-12-31",
                    "currency": "CHF",
                }
            ],
        )
    ]


def test_triangle_builder_build_payload_count_records_omit_amount(monkeypatch):
    definition = TriangleDefinition(
        triangle_definition_id="reported-counts-v1",
        origin_date="accident_date",
        development_date="report_date",
        measure={"const": "reported_count"},
        portfolio_id="reserving_class",
        aggregation="count",
    )
    builder = TriangleBuilder(definition)
    calls = []

    class FakeRust:
        def build_triangle_set(self, request, records):
            calls.append((request, records))
            return {"diagnostics": {"source_record_count": len(records)}, "triangles": []}

    monkeypatch.setattr(triangle_builder_module, "_load_rust_extension", lambda: FakeRust())

    builder._build_payload(
        [
            {
                "reserving_class": "Motor",
                "accident_date": "2024-01-15",
                "report_date": "2024-02-10",
            }
        ]
    )

    assert calls[0][0]["aggregation"] == "count"
    assert calls[0][1][0]["amount"] is None
    assert calls[0][1][0]["measure"] == "reported_count"


def test_triangle_builder_build_payload_reports_null_raw_dates():
    definition = TriangleDefinition(
        triangle_definition_id="paid-claims-v1",
        origin_date="accident_date",
        development_date="payment_date",
        amount="paid_loss",
        measure={"const": "paid"},
        portfolio_id="reserving_class",
    )
    builder = TriangleBuilder(definition)

    with pytest.raises(
        ValueError,
        match=r"canonical field `origin_date` mapped from source column `accident_date` "
        r"is null at row 0",
    ):
        builder._build_payload(
            [
                {
                    "reserving_class": "Motor",
                    "accident_date": None,
                    "payment_date": "2024-03-10",
                    "paid_loss": 100.0,
                }
            ]
        )


def test_triangle_builder_rejects_wrong_definition_type():
    with pytest.raises(TypeError, match="TriangleDefinition"):
        TriangleBuilder(definition="paid-claims-v1")
