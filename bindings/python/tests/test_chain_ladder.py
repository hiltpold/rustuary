import pytest
import pyarrow as pa

import rustuary.chain_ladder as chain_ladder_module
from rustuary import ChainLadder, ReserveResult, Triangle


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

    assert isinstance(result, ReserveResult)
    assert result["calculation_basis"] == "cumulative"
    assert_close(result["age_to_age_factors"][0], 390.0 / 220.0)
    assert_close(result["age_to_age_factors"][1], 240.0 / 180.0)
    assert_close(result["origins"][1]["ultimate"], 280.0)
    assert_close(result["origins"][2]["reserve"], 204.54545454545456)


def test_chain_ladder_class_only_materializes_inputs_before_delegating(monkeypatch):
    calls = []
    expected_payload = {"result": "from rust"}

    class FakeRust:
        def chain_ladder(self, **kwargs):
            calls.append(kwargs)
            return expected_payload

    monkeypatch.setattr(chain_ladder_module, "_load_rust_extension", lambda: FakeRust())

    result = ChainLadder(tail_factor=1.25).fit_predict(
        origin_periods=(period for period in [2020, 2021]),
        development_ages=(age for age in [12, 24]),
        rows=([100, 180], [120, None]),
        cumulative=False,
    )

    assert isinstance(result, ReserveResult)
    assert result.to_dict() == expected_payload
    assert calls == [
        {
            "origin_periods": [2020, 2021],
            "development_ages": [12, 24],
            "rows": [[100.0, 180.0], [120.0, None]],
            "cumulative": False,
            "tail_factor": 1.25,
        }
    ]


def test_chain_ladder_class_accepts_mapped_triangle_from_frame():
    triangle = Triangle.from_frame(
        [
            {"AY": 2020, "dev_month": 12, "paid": 100.0},
            {"AY": 2020, "dev_month": 24, "paid": 180.0},
            {"AY": 2020, "dev_month": 36, "paid": 240.0},
            {"AY": 2021, "dev_month": 12, "paid": 120.0},
            {"AY": 2021, "dev_month": 24, "paid": 210.0},
            {"AY": 2022, "dev_month": 12, "paid": 150.0},
        ],
        origin="AY",
        development="dev_month",
        value="paid",
    )

    result = ChainLadder().fit_predict(triangle)

    assert isinstance(result, ReserveResult)
    assert result["calculation_basis"] == "cumulative"
    assert_close(result["age_to_age_factors"][0], 390.0 / 220.0)
    assert_close(result["age_to_age_factors"][1], 240.0 / 180.0)
    assert_close(result["origins"][1]["ultimate"], 280.0)
    assert_close(result["origins"][2]["reserve"], 204.54545454545456)


def test_chain_ladder_class_reshapes_mapped_triangle_before_delegating(monkeypatch):
    calls = []
    expected_payload = {"result": "from rust"}

    class FakeRust:
        def chain_ladder(self, **kwargs):
            calls.append(kwargs)
            return expected_payload

    monkeypatch.setattr(chain_ladder_module, "_load_rust_extension", lambda: FakeRust())
    triangle = Triangle.from_frame(
        [
            {"AY": 2021, "dev_month": 12, "paid": 120, "basis": False},
            {"AY": 2020, "dev_month": 24, "paid": 80, "basis": False},
            {"AY": 2020, "dev_month": 12, "paid": 100, "basis": False},
        ],
        origin="AY",
        development="dev_month",
        value="paid",
        cumulative="basis",
    )

    result = ChainLadder(tail_factor=1.1).fit_predict(triangle)

    assert isinstance(result, ReserveResult)
    assert result.to_dict() == expected_payload
    assert calls == [
        {
            "origin_periods": [2020, 2021],
            "development_ages": [12, 24],
            "rows": [[100.0, 80.0], [120.0, None]],
            "cumulative": False,
            "tail_factor": 1.1,
        }
    ]


def test_reserve_result_summary_returns_origin_level_rows():
    result = ChainLadder().fit_predict(
        origin_periods=[2020, 2021, 2022],
        development_ages=[12, 24, 36],
        rows=[
            [100.0, 180.0, 240.0],
            [120.0, 210.0, None],
            [150.0, None, None],
        ],
    )

    summary = result.summary()

    assert [row["origin_period"] for row in summary] == [2020, 2021, 2022]
    assert [row["latest_development_age"] for row in summary] == [36, 24, 12]
    assert_close(summary[0]["latest_observed"], 240.0)
    assert_close(summary[0]["cdf_to_ultimate"], 1.0)
    assert_close(summary[0]["ultimate"], 240.0)
    assert_close(summary[0]["reserve"], 0.0)
    assert_close(summary[1]["ultimate"], 280.0)
    assert_close(summary[1]["reserve"], 70.0)
    assert_close(summary[2]["cdf_to_ultimate"], (390.0 / 220.0) * (240.0 / 180.0))
    assert_close(summary[2]["ultimate"], 354.54545454545456)
    assert_close(summary[2]["reserve"], 204.54545454545456)


def test_reserve_result_diagnostics_returns_calculation_details():
    result = ChainLadder().fit_predict(
        origin_periods=[2020, 2021, 2022],
        development_ages=[12, 24, 36],
        rows=[
            [100.0, 180.0, 240.0],
            [120.0, 210.0, None],
            [150.0, None, None],
        ],
    )

    diagnostics = result.diagnostics()

    assert diagnostics["basis"] == {
        "input": "cumulative",
        "calculation": "cumulative",
        "conversion_applied": False,
    }
    assert_close(diagnostics["age_to_age_factors"][0], 390.0 / 220.0)
    assert_close(diagnostics["cdfs"][0], (390.0 / 220.0) * (240.0 / 180.0))
    assert diagnostics["tail_factor"] == {"factor": 1.0, "rationale": None}
    assert diagnostics["selected_factors"][0]["method"] == "volume_weighted"
    assert diagnostics["selected_factors"][0]["observation_count"] == 2
    assert diagnostics["cdf_diagnostics"][-1]["development_age"] == 36
    assert diagnostics["origin_diagnostics"][2]["origin_period"] == 2022
    assert_close(
        diagnostics["origin_diagnostics"][2]["remaining_factor_product"],
        (390.0 / 220.0) * (240.0 / 180.0),
    )


def test_reserve_result_diagnostics_returns_detached_payload():
    result = ChainLadder().fit_predict(
        origin_periods=[2020, 2021],
        development_ages=[12, 24],
        rows=[
            [100.0, 180.0],
            [120.0, None],
        ],
    )

    diagnostics = result.diagnostics()
    diagnostics["selected_factors"][0]["factor"] = 999.0

    assert result["selected_factors"][0]["factor"] != 999.0


def test_reserve_result_to_arrow_returns_summary_table():
    result = ChainLadder().fit_predict(
        origin_periods=[2020, 2021, 2022],
        development_ages=[12, 24, 36],
        rows=[
            [100.0, 180.0, 240.0],
            [120.0, 210.0, None],
            [150.0, None, None],
        ],
    )

    table = result.to_arrow()

    assert isinstance(table, pa.Table)
    assert table.schema == pa.schema(
        [
            ("origin_period", pa.int64()),
            ("latest_development_age", pa.int64()),
            ("latest_observed", pa.float64()),
            ("cdf_to_ultimate", pa.float64()),
            ("ultimate", pa.float64()),
            ("reserve", pa.float64()),
        ]
    )
    assert table.to_pylist() == result.summary()


def test_reserve_result_to_pandas_returns_summary_dataframe():
    pandas = pytest.importorskip("pandas")
    result = ChainLadder().fit_predict(
        origin_periods=[2020, 2021, 2022],
        development_ages=[12, 24, 36],
        rows=[
            [100.0, 180.0, 240.0],
            [120.0, 210.0, None],
            [150.0, None, None],
        ],
    )

    frame = result.to_pandas()

    assert isinstance(frame, pandas.DataFrame)
    assert list(frame.columns) == [
        "origin_period",
        "latest_development_age",
        "latest_observed",
        "cdf_to_ultimate",
        "ultimate",
        "reserve",
    ]
    assert frame.to_dict("records") == result.summary()


def test_reserve_result_to_pandas_reports_missing_optional_dependency(monkeypatch):
    def import_or_fail(module_name):
        if module_name == "pandas":
            raise ImportError("missing pandas")
        return __import__(module_name)

    monkeypatch.setattr(chain_ladder_module, "import_module", import_or_fail)
    result = ReserveResult({})

    with pytest.raises(ImportError, match=r"rustuary\[pandas\]"):
        result.to_pandas()


def test_reserve_result_to_dict_returns_detached_payload():
    result = ChainLadder().fit_predict(
        origin_periods=[2020],
        development_ages=[12],
        rows=[[100.0]],
    )

    payload = result.to_dict()
    payload["origins"][0]["reserve"] = 999.0

    assert result["origins"][0]["reserve"] == 0.0


def test_chain_ladder_class_rejects_mapped_triangle_with_duplicate_cells():
    triangle = Triangle.from_frame(
        [
            {"AY": 2020, "dev_month": 12, "paid": 100.0},
            {"AY": 2020, "dev_month": 12, "paid": 120.0},
        ],
        origin="AY",
        development="dev_month",
        value="paid",
    )

    with pytest.raises(ValueError, match="duplicate cells"):
        ChainLadder().fit_predict(triangle)


def test_chain_ladder_class_rejects_mapped_triangle_with_mixed_basis():
    triangle = Triangle.from_frame(
        [
            {"AY": 2020, "dev_month": 12, "paid": 100.0, "basis": True},
            {"AY": 2020, "dev_month": 24, "paid": 180.0, "basis": False},
        ],
        origin="AY",
        development="dev_month",
        value="paid",
        cumulative="basis",
    )

    with pytest.raises(ValueError, match="single cumulative or incremental basis"):
        ChainLadder().fit_predict(triangle)


def test_chain_ladder_class_rejects_mixed_triangle_and_dense_arguments():
    triangle = Triangle.from_frame(
        [{"AY": 2020, "dev_month": 12, "paid": 100.0}],
        origin="AY",
        development="dev_month",
        value="paid",
    )

    with pytest.raises(TypeError, match="cannot be combined"):
        ChainLadder().fit_predict(
            triangle,
            origin_periods=[2020],
            development_ages=[12],
            rows=[[100.0]],
        )


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
