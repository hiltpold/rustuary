use crate::error::{ActuarialError, Result};
use crate::methods::development_factor::{
    select_volume_weighted_factors, SelectedDevelopmentFactor,
};
use crate::triangle::Triangle;

#[derive(Debug, Clone, PartialEq)]
pub struct ChainLadder {
    tail_factor: f64,
}

impl Default for ChainLadder {
    fn default() -> Self {
        Self { tail_factor: 1.0 }
    }
}

impl ChainLadder {
    pub fn new(tail_factor: f64) -> Result<Self> {
        if tail_factor <= 0.0 {
            return Err(ActuarialError::InvalidTailFactor);
        }
        Ok(Self { tail_factor })
    }

    pub fn fit_predict(&self, triangle: &Triangle) -> Result<ChainLadderResult> {
        let selected_factors = select_volume_weighted_factors(triangle)?;
        let age_to_age_factors = selected_factors
            .iter()
            .map(|selection| selection.factor)
            .collect::<Vec<_>>();
        let cdfs = cumulative_development_factors(&age_to_age_factors, self.tail_factor);
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
            tail_factor: self.tail_factor,
            origins,
        })
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq)]
pub struct ChainLadderResult {
    pub age_to_age_factors: Vec<f64>,
    pub selected_factors: Vec<SelectedDevelopmentFactor>,
    pub cdfs: Vec<f64>,
    pub tail_factor: f64,
    pub origins: Vec<OriginChainLadderResult>,
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
/// including the tail after the last observed development period.
#[must_use]
pub fn cumulative_development_factors(age_to_age_factors: &[f64], tail_factor: f64) -> Vec<f64> {
    let mut cdfs = vec![tail_factor; age_to_age_factors.len() + 1];
    let mut running = tail_factor;

    for idx in (0..age_to_age_factors.len()).rev() {
        running *= age_to_age_factors[idx];
        cdfs[idx] = running;
    }

    cdfs
}

#[cfg(test)]
mod tests {
    use super::{cumulative_development_factors, volume_weighted_factors, ChainLadder};
    use crate::triangle::Triangle;
    use crate::ActuarialError;

    fn assert_close(actual: f64, expected: f64) {
        let diff = (actual - expected).abs();
        assert!(
            diff < 1e-9,
            "actual={actual}, expected={expected}, diff={diff}"
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
        let cdfs = cumulative_development_factors(&[2.0, 1.5], 1.1);
        assert_close(cdfs[0], 3.3);
        assert_close(cdfs[1], 1.65);
        assert_close(cdfs[2], 1.1);
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
        assert_close(result.selected_factors[0].numerator, 390.0);
        assert_close(result.selected_factors[0].denominator, 220.0);
        assert_close(result.origins[0].ultimate, 240.0);
        assert_close(result.origins[1].ultimate, 210.0 * (240.0 / 180.0));
    }
}
