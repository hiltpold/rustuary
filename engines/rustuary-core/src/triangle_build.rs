#![allow(clippy::module_name_repetitions)]

use std::collections::{BTreeMap, BTreeSet};

use crate::error::{ActuarialError, Result};
use crate::raw_claim::{ClaimEventRecord, RecordDate, SegmentValue};
use crate::triangle::{Triangle, TriangleBasis};
use crate::types::{DevelopmentAge, OriginPeriod};

/// Aggregation used to convert canonical claim/event records into triangle cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriangleBuildAggregation {
    /// Sum finite monetary amounts into each triangle cell.
    Sum,
    /// Count one event for each input record.
    Count,
}

impl TriangleBuildAggregation {
    /// Return whether this aggregation requires an amount on each input record.
    #[must_use]
    pub const fn requires_amount(self) -> bool {
        matches!(self, Self::Sum)
    }
}

/// Requested output basis for a Rust-built triangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriangleBuildOutputKind {
    /// Return incremental cell values.
    Incremental,
    /// Return cumulative cell values.
    Cumulative,
}

/// Input bag for [`TriangleBuildRequest::new`].
///
/// This is the Rust mirror of the non-source-column `TriangleDefinition`
/// semantics needed by the core after adapter-level mapping. It intentionally
/// does not carry dataframe source column names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriangleBuildRequestInput {
    /// Stable triangle-definition identifier for audit and reproducibility.
    pub triangle_definition_id: String,
    /// Version of the logical triangle-definition schema.
    pub schema_version: String,
    /// Cell aggregation semantics.
    pub aggregation: TriangleBuildAggregation,
    /// Development bucket size in months.
    pub bucket_months: u8,
    /// Requested output triangle basis.
    pub output_kind: TriangleBuildOutputKind,
    /// Ordered segment names from the `TriangleDefinition`.
    pub segment_names: Vec<String>,
}

/// Validated Rust build request for raw claim/event triangle construction.
///
/// Python, CLI, or service adapters build this from `TriangleDefinition` after
/// validating and resolving source-column mappings. Rust construction uses it
/// with canonical claim/event records to drive bucketing, aggregation,
/// grouping-key validation, and basis conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriangleBuildRequest {
    triangle_definition_id: String,
    schema_version: String,
    aggregation: TriangleBuildAggregation,
    bucket_months: u8,
    output_kind: TriangleBuildOutputKind,
    segment_names: Vec<String>,
}

impl TriangleBuildRequest {
    /// Construct a validated triangle build request.
    pub fn new(input: TriangleBuildRequestInput) -> Result<Self> {
        validate_non_empty("triangle_definition_id", &input.triangle_definition_id)?;
        validate_non_empty("schema_version", &input.schema_version)?;
        if input.bucket_months == 0 || input.bucket_months > 12 {
            return Err(ActuarialError::InvalidTriangleBuildBucketMonths {
                bucket_months: input.bucket_months,
            });
        }
        validate_segment_names(&input.segment_names)?;

        Ok(Self {
            triangle_definition_id: input.triangle_definition_id,
            schema_version: input.schema_version,
            aggregation: input.aggregation,
            bucket_months: input.bucket_months,
            output_kind: input.output_kind,
            segment_names: input.segment_names,
        })
    }

    /// Return the source `TriangleDefinition` identifier.
    #[must_use]
    pub fn triangle_definition_id(&self) -> &str {
        &self.triangle_definition_id
    }

    /// Return the logical schema version.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Return the requested aggregation semantics.
    #[must_use]
    pub const fn aggregation(&self) -> TriangleBuildAggregation {
        self.aggregation
    }

    /// Return development bucket size in months.
    #[must_use]
    pub const fn bucket_months(&self) -> u8 {
        self.bucket_months
    }

    /// Return the requested output triangle basis.
    #[must_use]
    pub const fn output_kind(&self) -> TriangleBuildOutputKind {
        self.output_kind
    }

    /// Return the ordered segment names from the build definition.
    #[must_use]
    pub fn segment_names(&self) -> &[String] {
        &self.segment_names
    }
}

/// Deterministic grouping key for one built triangle.
///
/// The key is derived from `portfolio_id + ordered segments + measure`.
/// Display paths are derived from this structured key and are not stored as
/// independent canonical truth.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TriangleKey {
    portfolio_id: String,
    segments: Vec<SegmentValue>,
    measure: String,
}

impl TriangleKey {
    fn from_record(
        request: &TriangleBuildRequest,
        record: &ClaimEventRecord,
        record_index: usize,
    ) -> Result<Self> {
        validate_record_segments(request, record, record_index)?;

        Ok(Self {
            portfolio_id: record.portfolio_id().to_owned(),
            segments: record.segments().to_vec(),
            measure: record.measure().to_owned(),
        })
    }

    /// Return the main reserving class / actuarial reserving unit.
    #[must_use]
    pub fn portfolio_id(&self) -> &str {
        &self.portfolio_id
    }

    /// Return ordered segment values.
    #[must_use]
    pub fn segments(&self) -> &[SegmentValue] {
        &self.segments
    }

    /// Return the claims measure.
    #[must_use]
    pub fn measure(&self) -> &str {
        &self.measure
    }

    /// Return a display/folder path derived from portfolio and segment values.
    #[must_use]
    pub fn display_path(&self) -> String {
        let mut parts = Vec::with_capacity(1 + self.segments.len());
        parts.push(self.portfolio_id.as_str());
        parts.extend(self.segments.iter().map(SegmentValue::value));
        parts.join("/")
    }
}

/// Diagnostics for the complete triangle-set build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriangleBuildDiagnostics {
    source_record_count: usize,
    triangle_count: usize,
    cumulative_conversion_applied: bool,
}

impl TriangleBuildDiagnostics {
    /// Return the number of canonical claim/event records consumed.
    #[must_use]
    pub const fn source_record_count(&self) -> usize {
        self.source_record_count
    }

    /// Return the number of triangles built.
    #[must_use]
    pub const fn triangle_count(&self) -> usize {
        self.triangle_count
    }

    /// Return whether incremental cells were converted to cumulative output.
    #[must_use]
    pub const fn cumulative_conversion_applied(&self) -> bool {
        self.cumulative_conversion_applied
    }
}

/// Diagnostics for one built triangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltTriangleDiagnostics {
    source_record_count: usize,
    cumulative_conversion_applied: bool,
}

impl BuiltTriangleDiagnostics {
    /// Return the number of source records included in this triangle.
    #[must_use]
    pub const fn source_record_count(&self) -> usize {
        self.source_record_count
    }

    /// Return whether incremental cells were converted to cumulative output.
    #[must_use]
    pub const fn cumulative_conversion_applied(&self) -> bool {
        self.cumulative_conversion_applied
    }
}

/// One triangle built for a deterministic [`TriangleKey`].
#[derive(Debug, Clone, PartialEq)]
pub struct BuiltTriangle {
    key: TriangleKey,
    triangle: Triangle,
    diagnostics: BuiltTriangleDiagnostics,
}

impl BuiltTriangle {
    /// Return this triangle's deterministic grouping key.
    #[must_use]
    pub fn key(&self) -> &TriangleKey {
        &self.key
    }

    /// Return the canonical dense triangle.
    #[must_use]
    pub fn triangle(&self) -> &Triangle {
        &self.triangle
    }

    /// Return build diagnostics for this triangle.
    #[must_use]
    pub const fn diagnostics(&self) -> BuiltTriangleDiagnostics {
        self.diagnostics
    }
}

/// Collection of triangles built from one canonical claim/event dataset.
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleSet {
    triangles: Vec<BuiltTriangle>,
    diagnostics: TriangleBuildDiagnostics,
}

impl TriangleSet {
    /// Return built triangles in deterministic key order.
    #[must_use]
    pub fn triangles(&self) -> &[BuiltTriangle] {
        &self.triangles
    }

    /// Return an iterator over deterministic grouping keys.
    pub fn keys(&self) -> impl Iterator<Item = &TriangleKey> {
        self.triangles.iter().map(BuiltTriangle::key)
    }

    /// Return the triangle for a key, if present.
    #[must_use]
    pub fn get(&self, key: &TriangleKey) -> Option<&Triangle> {
        self.triangles
            .iter()
            .find(|entry| entry.key() == key)
            .map(BuiltTriangle::triangle)
    }

    /// Return build diagnostics for the complete set.
    #[must_use]
    pub const fn diagnostics(&self) -> TriangleBuildDiagnostics {
        self.diagnostics
    }
}

/// Build deterministic triangles from canonical claim/event records.
///
/// Records are always aggregated into incremental cells first. Cumulative
/// output is produced by an explicit row-wise conversion when requested.
pub fn build_triangle_set(
    request: &TriangleBuildRequest,
    records: &[ClaimEventRecord],
) -> Result<TriangleSet> {
    validate_supported_bucket_months(request.bucket_months())?;
    if records.is_empty() {
        return Err(ActuarialError::EmptyTriangleBuildRecords);
    }

    let mut groups = BTreeMap::<TriangleKey, GroupAccumulator>::new();
    for (record_index, record) in records.iter().enumerate() {
        let key = TriangleKey::from_record(request, record, record_index)?;
        let origin_period =
            origin_period_from_date(record.origin_date(), request.bucket_months(), record_index)?;
        let development_age = development_age_from_dates(
            record.origin_date(),
            record.development_date(),
            request.bucket_months(),
            record_index,
        )?;
        let contribution = contribution_from_record(request, record, record_index)?;

        groups.entry(key).or_default().add_cell(
            origin_period,
            development_age,
            contribution,
            record_index,
        )?;
    }

    let cumulative_conversion_applied =
        request.output_kind() == TriangleBuildOutputKind::Cumulative;
    let triangles =
        finalize_triangle_groups(groups, request.output_kind(), cumulative_conversion_applied)?;

    Ok(TriangleSet {
        diagnostics: TriangleBuildDiagnostics {
            source_record_count: records.len(),
            triangle_count: triangles.len(),
            cumulative_conversion_applied,
        },
        triangles,
    })
}

fn finalize_triangle_groups(
    groups: BTreeMap<TriangleKey, GroupAccumulator>,
    output_kind: TriangleBuildOutputKind,
    cumulative_conversion_applied: bool,
) -> Result<Vec<BuiltTriangle>> {
    // Each accumulated group is independent after record validation and
    // grouping. This boundary is the future Rust-only parallelization point.
    groups
        .into_iter()
        .map(|(key, accumulator)| {
            finalize_triangle_group(key, accumulator, output_kind, cumulative_conversion_applied)
        })
        .collect()
}

fn finalize_triangle_group(
    key: TriangleKey,
    accumulator: GroupAccumulator,
    output_kind: TriangleBuildOutputKind,
    cumulative_conversion_applied: bool,
) -> Result<BuiltTriangle> {
    let GroupAccumulator {
        cells,
        record_count: source_record_count,
    } = accumulator;
    let triangle = build_triangle_from_cells(&cells, output_kind)?;
    Ok(BuiltTriangle {
        key,
        triangle,
        diagnostics: BuiltTriangleDiagnostics {
            source_record_count,
            cumulative_conversion_applied,
        },
    })
}

#[derive(Debug, Default)]
struct GroupAccumulator {
    cells: BTreeMap<(OriginPeriod, DevelopmentAge), f64>,
    record_count: usize,
}

impl GroupAccumulator {
    fn add_cell(
        &mut self,
        origin_period: OriginPeriod,
        development_age: DevelopmentAge,
        contribution: f64,
        record_index: usize,
    ) -> Result<()> {
        let cell = self
            .cells
            .entry((origin_period, development_age))
            .or_insert(0.0);
        let aggregate = *cell + contribution;
        if !aggregate.is_finite() {
            return Err(ActuarialError::NonFiniteTriangleBuildCell { record_index });
        }
        *cell = aggregate;
        self.record_count += 1;
        Ok(())
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(ActuarialError::EmptyTriangleBuildRequestField { field });
    }
    Ok(())
}

fn validate_segment_names(segment_names: &[String]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for name in segment_names {
        validate_non_empty("segments.name", name)?;
        if !seen.insert(name.as_str()) {
            return Err(ActuarialError::DuplicateTriangleBuildSegmentName { name: name.clone() });
        }
    }
    Ok(())
}

fn validate_record_segments(
    request: &TriangleBuildRequest,
    record: &ClaimEventRecord,
    record_index: usize,
) -> Result<()> {
    if record.segments().len() != request.segment_names().len() {
        return Err(ActuarialError::ClaimEventSegmentCountMismatch {
            record_index,
            expected: request.segment_names().len(),
            actual: record.segments().len(),
        });
    }

    for (segment_index, (segment, expected)) in record
        .segments()
        .iter()
        .zip(request.segment_names().iter())
        .enumerate()
    {
        if segment.name() != expected {
            return Err(ActuarialError::ClaimEventSegmentNameMismatch {
                record_index,
                segment_index,
                expected: expected.clone(),
                actual: segment.name().to_owned(),
            });
        }
    }
    Ok(())
}

fn contribution_from_record(
    request: &TriangleBuildRequest,
    record: &ClaimEventRecord,
    record_index: usize,
) -> Result<f64> {
    match request.aggregation() {
        TriangleBuildAggregation::Sum => record
            .amount()
            .ok_or(ActuarialError::MissingClaimEventAmount { record_index }),
        TriangleBuildAggregation::Count => Ok(1.0),
    }
}

fn validate_supported_bucket_months(bucket_months: u8) -> Result<()> {
    match bucket_months {
        1 | 3 | 6 | 12 => Ok(()),
        _ => Err(ActuarialError::UnsupportedTriangleBuildBucketMonths { bucket_months }),
    }
}

fn origin_period_from_date(
    date: RecordDate,
    bucket_months: u8,
    record_index: usize,
) -> Result<OriginPeriod> {
    let start_month = bucket_start_month(date.month(), bucket_months);
    if bucket_months == 12 {
        return Ok(OriginPeriod(date.year()));
    }

    let label = date
        .year()
        .checked_mul(100)
        .and_then(|year| year.checked_add(i32::from(start_month)))
        .ok_or(ActuarialError::ClaimEventOriginPeriodOverflow { record_index })?;
    Ok(OriginPeriod(label))
}

fn development_age_from_dates(
    origin_date: RecordDate,
    development_date: RecordDate,
    bucket_months: u8,
    record_index: usize,
) -> Result<DevelopmentAge> {
    let origin_start_month = bucket_start_month(origin_date.month(), bucket_months);
    let development_start_month = bucket_start_month(development_date.month(), bucket_months);
    let origin_month_index = month_index(origin_date.year(), origin_start_month);
    let development_month_index = month_index(development_date.year(), development_start_month);
    let age_months = development_month_index - origin_month_index + i64::from(bucket_months);

    if age_months <= 0 {
        return Err(ActuarialError::NegativeClaimEventDevelopmentAge { record_index });
    }
    let age = u32::try_from(age_months)
        .map_err(|_| ActuarialError::ClaimEventDevelopmentAgeOverflow { record_index })?;
    Ok(DevelopmentAge(age))
}

fn month_index(year: i32, month: u8) -> i64 {
    i64::from(year) * 12 + i64::from(month) - 1
}

fn bucket_start_month(month: u8, bucket_months: u8) -> u8 {
    ((month - 1) / bucket_months) * bucket_months + 1
}

fn build_triangle_from_cells(
    cells: &BTreeMap<(OriginPeriod, DevelopmentAge), f64>,
    output_kind: TriangleBuildOutputKind,
) -> Result<Triangle> {
    let origin_periods = cells
        .keys()
        .map(|(origin_period, _)| *origin_period)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let development_ages = cells
        .keys()
        .map(|(_, development_age)| *development_age)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let latest_by_origin = latest_development_by_origin(cells);

    let rows = origin_periods
        .iter()
        .map(|origin_period| {
            let latest_development_age = latest_by_origin
                .get(origin_period)
                .copied()
                .expect("origin axis is derived from non-empty cell map");
            development_ages
                .iter()
                .map(|development_age| {
                    if *development_age <= latest_development_age {
                        Some(
                            cells
                                .get(&(*origin_period, *development_age))
                                .copied()
                                .unwrap_or(0.0),
                        )
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let incremental = Triangle::new(
        origin_periods,
        development_ages,
        rows,
        TriangleBasis::Incremental,
    )?;
    match output_kind {
        TriangleBuildOutputKind::Incremental => Ok(incremental),
        TriangleBuildOutputKind::Cumulative => incremental.to_cumulative(),
    }
}

fn latest_development_by_origin(
    cells: &BTreeMap<(OriginPeriod, DevelopmentAge), f64>,
) -> BTreeMap<OriginPeriod, DevelopmentAge> {
    let mut latest_by_origin = BTreeMap::new();
    for (origin_period, development_age) in cells.keys().copied() {
        latest_by_origin
            .entry(origin_period)
            .and_modify(|latest| {
                if development_age > *latest {
                    *latest = development_age;
                }
            })
            .or_insert(development_age);
    }
    latest_by_origin
}

#[cfg(test)]
mod tests {
    use super::{
        build_triangle_set, TriangleBuildAggregation, TriangleBuildOutputKind,
        TriangleBuildRequest, TriangleBuildRequestInput,
    };
    use crate::{
        ActuarialError, ClaimEventRecord, ClaimEventRecordInput, DevelopmentAge, OriginPeriod,
        RecordDate, SegmentValue, TriangleBasis,
    };

    fn assert_close(actual: f64, expected: f64) {
        let difference = (actual - expected).abs();
        assert!(
            difference <= 1e-9,
            "actual={actual}, expected={expected}, difference={difference}"
        );
    }

    fn date(year: i32, month: u8, day: u8) -> RecordDate {
        RecordDate::new(year, month, day).expect("test date should be valid")
    }

    fn segment(name: &str, value: &str) -> SegmentValue {
        SegmentValue::new(name, value).expect("test segment should be valid")
    }

    fn claim_event_record(
        origin_date: RecordDate,
        development_date: RecordDate,
        amount: Option<f64>,
        portfolio_id: &str,
        segments: Vec<SegmentValue>,
        measure: &str,
    ) -> ClaimEventRecord {
        ClaimEventRecord::new(ClaimEventRecordInput {
            origin_date,
            development_date,
            amount,
            portfolio_id: portfolio_id.to_owned(),
            segments,
            measure: measure.to_owned(),
            valuation_date: None,
            currency: None,
        })
        .expect("test claim/event record should be valid")
    }

    fn build_request(
        aggregation: TriangleBuildAggregation,
        bucket_months: u8,
        output_kind: TriangleBuildOutputKind,
        segment_names: &[&str],
    ) -> TriangleBuildRequest {
        TriangleBuildRequest::new(TriangleBuildRequestInput {
            triangle_definition_id: "paid-claims-v1".to_owned(),
            schema_version: "1".to_owned(),
            aggregation,
            bucket_months,
            output_kind,
            segment_names: segment_names
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
        })
        .expect("test build request should be valid")
    }

    fn sample_request_input() -> TriangleBuildRequestInput {
        TriangleBuildRequestInput {
            triangle_definition_id: "paid-claims-v1".to_owned(),
            schema_version: "1".to_owned(),
            aggregation: TriangleBuildAggregation::Sum,
            bucket_months: 12,
            output_kind: TriangleBuildOutputKind::Cumulative,
            segment_names: vec!["country".to_owned(), "coverage".to_owned()],
        }
    }

    #[test]
    fn triangle_build_request_preserves_definition_semantics() {
        let request =
            TriangleBuildRequest::new(sample_request_input()).expect("request should be valid");

        assert_eq!(request.triangle_definition_id(), "paid-claims-v1");
        assert_eq!(request.schema_version(), "1");
        assert_eq!(request.aggregation(), TriangleBuildAggregation::Sum);
        assert!(request.aggregation().requires_amount());
        assert_eq!(request.bucket_months(), 12);
        assert_eq!(request.output_kind(), TriangleBuildOutputKind::Cumulative);
        assert_eq!(request.segment_names(), ["country", "coverage"]);
    }

    #[test]
    fn triangle_build_request_allows_count_without_amount_requirement() {
        let mut input = sample_request_input();
        input.aggregation = TriangleBuildAggregation::Count;
        input.output_kind = TriangleBuildOutputKind::Incremental;
        input.segment_names.clear();

        let request = TriangleBuildRequest::new(input).expect("count request should be valid");

        assert_eq!(request.aggregation(), TriangleBuildAggregation::Count);
        assert!(!request.aggregation().requires_amount());
        assert_eq!(request.output_kind(), TriangleBuildOutputKind::Incremental);
        assert!(request.segment_names().is_empty());
    }

    #[test]
    fn triangle_build_request_rejects_blank_identifiers() {
        let mut input = sample_request_input();
        " ".clone_into(&mut input.triangle_definition_id);

        assert_eq!(
            TriangleBuildRequest::new(input).expect_err("blank id should fail"),
            ActuarialError::EmptyTriangleBuildRequestField {
                field: "triangle_definition_id"
            }
        );

        let mut input = sample_request_input();
        String::new().clone_into(&mut input.schema_version);

        assert_eq!(
            TriangleBuildRequest::new(input).expect_err("blank schema version should fail"),
            ActuarialError::EmptyTriangleBuildRequestField {
                field: "schema_version"
            }
        );
    }

    #[test]
    fn triangle_build_request_rejects_invalid_bucket_months() {
        let mut input = sample_request_input();
        input.bucket_months = 0;

        assert_eq!(
            TriangleBuildRequest::new(input).expect_err("zero bucket size should fail"),
            ActuarialError::InvalidTriangleBuildBucketMonths { bucket_months: 0 }
        );

        let mut input = sample_request_input();
        input.bucket_months = 13;

        assert_eq!(
            TriangleBuildRequest::new(input).expect_err("large bucket size should fail"),
            ActuarialError::InvalidTriangleBuildBucketMonths { bucket_months: 13 }
        );
    }

    #[test]
    fn triangle_build_request_rejects_blank_or_duplicate_segment_names() {
        let mut input = sample_request_input();
        input.segment_names = vec!["country".to_owned(), " ".to_owned()];

        assert_eq!(
            TriangleBuildRequest::new(input).expect_err("blank segment name should fail"),
            ActuarialError::EmptyTriangleBuildRequestField {
                field: "segments.name"
            }
        );

        let mut input = sample_request_input();
        input.segment_names = vec!["country".to_owned(), "country".to_owned()];

        assert_eq!(
            TriangleBuildRequest::new(input).expect_err("duplicate segment name should fail"),
            ActuarialError::DuplicateTriangleBuildSegmentName {
                name: "country".to_owned()
            }
        );
    }

    #[test]
    fn build_triangle_set_sums_raw_records_into_incremental_triangles() {
        // Expected values are hand-calculated from small synthetic claim events.
        let request = build_request(
            TriangleBuildAggregation::Sum,
            12,
            TriangleBuildOutputKind::Incremental,
            &["country"],
        );
        let records = vec![
            claim_event_record(
                date(2024, 1, 15),
                date(2024, 3, 10),
                Some(100.0),
                "Motor",
                vec![segment("country", "CH")],
                "paid",
            ),
            claim_event_record(
                date(2024, 5, 1),
                date(2025, 2, 1),
                Some(50.0),
                "Motor",
                vec![segment("country", "CH")],
                "paid",
            ),
            claim_event_record(
                date(2025, 1, 1),
                date(2025, 6, 1),
                Some(80.0),
                "Motor",
                vec![segment("country", "CH")],
                "paid",
            ),
        ];

        let set = build_triangle_set(&request, &records).expect("triangle set should build");

        assert_eq!(set.diagnostics().source_record_count(), 3);
        assert_eq!(set.diagnostics().triangle_count(), 1);
        assert!(!set.diagnostics().cumulative_conversion_applied());
        let entry = &set.triangles()[0];
        assert_eq!(entry.key().portfolio_id(), "Motor");
        assert_eq!(entry.key().segments()[0].name(), "country");
        assert_eq!(entry.key().segments()[0].value(), "CH");
        assert_eq!(entry.key().measure(), "paid");
        assert_eq!(entry.key().display_path(), "Motor/CH");
        assert_eq!(
            set.keys()
                .next()
                .expect("triangle set should have one key")
                .display_path(),
            "Motor/CH"
        );
        assert!(set.get(entry.key()).is_some());

        let triangle = entry.triangle();
        assert_eq!(triangle.basis(), TriangleBasis::Incremental);
        assert_eq!(
            triangle.origin_periods(),
            &[OriginPeriod(2024), OriginPeriod(2025)]
        );
        assert_eq!(
            triangle.development_ages(),
            &[DevelopmentAge(12), DevelopmentAge(24)]
        );
        assert_close(triangle.get(0, 0).expect("cell should be observed"), 100.0);
        assert_close(triangle.get(0, 1).expect("cell should be observed"), 50.0);
        assert_close(triangle.get(1, 0).expect("cell should be observed"), 80.0);
        assert_eq!(triangle.get(1, 1), None);
        assert_eq!(entry.diagnostics().source_record_count(), 3);
        assert!(!entry.diagnostics().cumulative_conversion_applied());
    }

    #[test]
    fn build_triangle_set_fills_missing_observed_cells_with_zero() {
        let request = build_request(
            TriangleBuildAggregation::Sum,
            12,
            TriangleBuildOutputKind::Incremental,
            &[],
        );
        let records = vec![
            claim_event_record(
                date(2024, 1, 15),
                date(2025, 3, 10),
                Some(50.0),
                "Motor",
                vec![],
                "paid",
            ),
            claim_event_record(
                date(2025, 1, 15),
                date(2025, 3, 10),
                Some(80.0),
                "Motor",
                vec![],
                "paid",
            ),
        ];

        let set = build_triangle_set(&request, &records).expect("triangle set should build");
        let triangle = set.triangles()[0].triangle();

        assert_eq!(
            triangle.development_ages(),
            &[DevelopmentAge(12), DevelopmentAge(24)]
        );
        assert_close(triangle.get(0, 0).expect("zero cell should be filled"), 0.0);
        assert_close(triangle.get(0, 1).expect("cell should be observed"), 50.0);
        assert_close(triangle.get(1, 0).expect("cell should be observed"), 80.0);
        assert_eq!(triangle.get(1, 1), None);
    }

    #[test]
    fn build_triangle_set_converts_incremental_cells_to_cumulative_output() {
        let request = build_request(
            TriangleBuildAggregation::Sum,
            12,
            TriangleBuildOutputKind::Cumulative,
            &[],
        );
        let records = vec![
            claim_event_record(
                date(2024, 1, 15),
                date(2024, 3, 10),
                Some(100.0),
                "Motor",
                vec![],
                "paid",
            ),
            claim_event_record(
                date(2024, 1, 15),
                date(2025, 3, 10),
                Some(50.0),
                "Motor",
                vec![],
                "paid",
            ),
        ];

        let set = build_triangle_set(&request, &records).expect("triangle set should build");
        let triangle = set.triangles()[0].triangle();

        assert_eq!(triangle.basis(), TriangleBasis::Cumulative);
        assert!(set.diagnostics().cumulative_conversion_applied());
        assert!(set.triangles()[0]
            .diagnostics()
            .cumulative_conversion_applied());
        assert_close(triangle.get(0, 0).expect("cell should be observed"), 100.0);
        assert_close(triangle.get(0, 1).expect("cell should be observed"), 150.0);
    }

    #[test]
    fn build_triangle_set_counts_records_without_amounts() {
        let request = build_request(
            TriangleBuildAggregation::Count,
            12,
            TriangleBuildOutputKind::Incremental,
            &[],
        );
        let records = vec![
            claim_event_record(
                date(2024, 1, 15),
                date(2024, 3, 10),
                None,
                "Motor",
                vec![],
                "reported_count",
            ),
            claim_event_record(
                date(2024, 2, 15),
                date(2024, 4, 10),
                None,
                "Motor",
                vec![],
                "reported_count",
            ),
            claim_event_record(
                date(2024, 2, 15),
                date(2025, 4, 10),
                None,
                "Motor",
                vec![],
                "reported_count",
            ),
        ];

        let set = build_triangle_set(&request, &records).expect("triangle set should build");
        let entry = &set.triangles()[0];

        assert_eq!(entry.key().display_path(), "Motor");
        assert_close(
            entry.triangle().get(0, 0).expect("cell should be observed"),
            2.0,
        );
        assert_close(
            entry.triangle().get(0, 1).expect("cell should be observed"),
            1.0,
        );
    }

    #[test]
    fn build_triangle_set_groups_records_by_key() {
        let request = build_request(
            TriangleBuildAggregation::Sum,
            12,
            TriangleBuildOutputKind::Incremental,
            &["country"],
        );
        let records = vec![
            claim_event_record(
                date(2024, 1, 15),
                date(2024, 3, 10),
                Some(100.0),
                "Motor",
                vec![segment("country", "CH")],
                "paid",
            ),
            claim_event_record(
                date(2024, 1, 15),
                date(2024, 3, 10),
                Some(80.0),
                "Property",
                vec![segment("country", "CH")],
                "paid",
            ),
        ];

        let set = build_triangle_set(&request, &records).expect("triangle set should build");
        let display_paths = set
            .keys()
            .map(super::TriangleKey::display_path)
            .collect::<Vec<_>>();

        assert_eq!(set.diagnostics().triangle_count(), 2);
        assert_eq!(display_paths, vec!["Motor/CH", "Property/CH"]);
    }

    #[test]
    fn build_triangle_set_supports_configured_bucket_sizes() {
        let cases = [
            (1, OriginPeriod(202_402), DevelopmentAge(7)),
            (3, OriginPeriod(202_401), DevelopmentAge(9)),
            (6, OriginPeriod(202_401), DevelopmentAge(12)),
            (12, OriginPeriod(2024), DevelopmentAge(12)),
        ];

        for (bucket_months, expected_origin, expected_development) in cases {
            let request = build_request(
                TriangleBuildAggregation::Count,
                bucket_months,
                TriangleBuildOutputKind::Incremental,
                &[],
            );
            let records = vec![claim_event_record(
                date(2024, 2, 15),
                date(2024, 8, 1),
                None,
                "Motor",
                vec![],
                "reported_count",
            )];

            let set = build_triangle_set(&request, &records)
                .expect("configured bucket size should build");
            let triangle = set.triangles()[0].triangle();

            assert_eq!(triangle.origin_periods(), &[expected_origin]);
            assert_eq!(triangle.development_ages(), &[expected_development]);
        }
    }

    #[test]
    fn build_triangle_set_rejects_unsupported_bucket_sizes() {
        let request = build_request(
            TriangleBuildAggregation::Count,
            2,
            TriangleBuildOutputKind::Incremental,
            &[],
        );
        let records = vec![claim_event_record(
            date(2024, 2, 15),
            date(2024, 8, 1),
            None,
            "Motor",
            vec![],
            "reported_count",
        )];

        assert_eq!(
            build_triangle_set(&request, &records).expect_err("unsupported bucket should fail"),
            ActuarialError::UnsupportedTriangleBuildBucketMonths { bucket_months: 2 }
        );
    }

    #[test]
    fn build_triangle_set_rejects_negative_development_age() {
        let request = build_request(
            TriangleBuildAggregation::Count,
            12,
            TriangleBuildOutputKind::Incremental,
            &[],
        );
        let records = vec![claim_event_record(
            date(2024, 1, 15),
            date(2023, 12, 31),
            None,
            "Motor",
            vec![],
            "reported_count",
        )];

        assert_eq!(
            build_triangle_set(&request, &records).expect_err("negative development should fail"),
            ActuarialError::NegativeClaimEventDevelopmentAge { record_index: 0 }
        );
    }

    #[test]
    fn build_triangle_set_rejects_missing_amount_for_sum() {
        let request = build_request(
            TriangleBuildAggregation::Sum,
            12,
            TriangleBuildOutputKind::Incremental,
            &[],
        );
        let records = vec![claim_event_record(
            date(2024, 1, 15),
            date(2024, 3, 10),
            None,
            "Motor",
            vec![],
            "paid",
        )];

        assert_eq!(
            build_triangle_set(&request, &records).expect_err("missing amount should fail"),
            ActuarialError::MissingClaimEventAmount { record_index: 0 }
        );
    }

    #[test]
    fn build_triangle_set_rejects_segment_mismatches() {
        let request = build_request(
            TriangleBuildAggregation::Count,
            12,
            TriangleBuildOutputKind::Incremental,
            &["country"],
        );
        let records = vec![claim_event_record(
            date(2024, 1, 15),
            date(2024, 3, 10),
            None,
            "Motor",
            vec![],
            "reported_count",
        )];

        assert_eq!(
            build_triangle_set(&request, &records).expect_err("segment count should fail"),
            ActuarialError::ClaimEventSegmentCountMismatch {
                record_index: 0,
                expected: 1,
                actual: 0,
            }
        );

        let records = vec![claim_event_record(
            date(2024, 1, 15),
            date(2024, 3, 10),
            None,
            "Motor",
            vec![segment("coverage", "MTPL")],
            "reported_count",
        )];

        assert_eq!(
            build_triangle_set(&request, &records).expect_err("segment name should fail"),
            ActuarialError::ClaimEventSegmentNameMismatch {
                record_index: 0,
                segment_index: 0,
                expected: "country".to_owned(),
                actual: "coverage".to_owned(),
            }
        );
    }

    #[test]
    fn build_triangle_set_rejects_empty_inputs() {
        let request = build_request(
            TriangleBuildAggregation::Count,
            12,
            TriangleBuildOutputKind::Incremental,
            &[],
        );

        assert_eq!(
            build_triangle_set(&request, &[]).expect_err("empty records should fail"),
            ActuarialError::EmptyTriangleBuildRecords
        );
    }
}
