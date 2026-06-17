import json
from dataclasses import FrozenInstanceError
from datetime import date
from decimal import Decimal

import pytest

from rustuary import SegmentDefinition, TriangleDefinition


def test_triangle_definition_stores_raw_record_configuration():
    definition = TriangleDefinition(
        triangle_definition_id="paid-claims-v1",
        origin_date="accident_date",
        development_date="payment_date",
        amount="paid_loss",
        measure={"const": "paid"},
        aggregation="sum",
        bucket_months=3,
        output_kind="cumulative",
        portfolio_id="reserving_class",
        segments=[
            SegmentDefinition(name="country", source="country"),
            {"name": "coverage", "source": {"const": "MTPL"}},
        ],
        valuation_date={"const": date(2026, 12, 31)},
        currency={"const": "CHF"},
    )

    assert definition.triangle_definition_id == "paid-claims-v1"
    assert definition.schema_version == "1"
    assert definition.origin_date == "accident_date"
    assert definition.development_date == "payment_date"
    assert definition.amount == "paid_loss"
    assert definition.measure == {"const": "paid"}
    assert definition.aggregation == "sum"
    assert definition.bucket_months == 3
    assert definition.output_kind == "cumulative"
    assert definition.portfolio_id == "reserving_class"
    assert definition.segments == (
        SegmentDefinition(name="country", source="country"),
        SegmentDefinition(name="coverage", source={"const": "MTPL"}),
    )


def test_triangle_definition_to_dict_returns_detached_json_safe_snapshot():
    definition = TriangleDefinition(
        triangle_definition_id="paid-claims-v1",
        origin_date="accident_date",
        development_date="payment_date",
        amount="paid_loss",
        measure={"const": "paid"},
        portfolio_id={"const": "Motor"},
        segments=[{"name": "threshold", "source": {"const": Decimal("100.50")}}],
        valuation_date={"const": date(2026, 12, 31)},
    )

    payload = definition.to_dict()
    payload["segments"][0]["source"]["const"] = "changed"

    assert definition.to_dict() == {
        "triangle_definition_id": "paid-claims-v1",
        "schema_version": "1",
        "origin_date": "accident_date",
        "development_date": "payment_date",
        "amount": "paid_loss",
        "measure": {"const": "paid"},
        "aggregation": "sum",
        "bucket_months": 12,
        "output_kind": "cumulative",
        "portfolio_id": {"const": "Motor"},
        "segments": [{"name": "threshold", "source": {"const": "100.50"}}],
        "valuation_date": {"const": "2026-12-31"},
    }
    json.dumps(definition.to_dict())


def test_triangle_definition_rejects_invalid_bucket_months():
    with pytest.raises(ValueError, match="bucket_months"):
        TriangleDefinition(
            triangle_definition_id="paid-claims-v1",
            origin_date="accident_date",
            development_date="payment_date",
            amount="paid_loss",
            measure={"const": "paid"},
            portfolio_id="reserving_class",
            bucket_months=13,
        )


def test_triangle_definition_rejects_boolean_bucket_months():
    with pytest.raises(ValueError, match="bucket_months"):
        TriangleDefinition(
            triangle_definition_id="paid-claims-v1",
            origin_date="accident_date",
            development_date="payment_date",
            amount="paid_loss",
            measure={"const": "paid"},
            portfolio_id="reserving_class",
            bucket_months=True,
        )


@pytest.mark.parametrize(
    ("field_name", "field_value"),
    [
        ("origin_date", " "),
        ("development_date", " "),
        ("amount", " "),
    ],
)
def test_triangle_definition_rejects_empty_required_source_columns(
    field_name,
    field_value,
):
    values = {
        "triangle_definition_id": "paid-claims-v1",
        "origin_date": "accident_date",
        "development_date": "payment_date",
        "amount": "paid_loss",
        "measure": {"const": "paid"},
        "portfolio_id": "reserving_class",
    }
    values[field_name] = field_value

    with pytest.raises(ValueError, match=field_name):
        TriangleDefinition(**values)


def test_triangle_definition_rejects_unknown_aggregation():
    with pytest.raises(ValueError, match="aggregation"):
        TriangleDefinition(
            triangle_definition_id="paid-claims-v1",
            origin_date="accident_date",
            development_date="payment_date",
            amount="paid_loss",
            measure={"const": "paid"},
            portfolio_id="reserving_class",
            aggregation="average",
        )


def test_triangle_definition_rejects_unknown_output_kind():
    with pytest.raises(ValueError, match="output_kind"):
        TriangleDefinition(
            triangle_definition_id="paid-claims-v1",
            origin_date="accident_date",
            development_date="payment_date",
            amount="paid_loss",
            measure={"const": "paid"},
            portfolio_id="reserving_class",
            output_kind="paid",
        )


def test_triangle_definition_rejects_malformed_segments():
    with pytest.raises(ValueError, match="name.*source"):
        TriangleDefinition(
            triangle_definition_id="paid-claims-v1",
            origin_date="accident_date",
            development_date="payment_date",
            amount="paid_loss",
            measure={"const": "paid"},
            portfolio_id="reserving_class",
            segments=[{"name": "country", "column": "country"}],
        )


def test_triangle_definition_is_frozen():
    definition = TriangleDefinition(
        triangle_definition_id="paid-claims-v1",
        origin_date="accident_date",
        development_date="payment_date",
        amount="paid_loss",
        measure={"const": "paid"},
        portfolio_id="reserving_class",
    )

    with pytest.raises(FrozenInstanceError):
        definition.bucket_months = 3
