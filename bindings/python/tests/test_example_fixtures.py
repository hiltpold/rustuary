from datetime import date
from pathlib import Path

import pyarrow.csv as pv
import yaml

from rustuary import ClaimsMapping, Triangle


REPO_ROOT = Path(__file__).resolve().parents[3]
CLAIMS_PATH = REPO_ROOT / "data" / "examples" / "paid_claims_custom_columns.csv"
MAPPING_PATH = REPO_ROOT / "contracts" / "examples" / "claims_mapping.yaml"


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
