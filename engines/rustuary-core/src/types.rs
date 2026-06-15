/// Ordered origin period used as a triangle row label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OriginPeriod(pub i32);

/// Ordered development age used as a triangle column label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DevelopmentAge(pub u32);
