use crate::error::{ActuarialError, Result};
use crate::methods::development_factor::{
    select_volume_weighted_factors, SelectedDevelopmentFactor,
};
use crate::triangle::Triangle;
use crate::types::DevelopmentAge;

/// Fixed multiplicative tail factor applied after the last selected age-to-age factor.
#[derive(Debug, Clone, PartialEq)]
pub struct FixedTailFactor {
    factor: f64,
    rationale: Option<String>,
}

impl FixedTailFactor {
    /// Create a fixed tail factor without a rationale.
    pub fn new(factor: f64) -> Result<Self> {
        validate_tail_factor(factor)?;
        Ok(Self {
            factor,
            rationale: None,
        })
    }

    /// Create a fixed tail factor with an explicit rationale.
    pub fn with_rationale(factor: f64, rationale: impl Into<String>) -> Result<Self> {
        validate_tail_factor(factor)?;

        let rationale = rationale.into();
        let rationale = rationale.trim();
        if rationale.is_empty() {
            return Err(ActuarialError::EmptyTailFactorRationale);
        }

        Ok(Self {
            factor,
            rationale: Some(rationale.to_owned()),
        })
    }

    /// Multiplicative tail factor value.
    #[must_use]
    pub const fn factor(&self) -> f64 {
        self.factor
    }

    /// Optional actuarial rationale supplied with the fixed tail factor.
    #[must_use]
    pub fn rationale(&self) -> Option<&str> {
        self.rationale.as_deref()
    }
}

impl Default for FixedTailFactor {
    fn default() -> Self {
        Self {
            factor: 1.0,
            rationale: Some("No tail development beyond final observed age".to_owned()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChainLadder {
    tail_factor: FixedTailFactor,
}

impl ChainLadder {
    /// Create a chain-ladder model with a fixed numeric tail factor.
    pub fn new(tail_factor: f64) -> Result<Self> {
        Ok(Self {
            tail_factor: FixedTailFactor::new(tail_factor)?,
        })
    }

    /// Create a chain-ladder model with a typed fixed tail-factor assumption.
    #[must_use]
    pub const fn with_fixed_tail(tail_factor: FixedTailFactor) -> Self {
        Self { tail_factor }
    }

    /// Fixed tail-factor assumption used by this chain-ladder model.
    #[must_use]
    pub const fn tail_factor(&self) -> &FixedTailFactor {
        &self.tail_factor
    }

    pub fn fit_predict(&self, triangle: &Triangle) -> Result<ChainLadderResult> {
        let selected_factors = select_volume_weighted_factors(triangle)?;
        let age_to_age_factors = selected_factors
            .iter()
            .map(|selection| selection.factor)
            .collect::<Vec<_>>();
        let cdf_diagnostics = cumulative_development_factor_diagnostics(
            triangle.development_ages(),
            &age_to_age_factors,
            &self.tail_factor,
        )?;
        let cdfs = cdf_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.cdf)
            .collect::<Vec<_>>();
        let latest_diagonal = triangle.latest_diagonal()?;
        let mut origins = Vec::with_capacity(latest_diagonal.len());

        for latest in latest_diagonal {
            let latest_development_index = latest.development_index;
            let latest_observed = latest.value;
            let cdf_to_ultimate = cdfs[latest_development_index];
            let ultimate = latest_observed * cdf_to_ultimate;
            origins.push(OriginChainLadderResult {
                origin_index: latest.origin_index,
                latest_development_index,
                latest_observed,
                cdf_to_ultimate,
                ultimate,
                reserve: ultimate - latest_observed,
            });
        }

        Ok(ChainLadderResult {
            age_to_age_factors,
            selected_factors,
            cdfs,
            cdf_diagnostics,
            tail_factor: self.tail_factor.clone(),
            origins,
        })
    }
}

fn validate_tail_factor(tail_factor: f64) -> Result<()> {
    if !tail_factor.is_finite() || tail_factor <= 0.0 {
        return Err(ActuarialError::InvalidTailFactor);
    }
    Ok(())
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq)]
pub struct ChainLadderResult {
    pub age_to_age_factors: Vec<f64>,
    pub selected_factors: Vec<SelectedDevelopmentFactor>,
    pub cdfs: Vec<f64>,
    pub cdf_diagnostics: Vec<CumulativeDevelopmentFactor>,
    pub tail_factor: FixedTailFactor,
    pub origins: Vec<OriginChainLadderResult>,
}

/// Remaining cumulative development factor from one development age to ultimate.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CumulativeDevelopmentFactor {
    /// Zero-based development-age position.
    pub development_index: usize,
    /// Development-age label for this CDF.
    pub development_age: DevelopmentAge,
    /// Next development-age label, when an age-to-age factor remains.
    pub next_development_age: Option<DevelopmentAge>,
    /// Selected factor from this development age to the next age, if any.
    pub age_to_age_factor: Option<f64>,
    /// Product of remaining selected age-to-age factors, excluding tail.
    pub remaining_factor_product: f64,
    /// Fixed tail factor multiplied after all selected age-to-age factors.
    pub tail_factor: f64,
    /// CDF to ultimate, equal to `remaining_factor_product * tail_factor`.
    pub cdf: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OriginChainLadderResult {
    pub origin_index: usize,
    pub latest_development_index: usize,
    pub latest_observed: f64,
    pub cdf_to_ultimate: f64,
    pub ultimate: f64,
    pub reserve: f64,
}

/// Volume-weighted chain-ladder age-to-age factors.
///
/// For each adjacent development age j -> j+1, use rows where both cells are observed:
/// sum(C\[i,j+1\]) / sum(C\[i,j\]).
pub fn volume_weighted_factors(triangle: &Triangle) -> Result<Vec<f64>> {
    select_volume_weighted_factors(triangle).map(|selections| {
        selections
            .into_iter()
            .map(|selection| selection.factor)
            .collect()
    })
}

/// CDF at development index j is product of selected factors from j onward,
/// including the fixed tail factor after the last observed development period.
#[must_use]
pub fn cumulative_development_factors(
    age_to_age_factors: &[f64],
    tail_factor: &FixedTailFactor,
) -> Vec<f64> {
    let mut cdfs = vec![tail_factor.factor(); age_to_age_factors.len() + 1];
    let mut running = tail_factor.factor();

    for idx in (0..age_to_age_factors.len()).rev() {
        running *= age_to_age_factors[idx];
        cdfs[idx] = running;
    }

    cdfs
}

/// Calculate typed CDF diagnostics for every development age.
///
/// For development age `j`, the CDF is the product of selected age-to-age
/// factors from `j` onward multiplied by the fixed tail factor. The final
/// development age has no age-to-age factor and therefore has a CDF equal to
/// the tail factor.
pub fn cumulative_development_factor_diagnostics(
    development_ages: &[DevelopmentAge],
    age_to_age_factors: &[f64],
    tail_factor: &FixedTailFactor,
) -> Result<Vec<CumulativeDevelopmentFactor>> {
    if development_ages.is_empty() {
        return Err(ActuarialError::EmptyDevelopmentAxis);
    }
    if development_ages.len() != age_to_age_factors.len() + 1 {
        return Err(ActuarialError::DevelopmentFactorAxisLengthMismatch {
            development_age_count: development_ages.len(),
            factor_count: age_to_age_factors.len(),
        });
    }

    let mut diagnostics = Vec::with_capacity(development_ages.len());
    let mut remaining_factor_product = 1.0;

    for development_index in (0..development_ages.len()).rev() {
        if development_index < age_to_age_factors.len() {
            let factor = age_to_age_factors[development_index];
            if !factor.is_finite() {
                return Err(ActuarialError::NonFiniteCumulativeDevelopmentFactor {
                    development_index,
                });
            }
            remaining_factor_product *= factor;
            if !remaining_factor_product.is_finite() {
                return Err(ActuarialError::NonFiniteCumulativeDevelopmentFactor {
                    development_index,
                });
            }
        }

        let cdf = remaining_factor_product * tail_factor.factor();
        if !cdf.is_finite() {
            return Err(ActuarialError::NonFiniteCumulativeDevelopmentFactor { development_index });
        }

        diagnostics.push(CumulativeDevelopmentFactor {
            development_index,
            development_age: development_ages[development_index],
            next_development_age: development_ages.get(development_index + 1).copied(),
            age_to_age_factor: age_to_age_factors.get(development_index).copied(),
            remaining_factor_product,
            tail_factor: tail_factor.factor(),
            cdf,
        });
    }

    diagnostics.reverse();
    Ok(diagnostics)
}

#[cfg(test)]
mod tests {
    use super::{
        cumulative_development_factor_diagnostics, cumulative_development_factors,
        volume_weighted_factors, ChainLadder, FixedTailFactor,
    };
    use crate::triangle::Triangle;
    use crate::{ActuarialError, DevelopmentAge};

    fn assert_close(actual: f64, expected: f64) {
        let diff = (actual - expected).abs();
        assert!(
            diff < 1e-9,
            "actual={actual}, expected={expected}, diff={diff}"
        );
    }

    #[test]
    fn validates_fixed_tail_factor() {
        let tail = FixedTailFactor::with_rationale(1.05, " Selected tail ")
            .expect("positive finite tail factor should be valid");

        assert_close(tail.factor(), 1.05);
        assert_eq!(tail.rationale(), Some("Selected tail"));
        assert_eq!(
            FixedTailFactor::new(0.0).expect_err("zero tail factor is invalid"),
            ActuarialError::InvalidTailFactor
        );
        assert_eq!(
            FixedTailFactor::new(f64::INFINITY).expect_err("infinite tail factor is invalid"),
            ActuarialError::InvalidTailFactor
        );
        assert_eq!(
            FixedTailFactor::new(f64::NAN).expect_err("NaN tail factor is invalid"),
            ActuarialError::InvalidTailFactor
        );
        assert_eq!(
            FixedTailFactor::with_rationale(1.05, " ")
                .expect_err("blank tail rationale is invalid"),
            ActuarialError::EmptyTailFactorRationale
        );
    }

    #[test]
    fn computes_volume_weighted_factors() {
        let triangle = Triangle::from_rows(
            vec![
                vec![Some(100.0), Some(180.0), Some(240.0)],
                vec![Some(120.0), Some(210.0), None],
                vec![Some(150.0), None, None],
            ],
            true,
        )
        .unwrap();

        let factors = volume_weighted_factors(&triangle).unwrap();
        assert_close(factors[0], 390.0 / 220.0);
        assert_close(factors[1], 240.0 / 180.0);
    }

    #[test]
    fn rejects_incremental_volume_weighted_factors() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), Some(80.0)]], false).unwrap();

        assert_eq!(
            volume_weighted_factors(&triangle).unwrap_err(),
            ActuarialError::CumulativeTriangleRequired {
                operation: "volume-weighted factor calculation"
            }
        );
    }

    #[test]
    fn computes_cdfs_with_tail() {
        let tail_factor =
            FixedTailFactor::new(1.1).expect("positive finite tail factor should be valid");
        let cdfs = cumulative_development_factors(&[2.0, 1.5], &tail_factor);
        assert_close(cdfs[0], 3.3);
        assert_close(cdfs[1], 1.65);
        assert_close(cdfs[2], 1.1);
    }

    #[test]
    fn computes_typed_cdf_diagnostics_with_tail() {
        let tail_factor =
            FixedTailFactor::new(1.1).expect("positive finite tail factor should be valid");

        let diagnostics = cumulative_development_factor_diagnostics(
            &[DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
            &[2.0, 1.5],
            &tail_factor,
        )
        .expect("CDF diagnostics should calculate");

        assert_eq!(diagnostics.len(), 3);
        assert_eq!(diagnostics[0].development_index, 0);
        assert_eq!(diagnostics[0].development_age, DevelopmentAge(12));
        assert_eq!(
            diagnostics[0].next_development_age,
            Some(DevelopmentAge(24))
        );
        assert_eq!(diagnostics[0].age_to_age_factor, Some(2.0));
        assert_close(diagnostics[0].remaining_factor_product, 3.0);
        assert_close(diagnostics[0].tail_factor, 1.1);
        assert_close(diagnostics[0].cdf, 3.3);

        assert_eq!(diagnostics[2].development_index, 2);
        assert_eq!(diagnostics[2].development_age, DevelopmentAge(36));
        assert_eq!(diagnostics[2].next_development_age, None);
        assert_eq!(diagnostics[2].age_to_age_factor, None);
        assert_close(diagnostics[2].remaining_factor_product, 1.0);
        assert_close(diagnostics[2].cdf, 1.1);
    }

    #[test]
    fn rejects_cdf_axis_length_mismatch() {
        let tail_factor =
            FixedTailFactor::new(1.0).expect("positive finite tail factor should be valid");

        assert_eq!(
            cumulative_development_factor_diagnostics(
                &[DevelopmentAge(12), DevelopmentAge(24)],
                &[1.5, 1.2],
                &tail_factor,
            )
            .expect_err("CDF calculation needs one more development age than factor"),
            ActuarialError::DevelopmentFactorAxisLengthMismatch {
                development_age_count: 2,
                factor_count: 2,
            }
        );
    }

    #[test]
    fn rejects_non_finite_cdf_factor_or_product() {
        let tail_factor =
            FixedTailFactor::new(1.0).expect("positive finite tail factor should be valid");

        assert_eq!(
            cumulative_development_factor_diagnostics(
                &[DevelopmentAge(12), DevelopmentAge(24)],
                &[f64::INFINITY],
                &tail_factor,
            )
            .expect_err("non-finite selected factor must be rejected"),
            ActuarialError::NonFiniteCumulativeDevelopmentFactor {
                development_index: 0,
            }
        );

        assert_eq!(
            cumulative_development_factor_diagnostics(
                &[DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
                &[f64::MAX, f64::MAX],
                &tail_factor,
            )
            .expect_err("overflowed CDF product must be rejected"),
            ActuarialError::NonFiniteCumulativeDevelopmentFactor {
                development_index: 0,
            }
        );
    }

    #[test]
    fn projects_ultimates() {
        let triangle = Triangle::from_rows(
            vec![
                vec![Some(100.0), Some(180.0), Some(240.0)],
                vec![Some(120.0), Some(210.0), None],
                vec![Some(150.0), None, None],
            ],
            true,
        )
        .unwrap();

        let result = ChainLadder::new(1.0)
            .unwrap()
            .fit_predict(&triangle)
            .unwrap();
        assert_eq!(result.origins.len(), 3);
        assert_eq!(result.selected_factors.len(), 2);
        assert_eq!(result.cdf_diagnostics.len(), 3);
        assert_close(result.tail_factor.factor(), 1.0);
        assert_close(result.cdf_diagnostics[1].cdf, result.cdfs[1]);
        assert_close(result.selected_factors[0].numerator, 390.0);
        assert_close(result.selected_factors[0].denominator, 220.0);
        assert_close(result.origins[0].ultimate, 240.0);
        assert_close(result.origins[1].ultimate, 210.0 * (240.0 / 180.0));
    }

    #[test]
    fn projects_ultimates_with_fixed_tail_rationale() {
        let triangle = Triangle::from_rows(
            vec![
                vec![Some(100.0), Some(180.0), Some(240.0)],
                vec![Some(120.0), Some(210.0), None],
                vec![Some(150.0), None, None],
            ],
            true,
        )
        .unwrap();
        let tail_factor = FixedTailFactor::with_rationale(1.1, "Selected market tail")
            .expect("positive finite tail factor should be valid");

        let result = ChainLadder::with_fixed_tail(tail_factor.clone())
            .fit_predict(&triangle)
            .expect("chain ladder should project with a fixed tail");

        assert_eq!(result.tail_factor, tail_factor);
        assert_eq!(result.tail_factor.rationale(), Some("Selected market tail"));
        assert_close(result.cdfs[2], 1.1);
        assert_close(result.cdfs[1], (240.0 / 180.0) * 1.1);
        assert_close(result.origins[0].ultimate, 240.0 * 1.1);
        assert_close(result.origins[0].reserve, 24.0);
    }
}
