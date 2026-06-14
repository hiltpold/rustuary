use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, ActuarialError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActuarialError {
    EmptyTriangle,
    RaggedTriangle,
    NoObservedValue { origin_index: usize },
    NonPositiveDevelopmentBase { development_index: usize },
    InvalidTailFactor,
}

impl Display for ActuarialError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTriangle => write!(f, "triangle must contain at least one origin period"),
            Self::RaggedTriangle => write!(f, "triangle rows must all have the same number of development periods"),
            Self::NoObservedValue { origin_index } => write!(f, "origin row {origin_index} has no observed values"),
            Self::NonPositiveDevelopmentBase { development_index } => write!(f, "development column {development_index} has a non-positive base for factor calculation"),
            Self::InvalidTailFactor => write!(f, "tail factor must be positive"),
        }
    }
}

impl std::error::Error for ActuarialError {}
