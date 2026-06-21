//! Pure Rust actuarial reserving core.
//!
//! This crate should remain independent from Python, web services, databases,
//! dataframe libraries, and UI frameworks.

pub mod error;
pub mod methods;
pub mod raw_claim;
pub mod triangle;
pub mod triangle_build;
pub mod types;

pub use error::{ActuarialError, Result};
pub use methods::chain_ladder::{
    cumulative_development_factor_diagnostics, cumulative_development_factors, ChainLadder,
    ChainLadderResult, CumulativeDevelopmentFactor, FixedTailFactor, OriginChainLadderResult,
};
pub use methods::development_factor::{
    select_development_factors, select_simple_average_factors, select_volume_weighted_factors,
    AppliedLinkRatioExclusion, DevelopmentFactorMethod, DevelopmentFactorOverride,
    DevelopmentFactorSelectionAssumptions, LinkRatioExclusion, SelectedDevelopmentFactor,
};
pub use methods::link_ratio::{link_ratios, LinkRatio};
pub use raw_claim::{ClaimEventRecord, ClaimEventRecordInput, RecordDate, SegmentValue};
pub use triangle::{LatestDiagonalEntry, Triangle, TriangleBasis};
pub use triangle_build::{
    build_triangle_set, BuiltTriangle, BuiltTriangleDiagnostics, TriangleBuildAggregation,
    TriangleBuildDiagnostics, TriangleBuildOutputKind, TriangleBuildRequest,
    TriangleBuildRequestInput, TriangleKey, TriangleSet,
};
pub use types::{DevelopmentAge, OriginPeriod};
