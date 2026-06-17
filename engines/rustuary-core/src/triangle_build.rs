#![allow(clippy::module_name_repetitions)]

use std::collections::BTreeSet;

use crate::error::{ActuarialError, Result};

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

#[cfg(test)]
mod tests {
    use super::{
        TriangleBuildAggregation, TriangleBuildOutputKind, TriangleBuildRequest,
        TriangleBuildRequestInput,
    };
    use crate::ActuarialError;

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
}
