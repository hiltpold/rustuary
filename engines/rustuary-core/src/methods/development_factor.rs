use crate::error::{ActuarialError, Result};
use crate::methods::link_ratio::link_ratios;
use crate::triangle::{Triangle, TriangleBasis};
use crate::types::DevelopmentAge;

/// Method used to select an age-to-age development factor.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevelopmentFactorMethod {
    /// Ratio of summed later cumulative values to summed earlier values.
    VolumeWeighted,
    /// Arithmetic mean of the observed individual link ratios.
    SimpleAverage,
}

/// Selected age-to-age development factor and its supporting diagnostics.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Selected age-to-age development factor.
    pub factor: f64,
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
    if triangle.basis() != TriangleBasis::Cumulative {
        return Err(ActuarialError::CumulativeTriangleRequired {
            operation: "volume-weighted factor calculation",
        });
    }

    let ratios = link_ratios(triangle)?;
    let mut selections = Vec::with_capacity(triangle.development_ages().len().saturating_sub(1));

    for (development_index, ages) in triangle.development_ages().windows(2).enumerate() {
        let mut numerator = 0.0;
        let mut denominator: f64 = 0.0;
        let mut observation_count = 0;

        for ratio in ratios
            .iter()
            .filter(|ratio| ratio.from_development_index == development_index)
        {
            numerator += ratio.to_value;
            denominator += ratio.from_value;
            observation_count += 1;
        }

        if observation_count == 0 {
            return Err(ActuarialError::NoDevelopmentFactorObservations { development_index });
        }
        if !numerator.is_finite() || !denominator.is_finite() {
            return Err(ActuarialError::NonFiniteDevelopmentFactor { development_index });
        }
        if denominator <= 0.0 {
            return Err(ActuarialError::NonPositiveDevelopmentBase { development_index });
        }

        let factor = numerator / denominator;
        if !factor.is_finite() {
            return Err(ActuarialError::NonFiniteDevelopmentFactor { development_index });
        }

        selections.push(SelectedDevelopmentFactor {
            from_development_index: development_index,
            from_development_age: ages[0],
            to_development_index: development_index + 1,
            to_development_age: ages[1],
            method: DevelopmentFactorMethod::VolumeWeighted,
            observation_count,
            numerator,
            denominator,
            factor,
        });
    }

    Ok(selections)
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
    if triangle.basis() != TriangleBasis::Cumulative {
        return Err(ActuarialError::CumulativeTriangleRequired {
            operation: "simple-average factor calculation",
        });
    }

    let ratios = link_ratios(triangle)?;
    let mut selections = Vec::with_capacity(triangle.development_ages().len().saturating_sub(1));

    for (development_index, ages) in triangle.development_ages().windows(2).enumerate() {
        let mut ratio_sum = 0.0;
        let mut denominator: f64 = 0.0;
        let mut observation_count = 0;

        for ratio in ratios
            .iter()
            .filter(|ratio| ratio.from_development_index == development_index)
        {
            ratio_sum += ratio.ratio;
            denominator += 1.0;
            observation_count += 1;
        }

        if observation_count == 0 {
            return Err(ActuarialError::NoDevelopmentFactorObservations { development_index });
        }

        if !ratio_sum.is_finite() || !denominator.is_finite() {
            return Err(ActuarialError::NonFiniteDevelopmentFactor { development_index });
        }

        let factor = ratio_sum / denominator;
        if !factor.is_finite() {
            return Err(ActuarialError::NonFiniteDevelopmentFactor { development_index });
        }

        selections.push(SelectedDevelopmentFactor {
            from_development_index: development_index,
            from_development_age: ages[0],
            to_development_index: development_index + 1,
            to_development_age: ages[1],
            method: DevelopmentFactorMethod::SimpleAverage,
            observation_count,
            numerator: ratio_sum,
            denominator,
            factor,
        });
    }

    Ok(selections)
}

#[cfg(test)]
mod tests {
    use super::{
        select_simple_average_factors, select_volume_weighted_factors, DevelopmentFactorMethod,
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
}
