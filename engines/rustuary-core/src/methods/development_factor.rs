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
    /// Sum of included later cumulative values.
    pub numerator: f64,
    /// Sum of included earlier cumulative values.
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
        let mut denominator = 0.0;
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

#[cfg(test)]
mod tests {
    use super::{select_volume_weighted_factors, DevelopmentFactorMethod};
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
}
