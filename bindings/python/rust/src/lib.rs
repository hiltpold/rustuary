use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rustuary_core::{
    AppliedLinkRatioExclusion, ChainLadder, ChainLadderResult, CumulativeDevelopmentFactor,
    DevelopmentAge, DevelopmentFactorMethod, DevelopmentFactorOverride, FixedTailFactor,
    OriginChainLadderResult, OriginPeriod, SelectedDevelopmentFactor, Triangle, TriangleBasis,
};

#[pyfunction]
fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pyfunction]
#[pyo3(signature = (origin_periods, development_ages, rows, cumulative=true, tail_factor=1.0))]
fn chain_ladder(
    py: Python<'_>,
    origin_periods: Vec<i32>,
    development_ages: Vec<u32>,
    rows: Vec<Vec<Option<f64>>>,
    cumulative: bool,
    tail_factor: f64,
) -> PyResult<Py<PyDict>> {
    let basis = if cumulative {
        TriangleBasis::Cumulative
    } else {
        TriangleBasis::Incremental
    };
    let triangle = Triangle::new(
        origin_periods.into_iter().map(OriginPeriod).collect(),
        development_ages.into_iter().map(DevelopmentAge).collect(),
        rows,
        basis,
    )
    .map_err(actuarial_error)?;
    let calculation_triangle = triangle.to_cumulative().map_err(actuarial_error)?;
    let result = ChainLadder::new(tail_factor)
        .and_then(|model| model.fit_predict(&calculation_triangle))
        .map_err(actuarial_error)?;

    chain_ladder_result_to_dict(py, &result, cumulative)
}

fn actuarial_error(error: rustuary_core::ActuarialError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn chain_ladder_result_to_dict(
    py: Python<'_>,
    result: &ChainLadderResult,
    input_is_cumulative: bool,
) -> PyResult<Py<PyDict>> {
    let output = PyDict::new_bound(py);
    output.set_item("input_basis", basis_name(input_is_cumulative))?;
    output.set_item("calculation_basis", "cumulative")?;
    output.set_item("basis_conversion_applied", !input_is_cumulative)?;
    output.set_item("age_to_age_factors", &result.age_to_age_factors)?;
    output.set_item(
        "selected_factors",
        selected_factors_to_list(py, &result.selected_factors)?,
    )?;
    output.set_item("cdfs", &result.cdfs)?;
    output.set_item(
        "cdf_diagnostics",
        cdf_diagnostics_to_list(py, &result.cdf_diagnostics)?,
    )?;
    output.set_item("tail_factor", tail_factor_to_dict(py, &result.tail_factor)?)?;
    output.set_item("origins", origin_results_to_list(py, &result.origins)?)?;

    Ok(output.unbind())
}

const fn basis_name(cumulative: bool) -> &'static str {
    if cumulative {
        "cumulative"
    } else {
        "incremental"
    }
}

fn selected_factors_to_list<'py>(
    py: Python<'py>,
    selections: &[SelectedDevelopmentFactor],
) -> PyResult<Bound<'py, PyList>> {
    let items = PyList::empty_bound(py);
    for selection in selections {
        let item = PyDict::new_bound(py);
        item.set_item("from_development_index", selection.from_development_index)?;
        item.set_item("from_development_age", selection.from_development_age.0)?;
        item.set_item("to_development_index", selection.to_development_index)?;
        item.set_item("to_development_age", selection.to_development_age.0)?;
        item.set_item("method", development_factor_method_name(selection.method))?;
        item.set_item("observation_count", selection.observation_count)?;
        item.set_item("exclusions", exclusions_to_list(py, &selection.exclusions)?)?;
        item.set_item("numerator", selection.numerator)?;
        item.set_item("denominator", selection.denominator)?;
        item.set_item("calculated_factor", selection.calculated_factor)?;
        set_applied_override(py, &item, selection.applied_override.as_ref())?;
        item.set_item("factor", selection.factor)?;
        items.append(item)?;
    }

    Ok(items)
}

fn development_factor_method_name(method: DevelopmentFactorMethod) -> &'static str {
    match method {
        DevelopmentFactorMethod::VolumeWeighted => "volume_weighted",
        DevelopmentFactorMethod::SimpleAverage => "simple_average",
    }
}

fn exclusions_to_list<'py>(
    py: Python<'py>,
    exclusions: &[AppliedLinkRatioExclusion],
) -> PyResult<Bound<'py, PyList>> {
    let items = PyList::empty_bound(py);
    for exclusion in exclusions {
        let link_ratio = exclusion.link_ratio;
        let item = PyDict::new_bound(py);
        item.set_item("origin_index", link_ratio.origin_index)?;
        item.set_item("origin_period", link_ratio.origin_period.0)?;
        item.set_item("from_development_index", link_ratio.from_development_index)?;
        item.set_item("from_development_age", link_ratio.from_development_age.0)?;
        item.set_item("to_development_index", link_ratio.to_development_index)?;
        item.set_item("to_development_age", link_ratio.to_development_age.0)?;
        item.set_item("from_value", link_ratio.from_value)?;
        item.set_item("to_value", link_ratio.to_value)?;
        item.set_item("ratio", link_ratio.ratio)?;
        item.set_item("rationale", &exclusion.rationale)?;
        items.append(item)?;
    }

    Ok(items)
}

fn set_applied_override(
    py: Python<'_>,
    item: &Bound<'_, PyDict>,
    applied_override: Option<&DevelopmentFactorOverride>,
) -> PyResult<()> {
    if let Some(applied_override) = applied_override {
        let override_item = PyDict::new_bound(py);
        override_item.set_item(
            "from_development_age",
            applied_override.from_development_age.0,
        )?;
        override_item.set_item("factor", applied_override.factor)?;
        override_item.set_item("rationale", &applied_override.rationale)?;
        item.set_item("applied_override", override_item)?;
    } else {
        item.set_item("applied_override", py.None())?;
    }

    Ok(())
}

fn cdf_diagnostics_to_list<'py>(
    py: Python<'py>,
    diagnostics: &[CumulativeDevelopmentFactor],
) -> PyResult<Bound<'py, PyList>> {
    let items = PyList::empty_bound(py);
    for diagnostic in diagnostics {
        let item = PyDict::new_bound(py);
        item.set_item("development_index", diagnostic.development_index)?;
        item.set_item("development_age", diagnostic.development_age.0)?;
        item.set_item(
            "next_development_age",
            diagnostic.next_development_age.map(|age| age.0),
        )?;
        item.set_item("age_to_age_factor", diagnostic.age_to_age_factor)?;
        item.set_item(
            "remaining_factor_product",
            diagnostic.remaining_factor_product,
        )?;
        item.set_item("tail_factor", diagnostic.tail_factor)?;
        item.set_item("cdf", diagnostic.cdf)?;
        items.append(item)?;
    }

    Ok(items)
}

fn tail_factor_to_dict<'py>(
    py: Python<'py>,
    tail_factor: &FixedTailFactor,
) -> PyResult<Bound<'py, PyDict>> {
    let item = PyDict::new_bound(py);
    item.set_item("factor", tail_factor.factor())?;
    item.set_item("rationale", tail_factor.rationale())?;
    Ok(item)
}

fn origin_results_to_list<'py>(
    py: Python<'py>,
    origins: &[OriginChainLadderResult],
) -> PyResult<Bound<'py, PyList>> {
    let items = PyList::empty_bound(py);
    for origin in origins {
        let item = PyDict::new_bound(py);
        set_origin_result(&item, origin)?;
        items.append(item)?;
    }

    Ok(items)
}

fn set_origin_result(item: &Bound<'_, PyDict>, origin: &OriginChainLadderResult) -> PyResult<()> {
    item.set_item("origin_index", origin.origin_index)?;
    item.set_item("origin_period", origin.origin_period.0)?;
    item.set_item("latest_development_index", origin.latest_development_index)?;
    item.set_item("latest_development_age", origin.latest_development_age.0)?;
    item.set_item("latest_observed", origin.latest_observed)?;
    item.set_item("cdf_to_ultimate", origin.cdf_to_ultimate)?;
    item.set_item("remaining_factor_product", origin.remaining_factor_product)?;
    item.set_item("tail_factor", origin.tail_factor)?;
    item.set_item("ultimate", origin.ultimate)?;
    item.set_item("reserve", origin.reserve)?;
    Ok(())
}

#[pymodule]
fn _rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(engine_version, m)?)?;
    m.add_function(wrap_pyfunction!(chain_ladder, m)?)?;
    Ok(())
}
