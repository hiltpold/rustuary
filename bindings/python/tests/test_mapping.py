from dataclasses import FrozenInstanceError

import pytest

from rustuary import ClaimsMapping, ExposureMapping


def test_claims_mapping_stores_canonical_mapping_metadata():
    mapping = ClaimsMapping(
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

    assert mapping.origin == "AY"
    assert mapping.development == "dev_month"
    assert mapping.value == "paid_loss"
    assert mapping.cumulative == "is_cumulative"
    assert mapping.portfolio == "segment"
    assert mapping.valuation_date == {"const": "2026-12-31"}
    assert mapping.measure == {"const": "paid"}
    assert mapping.currency == {"const": "CHF"}
    assert mapping.origin_type == "accident_year"
    assert mapping.development_unit == "months"


def test_claims_mapping_defaults_to_cumulative_without_optional_metadata():
    mapping = ClaimsMapping(origin="AY", development="dev_month", value="paid_loss")

    assert mapping.cumulative is True
    assert mapping.portfolio is None
    assert mapping.valuation_date is None
    assert mapping.measure is None
    assert mapping.currency is None
    assert mapping.origin_type is None
    assert mapping.development_unit is None


def test_exposure_mapping_stores_source_columns_and_constants():
    mapping = ExposureMapping(
        origin="AY",
        value="earned_premium",
        exposure_measure={"const": "earned_premium"},
        portfolio="segment",
        valuation_date={"const": "2026-12-31"},
        currency={"const": "CHF"},
    )

    assert mapping.origin == "AY"
    assert mapping.value == "earned_premium"
    assert mapping.exposure_measure == {"const": "earned_premium"}
    assert mapping.portfolio == "segment"
    assert mapping.valuation_date == {"const": "2026-12-31"}
    assert mapping.currency == {"const": "CHF"}


def test_mapping_objects_reject_attribute_reassignment():
    mapping = ClaimsMapping(origin="AY", development="dev_month", value="paid_loss")

    with pytest.raises(FrozenInstanceError):
        mapping.origin = "accident_year"


@pytest.mark.parametrize("field", ["origin", "development", "value"])
def test_claims_mapping_rejects_empty_required_source_columns(field):
    values = {"origin": "AY", "development": "dev_month", "value": "paid_loss"}
    values[field] = " "

    with pytest.raises(ValueError, match=field):
        ClaimsMapping(**values)


def test_claims_mapping_rejects_invalid_cumulative_value():
    with pytest.raises(TypeError, match="cumulative"):
        ClaimsMapping(origin="AY", development="dev_month", value="paid_loss", cumulative=1)


def test_claims_mapping_rejects_unknown_development_unit():
    with pytest.raises(ValueError, match="development_unit"):
        ClaimsMapping(
            origin="AY",
            development="dev_month",
            value="paid_loss",
            development_unit="weeks",
        )


def test_exposure_mapping_rejects_empty_exposure_measure():
    with pytest.raises(ValueError, match="exposure_measure"):
        ExposureMapping(origin="AY", value="earned_premium", exposure_measure=" ")
