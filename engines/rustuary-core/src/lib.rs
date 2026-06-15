//! Pure Rust actuarial reserving core.
//!
//! This crate should remain independent from Python, web services, databases,
//! dataframe libraries, and UI frameworks.

pub mod error;
pub mod methods;
pub mod triangle;
pub mod types;

pub use error::{ActuarialError, Result};
pub use methods::chain_ladder::{ChainLadder, ChainLadderResult, OriginChainLadderResult};
pub use methods::link_ratio::{link_ratios, LinkRatio};
pub use triangle::{LatestDiagonalEntry, Triangle, TriangleBasis};
pub use types::{DevelopmentAge, OriginPeriod};
