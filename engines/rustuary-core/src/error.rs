use crate::types::{DevelopmentAge, OriginPeriod};
use thiserror::Error;

/// Result type returned by fallible actuarial core operations.
pub type Result<T> = std::result::Result<T, ActuarialError>;

/// Validation and calculation errors returned by the actuarial core.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ActuarialError {
    #[error("triangle must contain at least one origin period")]
    EmptyTriangle,

    #[error("triangle must contain at least one development age")]
    EmptyDevelopmentAxis,

    #[error("triangle has {row_count} rows but {origin_count} origin periods")]
    OriginAxisLengthMismatch {
        origin_count: usize,
        row_count: usize,
    },

    #[error("origin row {origin_index} has {actual} cells; expected {expected} development ages")]
    RaggedTriangle {
        origin_index: usize,
        expected: usize,
        actual: usize,
    },

    #[error("origin periods must be strictly increasing; {current:?} follows {previous:?}")]
    UnorderedOriginPeriods {
        previous: OriginPeriod,
        current: OriginPeriod,
    },

    #[error("development ages must be strictly increasing; {current:?} follows {previous:?}")]
    UnorderedDevelopmentAges {
        previous: DevelopmentAge,
        current: DevelopmentAge,
    },

    #[error(
        "triangle value at origin row {origin_index}, development column {development_index} must be finite"
    )]
    NonFiniteTriangleValue {
        origin_index: usize,
        development_index: usize,
    },

    #[error(
        "origin row {origin_index} has an observed value after a missing cell at development column {development_index}"
    )]
    NonContiguousObservations {
        origin_index: usize,
        development_index: usize,
    },

    #[error("origin row {origin_index} is outside triangle row count {row_count}")]
    OriginIndexOutOfBounds {
        origin_index: usize,
        row_count: usize,
    },

    #[error("origin row {origin_index} has no observed values")]
    NoObservedValue { origin_index: usize },

    #[error(
        "development column {development_index} has a non-positive base for factor calculation"
    )]
    NonPositiveDevelopmentBase { development_index: usize },

    #[error("tail factor must be positive")]
    InvalidTailFactor,
}
