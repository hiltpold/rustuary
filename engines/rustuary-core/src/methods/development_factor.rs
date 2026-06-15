use crate::error::{ActuarialError, Result};
use crate::methods::link_ratio::{link_ratios, LinkRatio};
use crate::triangle::{Triangle, TriangleBasis};
use crate::types::{DevelopmentAge, OriginPeriod};

/// Method used to select an age-to-age development factor.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevelopmentFactorMethod {
    /// Ratio of summed later cumulative values to summed earlier values.
    VolumeWeighted,
    /// Arithmetic mean of the observed individual link ratios.
    SimpleAverage,
}

/// One observed link ratio to omit from development-factor selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRatioExclusion {
    /// Business origin-period label identifying the link ratio.
    pub origin_period: OriginPeriod,
    /// Earlier development-age label identifying the link-ratio interval.
    pub from_development_age: DevelopmentAge,
    /// Required business or actuarial reason for the exclusion.
    pub rationale: String,
}

/// Explicit selected-factor replacement for one development interval.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq)]
pub struct DevelopmentFactorOverride {
    /// Earlier development-age label identifying the factor interval.
    pub from_development_age: DevelopmentAge,
    /// Positive finite factor to select instead of the calculated factor.
    pub factor: f64,
    /// Required business or actuarial reason for the override.
    pub rationale: String,
}

/// User-supplied assumptions applied during development-factor selection.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DevelopmentFactorSelectionAssumptions {
    /// Individual observed link ratios to omit before aggregation.
    pub exclusions: Vec<LinkRatioExclusion>,
    /// Calculated interval factors to replace after aggregation.
    pub overrides: Vec<DevelopmentFactorOverride>,
}

/// One exclusion that was matched to an observed link ratio and applied.
#[derive(Debug, Clone, PartialEq)]
pub struct AppliedLinkRatioExclusion {
    /// Complete source diagnostic for the omitted link ratio.
    pub link_ratio: LinkRatio,
    /// Business or actuarial reason supplied with the exclusion.
    pub rationale: String,
}

/// Selected age-to-age development factor and its supporting diagnostics.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedDevelopmentFactor {
    /// Zero-based column position of the factor denominator.
    pub from_development_index: usize,
    /// Development-age label of the factor denominator.
    pub from_development_age: DevelopmentAge,
    /// Zero-based column position of the factor numerator.
    pub to_development_index: usize,
    /// Development-age label of the factor numerator.
    pub to_development_age: DevelopmentAge,
    /// Selection method applied to this development interval.
    pub method: DevelopmentFactorMethod,
    /// Number of observed origin rows included in the selection.
    pub observation_count: usize,
    /// Observed link ratios omitted from this interval before aggregation.
    pub exclusions: Vec<AppliedLinkRatioExclusion>,
    /// Method-specific numerator used to calculate the selected factor.
    ///
    /// This is the sum of later cumulative values for volume weighting and the
    /// sum of individual link ratios for simple average.
    pub numerator: f64,
    /// Method-specific denominator used to calculate the selected factor.
    ///
    /// This is the sum of earlier cumulative values for volume weighting and
    /// the observation count for simple average.
    pub denominator: f64,
    /// Factor calculated from the included observations before any override.
    pub calculated_factor: f64,
    /// Override applied after calculation, including its required rationale.
    pub applied_override: Option<DevelopmentFactorOverride>,
    /// Final selected age-to-age development factor.
    pub factor: f64,
}

/// Select development factors with explicit exclusions and overrides.
///
/// Exclusions are matched by origin-period and earlier development-age labels
/// and are applied before the selected method aggregates observations.
/// Overrides are matched by earlier development-age label and are applied after
/// the calculated factor is validated. Every assumption must match the
/// triangle, be unique, and include a non-blank rationale. Overrides must also
/// be positive and finite. Excluding every observation from an interval remains
/// an error even when an override exists.
pub fn select_development_factors(
    triangle: &Triangle,
    method: DevelopmentFactorMethod,
    assumptions: &DevelopmentFactorSelectionAssumptions,
) -> Result<Vec<SelectedDevelopmentFactor>> {
    if triangle.basis() != TriangleBasis::Cumulative {
        return Err(ActuarialError::CumulativeTriangleRequired {
            operation: method.operation_name(),
        });
    }

    let ratios = link_ratios(triangle)?;
    validate_assumptions(triangle, &ratios, assumptions)?;

    let mut selections = Vec::with_capacity(triangle.development_ages().len().saturating_sub(1));

    for (development_index, ages) in triangle.development_ages().windows(2).enumerate() {
        let mut numerator = 0.0;
        let mut denominator: f64 = 0.0;
        let mut observation_count = 0;
        let mut applied_exclusions = Vec::new();

        for ratio in ratios
            .iter()
            .filter(|ratio| ratio.from_development_index == development_index)
        {
            if let Some(exclusion) = assumptions.exclusions.iter().find(|exclusion| {
                exclusion.origin_period == ratio.origin_period
                    && exclusion.from_development_age == ratio.from_development_age
            }) {
                applied_exclusions.push(AppliedLinkRatioExclusion {
                    link_ratio: *ratio,
                    rationale: exclusion.rationale.clone(),
                });
                continue;
            }

            match method {
                DevelopmentFactorMethod::VolumeWeighted => {
                    numerator += ratio.to_value;
                    denominator += ratio.from_value;
                }
                DevelopmentFactorMethod::SimpleAverage => {
                    numerator += ratio.ratio;
                    denominator += 1.0;
                }
            }
            observation_count += 1;
        }

        if observation_count == 0 {
            return Err(ActuarialError::NoDevelopmentFactorObservations { development_index });
        }
        if !numerator.is_finite() || !denominator.is_finite() {
            return Err(ActuarialError::NonFiniteDevelopmentFactor { development_index });
        }
        if method == DevelopmentFactorMethod::VolumeWeighted && denominator <= 0.0 {
            return Err(ActuarialError::NonPositiveDevelopmentBase { development_index });
        }

        let calculated_factor = numerator / denominator;
        if !calculated_factor.is_finite() {
            return Err(ActuarialError::NonFiniteDevelopmentFactor { development_index });
        }

        let applied_override = assumptions
            .overrides
            .iter()
            .find(|factor_override| factor_override.from_development_age == ages[0])
            .cloned();
        let factor = applied_override
            .as_ref()
            .map_or(calculated_factor, |factor_override| factor_override.factor);

        selections.push(SelectedDevelopmentFactor {
            from_development_index: development_index,
            from_development_age: ages[0],
            to_development_index: development_index + 1,
            to_development_age: ages[1],
            method,
            observation_count,
            exclusions: applied_exclusions,
            numerator,
            denominator,
            calculated_factor,
            applied_override,
            factor,
        });
    }

    Ok(selections)
}

/// Select volume-weighted development factors for every adjacent age interval.
///
/// For development interval `j` to `j + 1`, the selected factor is
/// `sum(C[i, j + 1]) / sum(C[i, j])` over origin rows where both cumulative
/// cells are observed. No exclusions or overrides are applied. Every interval
/// must have at least one observation and a positive aggregate denominator.
pub fn select_volume_weighted_factors(
    triangle: &Triangle,
) -> Result<Vec<SelectedDevelopmentFactor>> {
    select_development_factors(
        triangle,
        DevelopmentFactorMethod::VolumeWeighted,
        &DevelopmentFactorSelectionAssumptions::default(),
    )
}

/// Select simple-average development factors for every adjacent age interval.
///
/// For development interval `j` to `j + 1`, the selected factor is the
/// arithmetic mean of the observed individual link ratios:
/// `sum(C[i, j + 1] / C[i, j]) / observation_count`. No weighting, exclusions,
/// overrides, or fallback assumptions are applied.
pub fn select_simple_average_factors(
    triangle: &Triangle,
) -> Result<Vec<SelectedDevelopmentFactor>> {
    select_development_factors(
        triangle,
        DevelopmentFactorMethod::SimpleAverage,
        &DevelopmentFactorSelectionAssumptions::default(),
    )
}

impl DevelopmentFactorMethod {
    const fn operation_name(self) -> &'static str {
        match self {
            Self::VolumeWeighted => "volume-weighted factor calculation",
            Self::SimpleAverage => "simple-average factor calculation",
        }
    }
}

fn validate_assumptions(
    triangle: &Triangle,
    ratios: &[LinkRatio],
    assumptions: &DevelopmentFactorSelectionAssumptions,
) -> Result<()> {
    for (index, exclusion) in assumptions.exclusions.iter().enumerate() {
        if exclusion.rationale.trim().is_empty() {
            return Err(ActuarialError::EmptyLinkRatioExclusionRationale {
                origin_period: exclusion.origin_period,
                from_development_age: exclusion.from_development_age,
            });
        }
        if assumptions.exclusions[..index].iter().any(|candidate| {
            candidate.origin_period == exclusion.origin_period
                && candidate.from_development_age == exclusion.from_development_age
        }) {
            return Err(ActuarialError::DuplicateLinkRatioExclusion {
                origin_period: exclusion.origin_period,
                from_development_age: exclusion.from_development_age,
            });
        }
        if !ratios.iter().any(|ratio| {
            ratio.origin_period == exclusion.origin_period
                && ratio.from_development_age == exclusion.from_development_age
        }) {
            return Err(ActuarialError::UnknownLinkRatioExclusion {
                origin_period: exclusion.origin_period,
                from_development_age: exclusion.from_development_age,
            });
        }
    }

    let valid_override_ages =
        &triangle.development_ages()[..triangle.development_ages().len().saturating_sub(1)];
    for (index, factor_override) in assumptions.overrides.iter().enumerate() {
        if factor_override.rationale.trim().is_empty() {
            return Err(ActuarialError::EmptyDevelopmentFactorOverrideRationale {
                from_development_age: factor_override.from_development_age,
            });
        }
        if !factor_override.factor.is_finite() || factor_override.factor <= 0.0 {
            return Err(ActuarialError::InvalidDevelopmentFactorOverride {
                from_development_age: factor_override.from_development_age,
            });
        }
        if assumptions.overrides[..index]
            .iter()
            .any(|candidate| candidate.from_development_age == factor_override.from_development_age)
        {
            return Err(ActuarialError::DuplicateDevelopmentFactorOverride {
                from_development_age: factor_override.from_development_age,
            });
        }
        if !valid_override_ages.contains(&factor_override.from_development_age) {
            return Err(ActuarialError::UnknownDevelopmentFactorOverride {
                from_development_age: factor_override.from_development_age,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        select_development_factors, select_simple_average_factors, select_volume_weighted_factors,
        DevelopmentFactorMethod, DevelopmentFactorOverride, DevelopmentFactorSelectionAssumptions,
        LinkRatioExclusion,
    };
    use crate::{ActuarialError, DevelopmentAge, OriginPeriod, Triangle, TriangleBasis};

    fn assert_close(actual: f64, expected: f64) {
        let difference = (actual - expected).abs();
        assert!(
            difference <= 1e-9,
            "actual={actual}, expected={expected}, difference={difference}"
        );
    }

    #[test]
    fn selects_volume_weighted_factors_with_diagnostics() {
        let triangle = Triangle::new(
            vec![OriginPeriod(2020), OriginPeriod(2021), OriginPeriod(2022)],
            vec![DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
            vec![
                vec![Some(100.0), Some(180.0), Some(240.0)],
                vec![Some(120.0), Some(210.0), None],
                vec![Some(150.0), None, None],
            ],
            TriangleBasis::Cumulative,
        )
        .expect("factor-selection test triangle should be valid");

        let factors = select_volume_weighted_factors(&triangle)
            .expect("volume-weighted factors should calculate");

        assert_eq!(factors.len(), 2);
        assert_eq!(factors[0].from_development_age, DevelopmentAge(12));
        assert_eq!(factors[0].to_development_age, DevelopmentAge(24));
        assert_eq!(factors[0].method, DevelopmentFactorMethod::VolumeWeighted);
        assert_eq!(factors[0].observation_count, 2);
        assert_close(factors[0].numerator, 390.0);
        assert_close(factors[0].denominator, 220.0);
        assert_close(factors[0].calculated_factor, 390.0 / 220.0);
        assert!(factors[0].exclusions.is_empty());
        assert!(factors[0].applied_override.is_none());
        assert_close(factors[0].factor, 390.0 / 220.0);

        assert_eq!(factors[1].from_development_age, DevelopmentAge(24));
        assert_eq!(factors[1].to_development_age, DevelopmentAge(36));
        assert_eq!(factors[1].observation_count, 1);
        assert_close(factors[1].numerator, 240.0);
        assert_close(factors[1].denominator, 180.0);
        assert_close(factors[1].factor, 240.0 / 180.0);
    }

    #[test]
    fn rejects_interval_without_observations() {
        let triangle = Triangle::from_rows(
            vec![vec![Some(100.0), None, None], vec![Some(120.0), None, None]],
            true,
        )
        .expect("sparse factor-selection triangle should be valid");

        assert_eq!(
            select_volume_weighted_factors(&triangle)
                .expect_err("every selected interval needs observations"),
            ActuarialError::NoDevelopmentFactorObservations {
                development_index: 0
            }
        );
    }

    #[test]
    fn rejects_non_positive_aggregate_denominator() {
        let triangle = Triangle::from_rows(
            vec![
                vec![Some(-100.0), Some(-180.0)],
                vec![Some(50.0), Some(90.0)],
            ],
            true,
        )
        .expect("negative-base factor-selection triangle should be valid");

        assert_eq!(
            select_volume_weighted_factors(&triangle)
                .expect_err("aggregate denominator must be positive"),
            ActuarialError::NonPositiveDevelopmentBase {
                development_index: 0
            }
        );
    }

    #[test]
    fn rejects_non_finite_aggregate() {
        let triangle = Triangle::from_rows(
            vec![
                vec![Some(f64::MAX), Some(f64::MAX)],
                vec![Some(f64::MAX), Some(f64::MAX)],
            ],
            true,
        )
        .expect("finite source values should be structurally valid");

        assert_eq!(
            select_volume_weighted_factors(&triangle)
                .expect_err("overflowed aggregate must be rejected"),
            ActuarialError::NonFiniteDevelopmentFactor {
                development_index: 0
            }
        );
    }

    #[test]
    fn selects_simple_average_factors_with_diagnostics() {
        let triangle = Triangle::new(
            vec![OriginPeriod(2020), OriginPeriod(2021), OriginPeriod(2022)],
            vec![DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
            vec![
                vec![Some(100.0), Some(200.0), Some(300.0)],
                vec![Some(300.0), Some(300.0), None],
                vec![Some(150.0), None, None],
            ],
            TriangleBasis::Cumulative,
        )
        .expect("simple-average test triangle should be valid");

        let factors = select_simple_average_factors(&triangle)
            .expect("simple-average factors should calculate");

        assert_eq!(factors.len(), 2);
        assert_eq!(factors[0].method, DevelopmentFactorMethod::SimpleAverage);
        assert_eq!(factors[0].observation_count, 2);
        assert_close(factors[0].numerator, 2.0 + 1.0);
        assert_close(factors[0].denominator, 2.0);
        assert_close(factors[0].calculated_factor, 1.5);
        assert!(factors[0].exclusions.is_empty());
        assert!(factors[0].applied_override.is_none());
        assert_close(factors[0].factor, 1.5);

        assert_eq!(factors[1].method, DevelopmentFactorMethod::SimpleAverage);
        assert_eq!(factors[1].observation_count, 1);
        assert_close(factors[1].numerator, 1.5);
        assert_close(factors[1].denominator, 1.0);
        assert_close(factors[1].factor, 1.5);
    }

    #[test]
    fn simple_average_differs_from_volume_weighting() {
        let triangle = Triangle::from_rows(
            vec![
                vec![Some(100.0), Some(200.0)],
                vec![Some(300.0), Some(300.0)],
            ],
            true,
        )
        .expect("comparison triangle should be valid");

        let simple_average = select_simple_average_factors(&triangle)
            .expect("simple-average factor should calculate");
        let volume_weighted = select_volume_weighted_factors(&triangle)
            .expect("volume-weighted factor should calculate");

        assert_close(simple_average[0].factor, 1.5);
        assert_close(volume_weighted[0].factor, 1.25);
    }

    #[test]
    fn rejects_incremental_simple_average_factors() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), Some(80.0)]], false)
            .expect("incremental test triangle should be valid");

        assert_eq!(
            select_simple_average_factors(&triangle)
                .expect_err("simple average requires cumulative values"),
            ActuarialError::CumulativeTriangleRequired {
                operation: "simple-average factor calculation"
            }
        );
    }

    #[test]
    fn rejects_simple_average_interval_without_observations() {
        let triangle = Triangle::from_rows(
            vec![vec![Some(100.0), None, None], vec![Some(120.0), None, None]],
            true,
        )
        .expect("sparse simple-average triangle should be valid");

        assert_eq!(
            select_simple_average_factors(&triangle)
                .expect_err("every selected interval needs observations"),
            ActuarialError::NoDevelopmentFactorObservations {
                development_index: 0
            }
        );
    }

    #[test]
    fn rejects_non_finite_simple_average_sum() {
        let triangle = Triangle::from_rows(
            vec![
                vec![Some(1.0), Some(f64::MAX)],
                vec![Some(1.0), Some(f64::MAX)],
            ],
            true,
        )
        .expect("finite source values should be structurally valid");

        assert_eq!(
            select_simple_average_factors(&triangle)
                .expect_err("overflowed ratio sum must be rejected"),
            ActuarialError::NonFiniteDevelopmentFactor {
                development_index: 0
            }
        );
    }

    #[test]
    fn applies_exclusions_before_calculation_and_overrides_afterward() {
        let triangle = Triangle::new(
            vec![OriginPeriod(2020), OriginPeriod(2021), OriginPeriod(2022)],
            vec![DevelopmentAge(12), DevelopmentAge(24)],
            vec![
                vec![Some(100.0), Some(200.0)],
                vec![Some(300.0), Some(300.0)],
                vec![Some(150.0), None],
            ],
            TriangleBasis::Cumulative,
        )
        .expect("adjusted factor-selection triangle should be valid");
        let assumptions = DevelopmentFactorSelectionAssumptions {
            exclusions: vec![LinkRatioExclusion {
                origin_period: OriginPeriod(2020),
                from_development_age: DevelopmentAge(12),
                rationale: "One-time claim settlement distorted development".to_owned(),
            }],
            overrides: vec![DevelopmentFactorOverride {
                from_development_age: DevelopmentAge(12),
                factor: 1.25,
                rationale: "Selected actuarial judgment".to_owned(),
            }],
        };

        let factors = select_development_factors(
            &triangle,
            DevelopmentFactorMethod::VolumeWeighted,
            &assumptions,
        )
        .expect("valid exclusions and overrides should be applied");

        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].observation_count, 1);
        assert_close(factors[0].numerator, 300.0);
        assert_close(factors[0].denominator, 300.0);
        assert_close(factors[0].calculated_factor, 1.0);
        assert_close(factors[0].factor, 1.25);
        assert_eq!(factors[0].exclusions.len(), 1);
        assert_eq!(
            factors[0].exclusions[0].link_ratio.origin_period,
            OriginPeriod(2020)
        );
        assert_close(factors[0].exclusions[0].link_ratio.ratio, 2.0);
        assert_eq!(
            factors[0].exclusions[0].rationale,
            "One-time claim settlement distorted development"
        );
        assert_eq!(
            factors[0]
                .applied_override
                .as_ref()
                .expect("override diagnostic should be retained")
                .rationale,
            "Selected actuarial judgment"
        );
    }

    #[test]
    fn rejects_exclusion_without_rationale() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), Some(200.0)]], true)
            .expect("assumption-validation triangle should be valid");
        let assumptions = DevelopmentFactorSelectionAssumptions {
            exclusions: vec![LinkRatioExclusion {
                origin_period: OriginPeriod(0),
                from_development_age: DevelopmentAge(0),
                rationale: " ".to_owned(),
            }],
            overrides: Vec::new(),
        };

        assert_eq!(
            select_development_factors(
                &triangle,
                DevelopmentFactorMethod::VolumeWeighted,
                &assumptions,
            )
            .expect_err("exclusion rationale is required"),
            ActuarialError::EmptyLinkRatioExclusionRationale {
                origin_period: OriginPeriod(0),
                from_development_age: DevelopmentAge(0),
            }
        );
    }

    #[test]
    fn rejects_unknown_or_duplicate_exclusions() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), Some(200.0)]], true)
            .expect("assumption-validation triangle should be valid");
        let unknown = DevelopmentFactorSelectionAssumptions {
            exclusions: vec![LinkRatioExclusion {
                origin_period: OriginPeriod(99),
                from_development_age: DevelopmentAge(0),
                rationale: "Known data issue".to_owned(),
            }],
            overrides: Vec::new(),
        };

        assert_eq!(
            select_development_factors(
                &triangle,
                DevelopmentFactorMethod::VolumeWeighted,
                &unknown,
            )
            .expect_err("an exclusion must match an observed ratio"),
            ActuarialError::UnknownLinkRatioExclusion {
                origin_period: OriginPeriod(99),
                from_development_age: DevelopmentAge(0),
            }
        );

        let exclusion = LinkRatioExclusion {
            origin_period: OriginPeriod(0),
            from_development_age: DevelopmentAge(0),
            rationale: "Known data issue".to_owned(),
        };
        let duplicate = DevelopmentFactorSelectionAssumptions {
            exclusions: vec![exclusion.clone(), exclusion],
            overrides: Vec::new(),
        };

        assert_eq!(
            select_development_factors(
                &triangle,
                DevelopmentFactorMethod::VolumeWeighted,
                &duplicate,
            )
            .expect_err("an exclusion cannot be duplicated"),
            ActuarialError::DuplicateLinkRatioExclusion {
                origin_period: OriginPeriod(0),
                from_development_age: DevelopmentAge(0),
            }
        );
    }

    #[test]
    fn rejects_invalid_or_unmatched_overrides() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), Some(200.0)]], true)
            .expect("assumption-validation triangle should be valid");
        let invalid = DevelopmentFactorSelectionAssumptions {
            exclusions: Vec::new(),
            overrides: vec![DevelopmentFactorOverride {
                from_development_age: DevelopmentAge(0),
                factor: 0.0,
                rationale: "Selected actuarial judgment".to_owned(),
            }],
        };

        assert_eq!(
            select_development_factors(
                &triangle,
                DevelopmentFactorMethod::VolumeWeighted,
                &invalid,
            )
            .expect_err("an override factor must be positive"),
            ActuarialError::InvalidDevelopmentFactorOverride {
                from_development_age: DevelopmentAge(0),
            }
        );

        let unknown = DevelopmentFactorSelectionAssumptions {
            exclusions: Vec::new(),
            overrides: vec![DevelopmentFactorOverride {
                from_development_age: DevelopmentAge(99),
                factor: 1.25,
                rationale: "Selected actuarial judgment".to_owned(),
            }],
        };

        assert_eq!(
            select_development_factors(
                &triangle,
                DevelopmentFactorMethod::VolumeWeighted,
                &unknown,
            )
            .expect_err("an override must match an interval"),
            ActuarialError::UnknownDevelopmentFactorOverride {
                from_development_age: DevelopmentAge(99),
            }
        );
    }

    #[test]
    fn rejects_override_without_rationale_or_with_duplicate_interval() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), Some(200.0)]], true)
            .expect("assumption-validation triangle should be valid");
        let without_rationale = DevelopmentFactorSelectionAssumptions {
            exclusions: Vec::new(),
            overrides: vec![DevelopmentFactorOverride {
                from_development_age: DevelopmentAge(0),
                factor: 1.25,
                rationale: String::new(),
            }],
        };

        assert_eq!(
            select_development_factors(
                &triangle,
                DevelopmentFactorMethod::VolumeWeighted,
                &without_rationale,
            )
            .expect_err("override rationale is required"),
            ActuarialError::EmptyDevelopmentFactorOverrideRationale {
                from_development_age: DevelopmentAge(0),
            }
        );

        let factor_override = DevelopmentFactorOverride {
            from_development_age: DevelopmentAge(0),
            factor: 1.25,
            rationale: "Selected actuarial judgment".to_owned(),
        };
        let duplicate = DevelopmentFactorSelectionAssumptions {
            exclusions: Vec::new(),
            overrides: vec![factor_override.clone(), factor_override],
        };

        assert_eq!(
            select_development_factors(
                &triangle,
                DevelopmentFactorMethod::VolumeWeighted,
                &duplicate,
            )
            .expect_err("an interval override cannot be duplicated"),
            ActuarialError::DuplicateDevelopmentFactorOverride {
                from_development_age: DevelopmentAge(0),
            }
        );
    }

    #[test]
    fn rejects_excluding_every_observation_from_an_interval() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), Some(200.0)]], true)
            .expect("assumption-validation triangle should be valid");
        let assumptions = DevelopmentFactorSelectionAssumptions {
            exclusions: vec![LinkRatioExclusion {
                origin_period: OriginPeriod(0),
                from_development_age: DevelopmentAge(0),
                rationale: "Known data issue".to_owned(),
            }],
            overrides: vec![DevelopmentFactorOverride {
                from_development_age: DevelopmentAge(0),
                factor: 1.25,
                rationale: "Selected actuarial judgment".to_owned(),
            }],
        };

        assert_eq!(
            select_development_factors(
                &triangle,
                DevelopmentFactorMethod::VolumeWeighted,
                &assumptions,
            )
            .expect_err("an override does not replace all observations"),
            ActuarialError::NoDevelopmentFactorObservations {
                development_index: 0,
            }
        );
    }
}
