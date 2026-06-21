use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList};
use rustuary_core::{
    build_triangle_set as core_build_triangle_set, BuiltTriangle, BuiltTriangleDiagnostics,
    ClaimEventRecord, ClaimEventRecordInput, TriangleBuildAggregation, TriangleBuildDiagnostics,
    TriangleBuildOutputKind, TriangleBuildRequest, TriangleBuildRequestInput, TriangleKey,
    TriangleSet,
};
use rustuary_core::{
    AppliedLinkRatioExclusion, ChainLadder, ChainLadderResult, CumulativeDevelopmentFactor,
    DevelopmentAge, DevelopmentFactorMethod, DevelopmentFactorOverride, FixedTailFactor,
    OriginChainLadderResult, OriginPeriod, RecordDate, SegmentValue, SelectedDevelopmentFactor,
    Triangle, TriangleBasis,
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

#[pyfunction]
#[pyo3(signature = (request, records))]
fn build_triangle_set(
    py: Python<'_>,
    request: &Bound<'_, PyDict>,
    records: &Bound<'_, PyAny>,
) -> PyResult<Py<PyDict>> {
    let request = triangle_build_request_from_dict(request)?;
    let records = claim_event_records_from_py(records)?;
    let result = core_build_triangle_set(&request, &records).map_err(actuarial_error)?;

    triangle_set_to_dict(py, &result)
}

fn actuarial_error(error: rustuary_core::ActuarialError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn triangle_build_request_from_dict(request: &Bound<'_, PyDict>) -> PyResult<TriangleBuildRequest> {
    let input = TriangleBuildRequestInput {
        triangle_definition_id: required_string(request, "triangle_definition_id")?,
        schema_version: required_string(request, "schema_version")?,
        aggregation: triangle_build_aggregation_from_name(&required_string(
            request,
            "aggregation",
        )?)?,
        bucket_months: required_u8(request, "bucket_months")?,
        output_kind: triangle_build_output_kind_from_name(&required_string(
            request,
            "output_kind",
        )?)?,
        segment_names: required_string_vec(request, "segment_names")?,
    };

    TriangleBuildRequest::new(input).map_err(actuarial_error)
}

fn claim_event_records_from_py(records: &Bound<'_, PyAny>) -> PyResult<Vec<ClaimEventRecord>> {
    let iterator = records
        .iter()
        .map_err(|_| PyValueError::new_err("triangle build records must be iterable"))?;
    let mut parsed_records = Vec::new();
    for (record_index, record) in iterator.enumerate() {
        let record = record?;
        let record = record.downcast::<PyDict>().map_err(|_| {
            PyValueError::new_err(format!(
                "triangle build record {record_index} must be a mapping"
            ))
        })?;
        parsed_records.push(claim_event_record_from_dict(record, record_index)?);
    }

    Ok(parsed_records)
}

fn claim_event_record_from_dict(
    record: &Bound<'_, PyDict>,
    record_index: usize,
) -> PyResult<ClaimEventRecord> {
    let input = ClaimEventRecordInput {
        origin_date: required_record_date(record, "origin_date", record_index)?,
        development_date: required_record_date(record, "development_date", record_index)?,
        amount: optional_record_f64(record, "amount", record_index)?,
        portfolio_id: required_record_string(record, "portfolio_id", record_index)?,
        segments: optional_record_segments(record, "segments", record_index)?,
        measure: required_record_string(record, "measure", record_index)?,
        valuation_date: optional_record_date(record, "valuation_date", record_index)?,
        currency: optional_record_string(record, "currency", record_index)?,
    };

    ClaimEventRecord::new(input).map_err(actuarial_error)
}

fn required_item<'py>(dict: &Bound<'py, PyDict>, field: &str) -> PyResult<Bound<'py, PyAny>> {
    dict.get_item(field)?.ok_or_else(|| {
        PyValueError::new_err(format!("triangle build request is missing `{field}`"))
    })
}

fn required_record_item<'py>(
    dict: &Bound<'py, PyDict>,
    field: &str,
    record_index: usize,
) -> PyResult<Bound<'py, PyAny>> {
    dict.get_item(field)?.ok_or_else(|| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} is missing canonical field `{field}`"
        ))
    })
}

fn optional_record_item<'py>(
    dict: &Bound<'py, PyDict>,
    field: &str,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    Ok(dict.get_item(field)?.filter(|value| !value.is_none()))
}

fn required_string(dict: &Bound<'_, PyDict>, field: &str) -> PyResult<String> {
    let value = required_item(dict, field)?;
    value.extract::<String>().map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build request field `{field}` must be a string"
        ))
    })
}

fn required_u8(dict: &Bound<'_, PyDict>, field: &str) -> PyResult<u8> {
    let value = required_item(dict, field)?;
    value.extract::<u8>().map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build request field `{field}` must be an integer between 0 and 255"
        ))
    })
}

fn required_string_vec(dict: &Bound<'_, PyDict>, field: &str) -> PyResult<Vec<String>> {
    let value = required_item(dict, field)?;
    value.extract::<Vec<String>>().map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build request field `{field}` must be a list of strings"
        ))
    })
}

fn required_record_string(
    record: &Bound<'_, PyDict>,
    field: &str,
    record_index: usize,
) -> PyResult<String> {
    let value = required_record_item(record, field, record_index)?;
    if value.is_none() {
        return Err(PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` is required and cannot be null"
        )));
    }
    value.extract::<String>().map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` must be a string"
        ))
    })
}

fn optional_record_string(
    record: &Bound<'_, PyDict>,
    field: &str,
    record_index: usize,
) -> PyResult<Option<String>> {
    let Some(value) = optional_record_item(record, field)? else {
        return Ok(None);
    };
    value.extract::<String>().map(Some).map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` must be a string"
        ))
    })
}

fn optional_record_f64(
    record: &Bound<'_, PyDict>,
    field: &str,
    record_index: usize,
) -> PyResult<Option<f64>> {
    let Some(value) = optional_record_item(record, field)? else {
        return Ok(None);
    };
    value.extract::<f64>().map(Some).map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` must be numeric"
        ))
    })
}

fn required_record_date(
    record: &Bound<'_, PyDict>,
    field: &str,
    record_index: usize,
) -> PyResult<RecordDate> {
    let value = required_record_item(record, field, record_index)?;
    record_date_from_py(&value, field, record_index)
}

fn optional_record_date(
    record: &Bound<'_, PyDict>,
    field: &str,
    record_index: usize,
) -> PyResult<Option<RecordDate>> {
    let Some(value) = optional_record_item(record, field)? else {
        return Ok(None);
    };
    record_date_from_py(&value, field, record_index).map(Some)
}

fn record_date_from_py(
    value: &Bound<'_, PyAny>,
    field: &str,
    record_index: usize,
) -> PyResult<RecordDate> {
    if value.is_none() {
        return Err(PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` is required and cannot be null"
        )));
    }

    if let Ok(value) = value.extract::<String>() {
        return record_date_from_iso_string(&value, field, record_index);
    }
    if let Ok(value) = value.downcast::<PyDict>() {
        return record_date_from_dict(value, field, record_index);
    }
    if let Ok(value) = value.call_method0("isoformat") {
        if let Ok(value) = value.extract::<String>() {
            return record_date_from_iso_string(&value, field, record_index);
        }
    }

    Err(PyValueError::new_err(format!(
        "triangle build record {record_index} canonical field `{field}` must be a date, ISO date string, or mapping with year, month, and day"
    )))
}

fn record_date_from_iso_string(
    value: &str,
    field: &str,
    record_index: usize,
) -> PyResult<RecordDate> {
    let mut parts = value.split('T').next().unwrap_or(value).split('-');
    let year = parts
        .next()
        .and_then(|part| part.parse::<i32>().ok())
        .ok_or_else(|| invalid_record_date_value(field, record_index))?;
    let month = parts
        .next()
        .and_then(|part| part.parse::<u8>().ok())
        .ok_or_else(|| invalid_record_date_value(field, record_index))?;
    let day = parts
        .next()
        .and_then(|part| part.parse::<u8>().ok())
        .ok_or_else(|| invalid_record_date_value(field, record_index))?;
    if parts.next().is_some() {
        return Err(invalid_record_date_value(field, record_index));
    }

    RecordDate::new(year, month, day).map_err(actuarial_error)
}

fn record_date_from_dict(
    value: &Bound<'_, PyDict>,
    field: &str,
    record_index: usize,
) -> PyResult<RecordDate> {
    let year = required_date_component_i32(value, "year", field, record_index)?;
    let month = required_date_component_u8(value, "month", field, record_index)?;
    let day = required_date_component_u8(value, "day", field, record_index)?;

    RecordDate::new(year, month, day).map_err(actuarial_error)
}

fn required_date_component_i32(
    value: &Bound<'_, PyDict>,
    component: &str,
    field: &str,
    record_index: usize,
) -> PyResult<i32> {
    let item = value.get_item(component)?.ok_or_else(|| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` date is missing `{component}`"
        ))
    })?;
    item.extract::<i32>().map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` date component `{component}` must be an integer"
        ))
    })
}

fn required_date_component_u8(
    value: &Bound<'_, PyDict>,
    component: &str,
    field: &str,
    record_index: usize,
) -> PyResult<u8> {
    let item = value.get_item(component)?.ok_or_else(|| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` date is missing `{component}`"
        ))
    })?;
    item.extract::<u8>().map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` date component `{component}` must be an integer between 0 and 255"
        ))
    })
}

fn invalid_record_date_value(field: &str, record_index: usize) -> PyErr {
    PyValueError::new_err(format!(
        "triangle build record {record_index} canonical field `{field}` must use ISO date format YYYY-MM-DD"
    ))
}

fn optional_record_segments(
    record: &Bound<'_, PyDict>,
    field: &str,
    record_index: usize,
) -> PyResult<Vec<SegmentValue>> {
    let Some(value) = optional_record_item(record, field)? else {
        return Ok(Vec::new());
    };
    let segments = value.downcast::<PyList>().map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} canonical field `{field}` must be a list"
        ))
    })?;
    let mut parsed_segments = Vec::with_capacity(segments.len());
    for (segment_index, segment) in segments.iter().enumerate() {
        let segment = segment.downcast::<PyDict>().map_err(|_| {
            PyValueError::new_err(format!(
                "triangle build record {record_index} segment {segment_index} must be a mapping"
            ))
        })?;
        let name = required_segment_string(segment, "name", record_index, segment_index)?;
        let value = required_segment_string(segment, "value", record_index, segment_index)?;
        parsed_segments.push(SegmentValue::new(name, value).map_err(actuarial_error)?);
    }

    Ok(parsed_segments)
}

fn required_segment_string(
    segment: &Bound<'_, PyDict>,
    field: &str,
    record_index: usize,
    segment_index: usize,
) -> PyResult<String> {
    let item = segment.get_item(field)?.ok_or_else(|| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} segment {segment_index} is missing `{field}`"
        ))
    })?;
    if item.is_none() {
        return Err(PyValueError::new_err(format!(
            "triangle build record {record_index} segment {segment_index} field `{field}` is required and cannot be null"
        )));
    }
    item.extract::<String>().map_err(|_| {
        PyValueError::new_err(format!(
            "triangle build record {record_index} segment {segment_index} field `{field}` must be a string"
        ))
    })
}

fn triangle_build_aggregation_from_name(name: &str) -> PyResult<TriangleBuildAggregation> {
    match name {
        "sum" => Ok(TriangleBuildAggregation::Sum),
        "count" => Ok(TriangleBuildAggregation::Count),
        _ => Err(PyValueError::new_err(
            "triangle build request field `aggregation` must be one of: count, sum",
        )),
    }
}

fn triangle_build_output_kind_from_name(name: &str) -> PyResult<TriangleBuildOutputKind> {
    match name {
        "incremental" => Ok(TriangleBuildOutputKind::Incremental),
        "cumulative" => Ok(TriangleBuildOutputKind::Cumulative),
        _ => Err(PyValueError::new_err(
            "triangle build request field `output_kind` must be one of: cumulative, incremental",
        )),
    }
}

fn triangle_set_to_dict(py: Python<'_>, set: &TriangleSet) -> PyResult<Py<PyDict>> {
    let output = PyDict::new_bound(py);
    output.set_item(
        "diagnostics",
        triangle_build_diagnostics_to_dict(py, set.diagnostics())?,
    )?;
    output.set_item("triangles", built_triangles_to_list(py, set.triangles())?)?;

    Ok(output.unbind())
}

fn triangle_build_diagnostics_to_dict(
    py: Python<'_>,
    diagnostics: TriangleBuildDiagnostics,
) -> PyResult<Bound<'_, PyDict>> {
    let item = PyDict::new_bound(py);
    item.set_item("source_record_count", diagnostics.source_record_count())?;
    item.set_item("triangle_count", diagnostics.triangle_count())?;
    item.set_item(
        "cumulative_conversion_applied",
        diagnostics.cumulative_conversion_applied(),
    )?;
    Ok(item)
}

fn built_triangle_diagnostics_to_dict(
    py: Python<'_>,
    diagnostics: BuiltTriangleDiagnostics,
) -> PyResult<Bound<'_, PyDict>> {
    let item = PyDict::new_bound(py);
    item.set_item("source_record_count", diagnostics.source_record_count())?;
    item.set_item(
        "cumulative_conversion_applied",
        diagnostics.cumulative_conversion_applied(),
    )?;
    Ok(item)
}

fn built_triangles_to_list<'py>(
    py: Python<'py>,
    triangles: &[BuiltTriangle],
) -> PyResult<Bound<'py, PyList>> {
    let items = PyList::empty_bound(py);
    for triangle in triangles {
        items.append(built_triangle_to_dict(py, triangle)?)?;
    }

    Ok(items)
}

fn built_triangle_to_dict<'py>(
    py: Python<'py>,
    built_triangle: &BuiltTriangle,
) -> PyResult<Bound<'py, PyDict>> {
    let item = PyDict::new_bound(py);
    let triangle = built_triangle.triangle();
    item.set_item("key", triangle_key_to_dict(py, built_triangle.key())?)?;
    item.set_item(
        "origin_periods",
        triangle
            .origin_periods()
            .iter()
            .map(|period| period.0)
            .collect::<Vec<_>>(),
    )?;
    item.set_item(
        "development_ages",
        triangle
            .development_ages()
            .iter()
            .map(|age| age.0)
            .collect::<Vec<_>>(),
    )?;
    item.set_item("rows", triangle_rows_to_list(py, triangle)?)?;
    item.set_item("basis", triangle_basis_name(triangle.basis()))?;
    item.set_item(
        "diagnostics",
        built_triangle_diagnostics_to_dict(py, built_triangle.diagnostics())?,
    )?;

    Ok(item)
}

fn triangle_key_to_dict<'py>(py: Python<'py>, key: &TriangleKey) -> PyResult<Bound<'py, PyDict>> {
    let item = PyDict::new_bound(py);
    item.set_item("portfolio_id", key.portfolio_id())?;
    item.set_item("segments", segment_values_to_list(py, key.segments())?)?;
    item.set_item("measure", key.measure())?;
    item.set_item("display_path", key.display_path())?;
    Ok(item)
}

fn segment_values_to_list<'py>(
    py: Python<'py>,
    segments: &[SegmentValue],
) -> PyResult<Bound<'py, PyList>> {
    let items = PyList::empty_bound(py);
    for segment in segments {
        let item = PyDict::new_bound(py);
        item.set_item("name", segment.name())?;
        item.set_item("value", segment.value())?;
        items.append(item)?;
    }

    Ok(items)
}

fn triangle_rows_to_list<'py>(
    py: Python<'py>,
    triangle: &Triangle,
) -> PyResult<Bound<'py, PyList>> {
    let rows = PyList::empty_bound(py);
    for row_index in 0..triangle.row_count() {
        let row = (0..triangle.col_count())
            .map(|col_index| triangle.get(row_index, col_index))
            .collect::<Vec<_>>();
        rows.append(row)?;
    }

    Ok(rows)
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

const fn triangle_basis_name(basis: TriangleBasis) -> &'static str {
    match basis {
        TriangleBasis::Cumulative => "cumulative",
        TriangleBasis::Incremental => "incremental",
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
    m.add_function(wrap_pyfunction!(build_triangle_set, m)?)?;
    Ok(())
}
