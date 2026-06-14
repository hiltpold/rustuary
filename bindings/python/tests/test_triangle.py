import pytest

from rustuary import Triangle


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

    assert triangle.data is claims
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
