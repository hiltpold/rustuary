import pytest

import rustuary._rust as _rust  # type: ignore[import-untyped]


def assert_close(actual, expected):
    assert abs(actual - expected) <= 1e-9


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
