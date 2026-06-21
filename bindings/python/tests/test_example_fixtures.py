import json
from datetime import date
from pathlib import Path

import pyarrow.csv as pv
import yaml  # type: ignore[import-untyped]

from rustuary import ClaimsMapping, SegmentDefinition, Triangle, TriangleBuilder, TriangleDefinition


REPO_ROOT = Path(__file__).resolve().parents[3]
CLAIMS_PATH = REPO_ROOT / "data" / "examples" / "paid_claims_custom_columns.csv"
MAPPING_PATH = REPO_ROOT / "contracts" / "examples" / "claims_mapping.yaml"
RAW_CLAIM_EVENTS_PATH = REPO_ROOT / "data" / "examples" / "raw_claim_events.csv"
RAW_CLAIM_TRIANGLE_SET_GOLDEN_PATH = (
    REPO_ROOT / "data" / "golden" / "raw_claim_triangle_set.json"
)


def test_custom_column_claims_and_yaml_mapping_form_canonical_triangle():
    mapping_document = yaml.safe_load(MAPPING_PATH.read_text())
    mapping = ClaimsMapping(**mapping_document["claims_mapping"])
    claims = pv.read_csv(CLAIMS_PATH)

    triangle = Triangle.from_frame(claims, mapping=mapping)

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
    assert triangle.data.num_rows == 6
    assert triangle.data["origin_period"].to_pylist() == [2020, 2020, 2020, 2021, 2021, 2022]
    assert triangle.data["development_age"].to_pylist() == [12, 24, 36, 12, 24, 12]
    assert triangle.data["amount"].to_pylist() == [1000, 1800, 2300, 1200, 2100, 1400]
    assert triangle.data["valuation_date"].to_pylist() == [date(2026, 12, 31)] * 6
    assert triangle.data["measure"].to_pylist() == ["paid"] * 6
    assert triangle.data["currency"].to_pylist() == ["CHF"] * 6
    assert triangle.data["is_cumulative"].to_pylist() == [True] * 6

    metadata = triangle.model_run_metadata.to_dict()
    assert metadata["claims_mapping"]["valuation_date"] == {"const": "2026-12-31"}
    assert metadata["claims_mapping"]["development_unit"] == "months"


def test_raw_claim_events_build_rust_backed_triangle_set_golden_fixture():
    definition = TriangleDefinition(
        triangle_definition_id="paid-claims-v1",
        origin_date="accident_date",
        development_date="payment_date",
        amount="paid_loss",
        measure={"const": "paid"},
        portfolio_id="reserving_class",
        segments=[
            SegmentDefinition(name="country", source="country"),
            SegmentDefinition(name="coverage", source="coverage"),
        ],
        currency="currency",
    )
    raw_claims = pv.read_csv(RAW_CLAIM_EVENTS_PATH)
    expected_payload = json.loads(RAW_CLAIM_TRIANGLE_SET_GOLDEN_PATH.read_text())

    triangle_set = TriangleBuilder.from_frame(raw_claims, definition=definition)

    assert triangle_set.to_dict() == expected_payload
    assert triangle_set.keys() == [
        triangle["key"] for triangle in expected_payload["triangles"]
    ]
    assert triangle_set.audit_trail()["input"]["triangle_definition"] == definition.to_dict()
