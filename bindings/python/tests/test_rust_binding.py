from datetime import date

import pytest

import rustuary._rust as _rust  # type: ignore[import-untyped]


def assert_close(actual, expected):
    assert abs(actual - expected) <= 1e-9


def raw_triangle_request(**overrides):
    request = {
        "triangle_definition_id": "paid-claims-v1",
        "schema_version": "1",
        "aggregation": "sum",
        "bucket_months": 12,
        "output_kind": "cumulative",
        "segment_names": ["country", "coverage"],
    }
    request.update(overrides)
    return request


def raw_triangle_record(**overrides):
    record = {
        "origin_date": date(2024, 1, 15),
        "development_date": date(2024, 3, 10),
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
    record.update(overrides)
    return record


def test_chain_ladder_binding_matches_basic_golden_triangle():
    result = _rust.chain_ladder(
        origin_periods=[2020, 2021, 2022],
        development_ages=[12, 24, 36],
        rows=[
            [100.0, 180.0, 240.0],
            [120.0, 210.0, None],
            [150.0, None, None],
        ],
    )

    assert result["input_basis"] == "cumulative"
    assert result["calculation_basis"] == "cumulative"
    assert result["basis_conversion_applied"] is False
    assert_close(result["age_to_age_factors"][0], 390.0 / 220.0)
    assert_close(result["age_to_age_factors"][1], 240.0 / 180.0)

    first_factor = result["selected_factors"][0]
    assert first_factor["from_development_age"] == 12
    assert first_factor["to_development_age"] == 24
    assert first_factor["method"] == "volume_weighted"
    assert first_factor["observation_count"] == 2
    assert first_factor["exclusions"] == []
    assert first_factor["applied_override"] is None
    assert_close(first_factor["numerator"], 390.0)
    assert_close(first_factor["denominator"], 220.0)
    assert_close(first_factor["factor"], 390.0 / 220.0)

    final_cdf = result["cdf_diagnostics"][-1]
    assert final_cdf["development_age"] == 36
    assert final_cdf["next_development_age"] is None
    assert final_cdf["age_to_age_factor"] is None
    assert_close(final_cdf["cdf"], 1.0)

    expected_origins = [
        {
            "origin_index": 0,
            "origin_period": 2020,
            "latest_development_index": 2,
            "latest_development_age": 36,
            "latest_observed": 240.0,
            "remaining_factor_product": 1.0,
            "tail_factor": 1.0,
            "cdf_to_ultimate": 1.0,
            "ultimate": 240.0,
            "reserve": 0.0,
        },
        {
            "origin_index": 1,
            "origin_period": 2021,
            "latest_development_index": 1,
            "latest_development_age": 24,
            "latest_observed": 210.0,
            "remaining_factor_product": 240.0 / 180.0,
            "tail_factor": 1.0,
            "cdf_to_ultimate": 240.0 / 180.0,
            "ultimate": 280.0,
            "reserve": 70.0,
        },
        {
            "origin_index": 2,
            "origin_period": 2022,
            "latest_development_index": 0,
            "latest_development_age": 12,
            "latest_observed": 150.0,
            "remaining_factor_product": (390.0 / 220.0) * (240.0 / 180.0),
            "tail_factor": 1.0,
            "cdf_to_ultimate": (390.0 / 220.0) * (240.0 / 180.0),
            "ultimate": 354.54545454545456,
            "reserve": 204.54545454545456,
        },
    ]

    assert len(result["origins"]) == len(expected_origins)
    for actual, expected in zip(result["origins"], expected_origins, strict=True):
        for key, expected_value in expected.items():
            if isinstance(expected_value, float):
                assert_close(actual[key], expected_value)
            else:
                assert actual[key] == expected_value


def test_chain_ladder_binding_converts_incremental_input_before_calculation():
    result = _rust.chain_ladder(
        origin_periods=[2020, 2021, 2022],
        development_ages=[12, 24, 36],
        rows=[
            [100.0, 80.0, 60.0],
            [120.0, 90.0, None],
            [150.0, None, None],
        ],
        cumulative=False,
    )

    assert result["input_basis"] == "incremental"
    assert result["calculation_basis"] == "cumulative"
    assert result["basis_conversion_applied"] is True
    assert_close(result["origins"][1]["ultimate"], 280.0)
    assert_close(result["origins"][2]["reserve"], 204.54545454545456)


def test_chain_ladder_binding_maps_core_validation_errors_to_value_error():
    with pytest.raises(ValueError, match="triangle has 1 rows but 2 origin periods"):
        _rust.chain_ladder(
            origin_periods=[2020, 2021],
            development_ages=[12],
            rows=[[100.0]],
        )


def test_build_triangle_set_binding_sums_raw_records_into_cumulative_triangles():
    result = _rust.build_triangle_set(
        raw_triangle_request(),
        [
            raw_triangle_record(),
            raw_triangle_record(development_date=date(2025, 1, 5), amount=50.0),
            raw_triangle_record(
                origin_date=date(2025, 2, 1),
                development_date=date(2025, 8, 15),
                amount=80.0,
            ),
        ],
    )

    assert result["diagnostics"] == {
        "source_record_count": 3,
        "triangle_count": 1,
        "cumulative_conversion_applied": True,
    }
    assert len(result["triangles"]) == 1

    triangle = result["triangles"][0]
    assert triangle["key"] == {
        "portfolio_id": "Motor",
        "segments": [
            {"name": "country", "value": "CH"},
            {"name": "coverage", "value": "MTPL"},
        ],
        "measure": "paid",
        "display_path": "Motor/CH/MTPL",
    }
    assert triangle["origin_periods"] == [2024, 2025]
    assert triangle["development_ages"] == [12, 24]
    assert triangle["basis"] == "cumulative"
    assert triangle["rows"] == [[100.0, 150.0], [80.0, None]]
    assert triangle["diagnostics"] == {
        "source_record_count": 3,
        "cumulative_conversion_applied": True,
    }


def test_build_triangle_set_binding_counts_records_without_amounts():
    result = _rust.build_triangle_set(
        raw_triangle_request(
            triangle_definition_id="reported-counts-v1",
            aggregation="count",
            output_kind="incremental",
            segment_names=[],
        ),
        [
            raw_triangle_record(amount=None, segments=[], measure="reported_count"),
            raw_triangle_record(
                origin_date={"year": 2024, "month": 1, "day": 20},
                development_date={"year": 2024, "month": 11, "day": 5},
                amount=None,
                segments=[],
                measure="reported_count",
            ),
        ],
    )

    triangle = result["triangles"][0]
    assert triangle["key"] == {
        "portfolio_id": "Motor",
        "segments": [],
        "measure": "reported_count",
        "display_path": "Motor",
    }
    assert triangle["origin_periods"] == [2024]
    assert triangle["development_ages"] == [12]
    assert triangle["basis"] == "incremental"
    assert triangle["rows"] == [[2.0]]


def test_build_triangle_set_binding_maps_core_errors_to_value_error():
    with pytest.raises(ValueError, match="does not support bucket_months=2"):
        _rust.build_triangle_set(
            raw_triangle_request(bucket_months=2),
            [raw_triangle_record()],
        )


def test_build_triangle_set_binding_reports_missing_record_dates():
    with pytest.raises(
        ValueError,
        match=r"record 0 canonical field `origin_date` is required and cannot be null",
    ):
        _rust.build_triangle_set(
            raw_triangle_request(),
            [raw_triangle_record(origin_date=None)],
        )
