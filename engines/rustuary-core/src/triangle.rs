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

/// Latest observed cell for one origin period.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatestDiagonalEntry {
    /// Zero-based row position in the triangle matrix.
    pub origin_index: usize,
    /// Business origin-period label for the row.
    pub origin_period: OriginPeriod,
    /// Zero-based column position of the latest observed cell.
    pub development_index: usize,
    /// Business development-age label for the latest observed cell.
    pub development_age: DevelopmentAge,
    /// Latest observed amount in the triangle's current basis.
    pub value: f64,
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

    /// Return a cumulative copy of this triangle.
    ///
    /// For an incremental row `X`, cumulative values are calculated as
    /// `C[j] = sum(X[k], k = 0..j)`. Axes and trailing unobserved cells are
    /// preserved. A cumulative triangle is returned unchanged apart from the
    /// cloned allocation.
    pub fn to_cumulative(&self) -> Result<Self> {
        if self.basis == TriangleBasis::Cumulative {
            return Ok(self.clone());
        }

        self.convert_rows(TriangleBasis::Cumulative)
    }

    /// Return an incremental copy of this triangle.
    ///
    /// The first observed value is unchanged. Later values are calculated as
    /// `X[j] = C[j] - C[j - 1]`. Negative incremental values are retained
    /// because cumulative claims may decrease after recoveries or corrections.
    /// Axes and trailing unobserved cells are preserved.
    pub fn to_incremental(&self) -> Result<Self> {
        if self.basis == TriangleBasis::Incremental {
            return Ok(self.clone());
        }

        self.convert_rows(TriangleBasis::Incremental)
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

    fn convert_rows(&self, basis: TriangleBasis) -> Result<Self> {
        let mut values = Vec::with_capacity(self.values.len());

        for origin_index in 0..self.row_count() {
            let mut previous = 0.0;
            for development_index in 0..self.col_count() {
                match self.get(origin_index, development_index) {
                    Some(current) => {
                        let converted = match basis {
                            TriangleBasis::Cumulative => previous + current,
                            TriangleBasis::Incremental => current - previous,
                        };
                        if !converted.is_finite() {
                            return Err(ActuarialError::NonFiniteConvertedValue {
                                origin_index,
                                development_index,
                            });
                        }
                        values.push(Some(converted));
                        previous = if basis == TriangleBasis::Cumulative {
                            converted
                        } else {
                            current
                        };
                    }
                    None => values.push(None),
                }
            }
        }

        Ok(Self {
            origin_periods: self.origin_periods.clone(),
            development_ages: self.development_ages.clone(),
            values,
            basis,
        })
    }

    /// Return the latest observed cell for one origin row.
    ///
    /// The value remains in the triangle's current basis. Call
    /// [`Triangle::to_cumulative`] before extraction when a cumulative latest
    /// diagonal is required.
    pub fn latest_observed(&self, origin_index: usize) -> Result<LatestDiagonalEntry> {
        let Some(origin_period) = self.origin_periods.get(origin_index).copied() else {
            return Err(ActuarialError::OriginIndexOutOfBounds {
                origin_index,
                row_count: self.row_count(),
            });
        };

        for (development_index, development_age) in
            self.development_ages.iter().copied().enumerate().rev()
        {
            if let Some(value) = self.get(origin_index, development_index) {
                return Ok(LatestDiagonalEntry {
                    origin_index,
                    origin_period,
                    development_index,
                    development_age,
                    value,
                });
            }
        }
        Err(ActuarialError::NoObservedValue { origin_index })
    }

    /// Return the latest observed cell for every origin period.
    ///
    /// Entries retain the triangle's origin and development labels and are
    /// ordered by the triangle's origin-period axis.
    pub fn latest_diagonal(&self) -> Result<Vec<LatestDiagonalEntry>> {
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

    fn assert_close(actual: f64, expected: f64) {
        let difference = (actual - expected).abs();
        assert!(
            difference <= 1e-9,
            "actual={actual}, expected={expected}, difference={difference}"
        );
    }

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
    fn converts_incremental_rows_to_cumulative() {
        let incremental = Triangle::new(
            vec![OriginPeriod(2020), OriginPeriod(2021)],
            vec![DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
            vec![
                vec![Some(100.0), Some(50.0), Some(-10.0)],
                vec![Some(110.0), Some(40.0), None],
            ],
            TriangleBasis::Incremental,
        )
        .expect("incremental test triangle should be valid");

        let cumulative = incremental
            .to_cumulative()
            .expect("finite incremental values should convert");

        assert_eq!(cumulative.basis(), TriangleBasis::Cumulative);
        assert_eq!(cumulative.origin_periods(), incremental.origin_periods());
        assert_eq!(
            cumulative.development_ages(),
            incremental.development_ages()
        );
        assert_eq!(cumulative.get(0, 0), Some(100.0));
        assert_eq!(cumulative.get(0, 1), Some(150.0));
        assert_eq!(cumulative.get(0, 2), Some(140.0));
        assert_eq!(cumulative.get(1, 0), Some(110.0));
        assert_eq!(cumulative.get(1, 1), Some(150.0));
        assert_eq!(cumulative.get(1, 2), None);
    }

    #[test]
    fn converts_cumulative_rows_to_incremental() {
        let cumulative = canonical_triangle();

        let incremental = cumulative
            .to_incremental()
            .expect("finite cumulative values should convert");

        assert_eq!(incremental.basis(), TriangleBasis::Incremental);
        assert_eq!(incremental.get(0, 0), Some(100.0));
        assert_eq!(incremental.get(0, 1), Some(50.0));
        assert_eq!(incremental.get(0, 2), Some(30.0));
        assert_eq!(incremental.get(1, 0), Some(110.0));
        assert_eq!(incremental.get(1, 1), Some(50.0));
        assert_eq!(incremental.get(1, 2), None);
    }

    #[test]
    fn conversion_to_existing_basis_returns_equal_triangle() {
        let cumulative = canonical_triangle();
        let incremental = cumulative
            .to_incremental()
            .expect("finite cumulative values should convert");

        assert_eq!(
            cumulative
                .to_cumulative()
                .expect("identity conversion should succeed"),
            cumulative
        );
        assert_eq!(
            incremental
                .to_incremental()
                .expect("identity conversion should succeed"),
            incremental
        );
    }

    #[test]
    fn rejects_non_finite_conversion_results() {
        let incremental = Triangle::new(
            vec![OriginPeriod(2020)],
            vec![DevelopmentAge(12), DevelopmentAge(24)],
            vec![vec![Some(f64::MAX), Some(f64::MAX)]],
            TriangleBasis::Incremental,
        )
        .expect("finite source amounts should be valid");

        assert_eq!(
            incremental
                .to_cumulative()
                .expect_err("overflow must not create an invalid triangle"),
            ActuarialError::NonFiniteConvertedValue {
                origin_index: 0,
                development_index: 1
            }
        );

        let cumulative = Triangle::new(
            vec![OriginPeriod(2020)],
            vec![DevelopmentAge(12), DevelopmentAge(24)],
            vec![vec![Some(f64::MAX), Some(-f64::MAX)]],
            TriangleBasis::Cumulative,
        )
        .expect("finite source amounts should be valid");

        assert_eq!(
            cumulative
                .to_incremental()
                .expect_err("overflow must not create an invalid triangle"),
            ActuarialError::NonFiniteConvertedValue {
                origin_index: 0,
                development_index: 1
            }
        );
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

        let first = triangle
            .latest_observed(0)
            .expect("first row has observations");
        assert_eq!(first.origin_index, 0);
        assert_eq!(first.origin_period, OriginPeriod(2020));
        assert_eq!(first.development_index, 2);
        assert_eq!(first.development_age, DevelopmentAge(36));
        assert_close(first.value, 180.0);

        let second = triangle
            .latest_observed(1)
            .expect("second row has observations");
        assert_eq!(second.origin_index, 1);
        assert_eq!(second.origin_period, OriginPeriod(2021));
        assert_eq!(second.development_index, 1);
        assert_eq!(second.development_age, DevelopmentAge(24));
        assert_close(second.value, 160.0);
    }

    #[test]
    fn extracts_typed_latest_diagonal_in_origin_order() {
        let triangle = canonical_triangle();

        let diagonal = triangle
            .latest_diagonal()
            .expect("validated rows have latest observations");

        assert_eq!(diagonal.len(), 2);
        assert_eq!(diagonal[0].origin_period, OriginPeriod(2020));
        assert_eq!(diagonal[0].development_age, DevelopmentAge(36));
        assert_close(diagonal[0].value, 180.0);
        assert_eq!(diagonal[1].origin_period, OriginPeriod(2021));
        assert_eq!(diagonal[1].development_age, DevelopmentAge(24));
        assert_close(diagonal[1].value, 160.0);
    }

    #[test]
    fn latest_diagonal_preserves_incremental_basis_values() {
        let incremental = Triangle::new(
            vec![OriginPeriod(2020)],
            vec![DevelopmentAge(12), DevelopmentAge(24)],
            vec![vec![Some(100.0), Some(50.0)]],
            TriangleBasis::Incremental,
        )
        .expect("incremental test triangle should be valid");

        let latest = incremental
            .latest_diagonal()
            .expect("validated row has a latest observation");

        assert_eq!(latest[0].development_age, DevelopmentAge(24));
        assert_close(latest[0].value, 50.0);
    }
}
