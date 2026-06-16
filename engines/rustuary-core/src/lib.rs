//! Pure Rust actuarial reserving core.
//!
//! This crate should remain independent from Python, web services, databases,
//! dataframe libraries, and UI frameworks.

pub mod error;
pub mod methods;
pub mod triangle;
pub mod types;

pub use error::{ActuarialError, Result};
pub use methods::chain_ladder::{
    ChainLadder, ChainLadderResult, FixedTailFactor, OriginChainLadderResult,
};
pub use methods::development_factor::{
    select_development_factors, select_simple_average_factors, select_volume_weighted_factors,
    AppliedLinkRatioExclusion, DevelopmentFactorMethod, DevelopmentFactorOverride,
    DevelopmentFactorSelectionAssumptions, LinkRatioExclusion, SelectedDevelopmentFactor,
};
pub use methods::link_ratio::{link_ratios, LinkRatio};
pub use triangle::{LatestDiagonalEntry, Triangle, TriangleBasis};
pub use types::{DevelopmentAge, OriginPeriod};
