import pytest

from rustuary import ChainLadder
import rustuary.chain_ladder as chain_ladder_module


def assert_close(actual, expected):
    assert abs(actual - expected) <= 1e-9


def test_chain_ladder_class_delegates_to_rust_core_for_dense_triangle():
    result = ChainLadder().fit_predict(
        origin_periods=[2020, 2021, 2022],
        development_ages=[12, 24, 36],
        rows=[
            [100.0, 180.0, 240.0],
            [120.0, 210.0, None],
            [150.0, None, None],
        ],
    )

    assert result["calculation_basis"] == "cumulative"
    assert_close(result["age_to_age_factors"][0], 390.0 / 220.0)
    assert_close(result["age_to_age_factors"][1], 240.0 / 180.0)
    assert_close(result["origins"][1]["ultimate"], 280.0)
    assert_close(result["origins"][2]["reserve"], 204.54545454545456)


def test_chain_ladder_class_only_materializes_inputs_before_delegating(monkeypatch):
    calls = []
    expected_result = {"result": "from rust"}

    class FakeRust:
        def chain_ladder(self, **kwargs):
            calls.append(kwargs)
            return expected_result

    monkeypatch.setattr(chain_ladder_module, "_load_rust_extension", lambda: FakeRust())

    result = ChainLadder(tail_factor=1.25).fit_predict(
        origin_periods=(period for period in [2020, 2021]),
        development_ages=(age for age in [12, 24]),
        rows=([100, 180], [120, None]),
        cumulative=False,
    )

    assert result is expected_result
    assert calls == [
        {
            "origin_periods": [2020, 2021],
            "development_ages": [12, 24],
            "rows": [[100.0, 180.0], [120.0, None]],
            "cumulative": False,
            "tail_factor": 1.25,
        }
    ]


def test_chain_ladder_class_passes_tail_factor_to_rust_core():
    result = ChainLadder(tail_factor=1.05).fit_predict(
        origin_periods=[2020, 2021, 2022],
        development_ages=[12, 24, 36],
        rows=[
            [100.0, 180.0, 240.0],
            [120.0, 210.0, None],
            [150.0, None, None],
        ],
    )

    assert_close(result["tail_factor"]["factor"], 1.05)
    assert_close(result["origins"][0]["ultimate"], 252.0)
    assert_close(result["origins"][0]["reserve"], 12.0)


def test_chain_ladder_class_surfaces_core_validation_errors():
    with pytest.raises(ValueError, match="tail factor must be positive and finite"):
        ChainLadder(tail_factor=0.0).fit_predict(
            origin_periods=[2020],
            development_ages=[12],
            rows=[[100.0]],
        )
