use crate::error::{ActuarialError, Result};
use crate::triangle::{Triangle, TriangleBasis};
use crate::types::{DevelopmentAge, OriginPeriod};
use std::num::FpCategory;

/// One observed age-to-age link ratio and its source values.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkRatio {
    /// Zero-based row position in the triangle matrix.
    pub origin_index: usize,
    /// Business origin-period label for the row.
    pub origin_period: OriginPeriod,
    /// Zero-based column position of the ratio denominator.
    pub from_development_index: usize,
    /// Development-age label of the ratio denominator.
    pub from_development_age: DevelopmentAge,
    /// Zero-based column position of the ratio numerator.
    pub to_development_index: usize,
    /// Development-age label of the ratio numerator.
    pub to_development_age: DevelopmentAge,
    /// Earlier cumulative observed value used as the denominator.
    pub from_value: f64,
    /// Later cumulative observed value used as the numerator.
    pub to_value: f64,
    /// Age-to-age ratio calculated as `to_value / from_value`.
    pub ratio: f64,
}

/// Calculate every observed cumulative age-to-age link ratio.
///
/// For origin `i` and adjacent development ages `j` and `j + 1`, the ratio is
/// `C[i, j + 1] / C[i, j]`. A pair is included only when both cells are
/// observed. Incremental triangles must be converted explicitly before calling
/// this function. A zero denominator is undefined and returns a typed error;
/// finite negative ratios are retained for auditability.
pub fn link_ratios(triangle: &Triangle) -> Result<Vec<LinkRatio>> {
    if triangle.basis() != TriangleBasis::Cumulative {
        return Err(ActuarialError::CumulativeTriangleRequired {
            operation: "link-ratio calculation",
        });
    }

    let mut ratios = Vec::new();
    for (origin_index, origin_period) in triangle.origin_periods().iter().copied().enumerate() {
        for (development_index, development_ages) in
            triangle.development_ages().windows(2).enumerate()
        {
            let (Some(from_value), Some(to_value)) = (
                triangle.get(origin_index, development_index),
                triangle.get(origin_index, development_index + 1),
            ) else {
                continue;
            };

            if from_value.classify() == FpCategory::Zero {
                return Err(ActuarialError::ZeroLinkRatioBase {
                    origin_index,
                    development_index,
                });
            }

            let ratio = to_value / from_value;
            if !ratio.is_finite() {
                return Err(ActuarialError::NonFiniteLinkRatio {
                    origin_index,
                    development_index,
                });
            }

            ratios.push(LinkRatio {
                origin_index,
                origin_period,
                from_development_index: development_index,
                from_development_age: development_ages[0],
                to_development_index: development_index + 1,
                to_development_age: development_ages[1],
                from_value,
                to_value,
                ratio,
            });
        }
    }

    Ok(ratios)
}

#[cfg(test)]
mod tests {
    use super::link_ratios;
    use crate::{ActuarialError, DevelopmentAge, OriginPeriod, Triangle, TriangleBasis};

    fn assert_close(actual: f64, expected: f64) {
        let difference = (actual - expected).abs();
        assert!(
            difference <= 1e-9,
            "actual={actual}, expected={expected}, difference={difference}"
        );
    }

    #[test]
    fn calculates_typed_link_ratio_diagnostics() {
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
        .expect("link-ratio test triangle should be valid");

        let ratios = link_ratios(&triangle).expect("cumulative ratios should calculate");

        assert_eq!(ratios.len(), 3);
        assert_eq!(ratios[0].origin_period, OriginPeriod(2020));
        assert_eq!(ratios[0].from_development_age, DevelopmentAge(12));
        assert_eq!(ratios[0].to_development_age, DevelopmentAge(24));
        assert_close(ratios[0].from_value, 100.0);
        assert_close(ratios[0].to_value, 180.0);
        assert_close(ratios[0].ratio, 1.8);

        assert_eq!(ratios[1].origin_period, OriginPeriod(2020));
        assert_eq!(ratios[1].from_development_age, DevelopmentAge(24));
        assert_eq!(ratios[1].to_development_age, DevelopmentAge(36));
        assert_close(ratios[1].ratio, 240.0 / 180.0);

        assert_eq!(ratios[2].origin_period, OriginPeriod(2021));
        assert_eq!(ratios[2].from_development_age, DevelopmentAge(12));
        assert_eq!(ratios[2].to_development_age, DevelopmentAge(24));
        assert_close(ratios[2].ratio, 210.0 / 120.0);
    }

    #[test]
    fn rejects_incremental_triangle() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), Some(80.0)]], false)
            .expect("incremental test triangle should be valid");

        assert_eq!(
            link_ratios(&triangle).expect_err("link ratios require cumulative values"),
            ActuarialError::CumulativeTriangleRequired {
                operation: "link-ratio calculation"
            }
        );
    }

    #[test]
    fn rejects_zero_link_ratio_base() {
        let triangle = Triangle::from_rows(vec![vec![Some(0.0), Some(10.0)]], true)
            .expect("zero-base test triangle should be structurally valid");

        assert_eq!(
            link_ratios(&triangle).expect_err("zero link-ratio base is undefined"),
            ActuarialError::ZeroLinkRatioBase {
                origin_index: 0,
                development_index: 0
            }
        );
    }

    #[test]
    fn rejects_non_finite_link_ratio() {
        let smallest_positive = f64::from_bits(1);
        let triangle =
            Triangle::from_rows(vec![vec![Some(smallest_positive), Some(f64::MAX)]], true)
                .expect("finite source values should be structurally valid");

        assert_eq!(
            link_ratios(&triangle).expect_err("non-finite ratio must be rejected"),
            ActuarialError::NonFiniteLinkRatio {
                origin_index: 0,
                development_index: 0
            }
        );
    }

    #[test]
    fn retains_finite_negative_link_ratios() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), Some(-20.0)]], true)
            .expect("negative-value test triangle should be valid");

        let ratios = link_ratios(&triangle).expect("finite negative ratio should be retained");

        assert_close(ratios[0].ratio, -0.2);
    }

    #[test]
    fn returns_no_ratios_when_no_adjacent_pair_exists() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0)], vec![Some(120.0)]], true)
            .expect("single-development triangle should be valid");

        assert!(link_ratios(&triangle)
            .expect("no adjacent pairs is a valid diagnostic result")
            .is_empty());
    }
}
