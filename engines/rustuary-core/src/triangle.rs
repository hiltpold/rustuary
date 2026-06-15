use crate::error::{ActuarialError, Result};
use crate::types::{DevelopmentAge, OriginPeriod};

/// Whether triangle amounts are cumulative or incremental.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangleBasis {
    /// Each observed cell is the cumulative amount through its development age.
    Cumulative,
    /// Each observed cell is the amount emerging during its development interval.
    Incremental,
}

/// Canonical dense triangle for one homogeneous claims data slice.
///
/// Origin periods label rows and development ages label columns. Both axes are
/// unique and strictly increasing. Cells are stored row-major; `None` denotes
/// an unobserved trailing development cell. Observed cells must be finite and
/// left-contiguous within each origin row.
#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    origin_periods: Vec<OriginPeriod>,
    development_ages: Vec<DevelopmentAge>,
    values: Vec<Option<f64>>,
    basis: TriangleBasis,
}

impl Triangle {
    /// Construct a validated canonical triangle with explicit domain axes.
    pub fn new(
        origin_periods: Vec<OriginPeriod>,
        development_ages: Vec<DevelopmentAge>,
        rows: Vec<Vec<Option<f64>>>,
        basis: TriangleBasis,
    ) -> Result<Self> {
        validate_axes(&origin_periods, &development_ages, &rows)?;
        validate_rows(&rows, development_ages.len())?;

        Ok(Self {
            origin_periods,
            development_ages,
            values: rows.into_iter().flatten().collect(),
            basis,
        })
    }

    /// Construct a triangle using synthetic zero-based axes.
    ///
    /// This convenience constructor preserves the original matrix-only API.
    /// Adapters should prefer [`Triangle::new`] so business origin periods and
    /// development ages remain explicit.
    pub fn from_rows(rows: Vec<Vec<Option<f64>>>, cumulative: bool) -> Result<Self> {
        if rows.is_empty() {
            return Err(ActuarialError::EmptyTriangle);
        }

        let origin_periods = (0_i32..)
            .take(rows.len())
            .map(OriginPeriod)
            .collect::<Vec<_>>();
        let development_ages = (0_u32..)
            .take(rows[0].len())
            .map(DevelopmentAge)
            .collect::<Vec<_>>();
        let basis = if cumulative {
            TriangleBasis::Cumulative
        } else {
            TriangleBasis::Incremental
        };

        Self::new(origin_periods, development_ages, rows, basis)
    }

    /// Return the ordered origin-period axis.
    #[must_use]
    pub fn origin_periods(&self) -> &[OriginPeriod] {
        &self.origin_periods
    }

    /// Return the ordered development-age axis.
    #[must_use]
    pub fn development_ages(&self) -> &[DevelopmentAge] {
        &self.development_ages
    }

    /// Return whether amounts are cumulative or incremental.
    #[must_use]
    pub const fn basis(&self) -> TriangleBasis {
        self.basis
    }

    /// Return the number of origin periods.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.origin_periods.len()
    }

    /// Return the number of development ages.
    #[must_use]
    pub fn col_count(&self) -> usize {
        self.development_ages.len()
    }

    /// Return whether amounts are cumulative.
    #[must_use]
    pub const fn is_cumulative(&self) -> bool {
        matches!(self.basis, TriangleBasis::Cumulative)
    }

    /// Return a cell value, or `None` for an unobserved or out-of-bounds cell.
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        if row >= self.row_count() || col >= self.col_count() {
            return None;
        }

        row.checked_mul(self.col_count())
            .and_then(|offset| offset.checked_add(col))
            .and_then(|index| self.values.get(index))
            .copied()
            .flatten()
    }

    /// Return the latest observed development index and value for one row.
    pub fn latest_observed(&self, row: usize) -> Result<(usize, f64)> {
        if row >= self.row_count() {
            return Err(ActuarialError::OriginIndexOutOfBounds {
                origin_index: row,
                row_count: self.row_count(),
            });
        }

        for col in (0..self.col_count()).rev() {
            if let Some(value) = self.get(row, col) {
                return Ok((col, value));
            }
        }
        Err(ActuarialError::NoObservedValue { origin_index: row })
    }

    /// Return the latest observed development index and value for every row.
    pub fn latest_diagonal(&self) -> Result<Vec<(usize, f64)>> {
        (0..self.row_count())
            .map(|row| self.latest_observed(row))
            .collect()
    }
}

fn validate_axes(
    origin_periods: &[OriginPeriod],
    development_ages: &[DevelopmentAge],
    rows: &[Vec<Option<f64>>],
) -> Result<()> {
    if origin_periods.is_empty() {
        return Err(ActuarialError::EmptyTriangle);
    }
    if development_ages.is_empty() {
        return Err(ActuarialError::EmptyDevelopmentAxis);
    }
    if origin_periods.len() != rows.len() {
        return Err(ActuarialError::OriginAxisLengthMismatch {
            origin_count: origin_periods.len(),
            row_count: rows.len(),
        });
    }

    for periods in origin_periods.windows(2) {
        let previous = periods[0];
        let current = periods[1];
        if current <= previous {
            return Err(ActuarialError::UnorderedOriginPeriods { previous, current });
        }
    }
    for ages in development_ages.windows(2) {
        let previous = ages[0];
        let current = ages[1];
        if current <= previous {
            return Err(ActuarialError::UnorderedDevelopmentAges { previous, current });
        }
    }

    Ok(())
}

fn validate_rows(rows: &[Vec<Option<f64>>], development_count: usize) -> Result<()> {
    for (origin_index, row) in rows.iter().enumerate() {
        if row.len() != development_count {
            return Err(ActuarialError::RaggedTriangle {
                origin_index,
                expected: development_count,
                actual: row.len(),
            });
        }

        let mut missing_cell_seen = false;
        let mut observed_cell_seen = false;
        for (development_index, cell) in row.iter().enumerate() {
            match cell {
                Some(value) => {
                    if !value.is_finite() {
                        return Err(ActuarialError::NonFiniteTriangleValue {
                            origin_index,
                            development_index,
                        });
                    }
                    if missing_cell_seen {
                        return Err(ActuarialError::NonContiguousObservations {
                            origin_index,
                            development_index,
                        });
                    }
                    observed_cell_seen = true;
                }
                None => missing_cell_seen = true,
            }
        }

        if !observed_cell_seen {
            return Err(ActuarialError::NoObservedValue { origin_index });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Triangle, TriangleBasis};
    use crate::{ActuarialError, DevelopmentAge, OriginPeriod};

    fn canonical_triangle() -> Triangle {
        Triangle::new(
            vec![OriginPeriod(2020), OriginPeriod(2021)],
            vec![DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
            vec![
                vec![Some(100.0), Some(150.0), Some(180.0)],
                vec![Some(110.0), Some(160.0), None],
            ],
            TriangleBasis::Cumulative,
        )
        .expect("canonical test triangle should be valid")
    }

    #[test]
    fn preserves_canonical_axes_basis_and_values() {
        let triangle = canonical_triangle();

        assert_eq!(
            triangle.origin_periods(),
            &[OriginPeriod(2020), OriginPeriod(2021)]
        );
        assert_eq!(
            triangle.development_ages(),
            &[DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)]
        );
        assert_eq!(triangle.basis(), TriangleBasis::Cumulative);
        assert!(triangle.is_cumulative());
        assert_eq!(triangle.get(1, 1), Some(160.0));
        assert_eq!(triangle.get(1, 2), None);
    }

    #[test]
    fn from_rows_supplies_synthetic_axes() {
        let triangle = Triangle::from_rows(vec![vec![Some(100.0), None]], false)
            .expect("matrix-only triangle should be valid");

        assert_eq!(triangle.origin_periods(), &[OriginPeriod(0)]);
        assert_eq!(
            triangle.development_ages(),
            &[DevelopmentAge(0), DevelopmentAge(1)]
        );
        assert_eq!(triangle.basis(), TriangleBasis::Incremental);
    }

    #[test]
    fn rejects_mismatched_origin_axis() {
        let error = Triangle::new(
            vec![OriginPeriod(2020)],
            vec![DevelopmentAge(12)],
            vec![vec![Some(100.0)], vec![Some(110.0)]],
            TriangleBasis::Cumulative,
        )
        .expect_err("origin axis length must match row count");

        assert_eq!(
            error,
            ActuarialError::OriginAxisLengthMismatch {
                origin_count: 1,
                row_count: 2
            }
        );
    }

    #[test]
    fn rejects_empty_development_axis() {
        let error = Triangle::new(
            vec![OriginPeriod(2020)],
            vec![],
            vec![vec![]],
            TriangleBasis::Cumulative,
        )
        .expect_err("triangle must include a development age");

        assert_eq!(error, ActuarialError::EmptyDevelopmentAxis);
    }

    #[test]
    fn rejects_ragged_rows() {
        let error = Triangle::new(
            vec![OriginPeriod(2020), OriginPeriod(2021)],
            vec![DevelopmentAge(12), DevelopmentAge(24)],
            vec![vec![Some(100.0), Some(150.0)], vec![Some(110.0)]],
            TriangleBasis::Cumulative,
        )
        .expect_err("all rows must match the development axis");

        assert_eq!(
            error,
            ActuarialError::RaggedTriangle {
                origin_index: 1,
                expected: 2,
                actual: 1
            }
        );
    }

    #[test]
    fn rejects_unordered_axes() {
        let origin_error = Triangle::new(
            vec![OriginPeriod(2020), OriginPeriod(2020)],
            vec![DevelopmentAge(12)],
            vec![vec![Some(100.0)], vec![Some(110.0)]],
            TriangleBasis::Cumulative,
        )
        .expect_err("origin periods must be unique and ordered");
        assert!(matches!(
            origin_error,
            ActuarialError::UnorderedOriginPeriods { .. }
        ));

        let development_error = Triangle::new(
            vec![OriginPeriod(2020)],
            vec![DevelopmentAge(24), DevelopmentAge(12)],
            vec![vec![Some(100.0), Some(150.0)]],
            TriangleBasis::Cumulative,
        )
        .expect_err("development ages must be ordered");
        assert!(matches!(
            development_error,
            ActuarialError::UnorderedDevelopmentAges { .. }
        ));
    }

    #[test]
    fn rejects_non_finite_values_and_observation_gaps() {
        let non_finite_error = Triangle::new(
            vec![OriginPeriod(2020)],
            vec![DevelopmentAge(12)],
            vec![vec![Some(f64::NAN)]],
            TriangleBasis::Cumulative,
        )
        .expect_err("triangle amounts must be finite");
        assert_eq!(
            non_finite_error,
            ActuarialError::NonFiniteTriangleValue {
                origin_index: 0,
                development_index: 0
            }
        );

        let gap_error = Triangle::new(
            vec![OriginPeriod(2020)],
            vec![DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
            vec![vec![Some(100.0), None, Some(150.0)]],
            TriangleBasis::Cumulative,
        )
        .expect_err("observed cells must be left-contiguous");
        assert_eq!(
            gap_error,
            ActuarialError::NonContiguousObservations {
                origin_index: 0,
                development_index: 2
            }
        );
    }

    #[test]
    fn rejects_rows_without_observations() {
        let error = Triangle::new(
            vec![OriginPeriod(2020)],
            vec![DevelopmentAge(12)],
            vec![vec![None]],
            TriangleBasis::Cumulative,
        )
        .expect_err("every origin row must contain an observed amount");

        assert_eq!(error, ActuarialError::NoObservedValue { origin_index: 0 });
    }

    #[test]
    fn indexed_access_does_not_panic_outside_triangle_bounds() {
        let triangle = canonical_triangle();

        assert_eq!(triangle.get(10, 0), None);
        assert_eq!(triangle.get(0, 10), None);
        assert_eq!(triangle.get(0, 4), None);
        assert_eq!(
            triangle
                .latest_observed(10)
                .expect_err("invalid row must return an error"),
            ActuarialError::OriginIndexOutOfBounds {
                origin_index: 10,
                row_count: 2
            }
        );
    }

    #[test]
    fn returns_latest_observed_by_row() {
        let triangle = canonical_triangle();

        assert_eq!(
            triangle
                .latest_observed(0)
                .expect("first row has observations"),
            (2, 180.0)
        );
        assert_eq!(
            triangle
                .latest_observed(1)
                .expect("second row has observations"),
            (1, 160.0)
        );
    }
}
