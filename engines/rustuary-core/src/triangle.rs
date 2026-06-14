use crate::error::{ActuarialError, Result};

/// Dense rectangular triangle representation.
///
/// Cells are row-major. `None` means not observed.
#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    rows: usize,
    cols: usize,
    values: Vec<Option<f64>>,
    cumulative: bool,
}

impl Triangle {
    pub fn from_rows(rows: Vec<Vec<Option<f64>>>, cumulative: bool) -> Result<Self> {
        if rows.is_empty() {
            return Err(ActuarialError::EmptyTriangle);
        }
        let cols = rows[0].len();
        if cols == 0 || rows.iter().any(|row| row.len() != cols) {
            return Err(ActuarialError::RaggedTriangle);
        }
        let row_count = rows.len();
        let values = rows.into_iter().flatten().collect();
        Ok(Self {
            rows: row_count,
            cols,
            values,
            cumulative,
        })
    }

    #[must_use]
    pub const fn row_count(&self) -> usize {
        self.rows
    }

    #[must_use]
    pub const fn col_count(&self) -> usize {
        self.cols
    }

    #[must_use]
    pub const fn is_cumulative(&self) -> bool {
        self.cumulative
    }

    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        self.values[row * self.cols + col]
    }

    pub fn latest_observed(&self, row: usize) -> Result<(usize, f64)> {
        for col in (0..self.cols).rev() {
            if let Some(value) = self.get(row, col) {
                return Ok((col, value));
            }
        }
        Err(ActuarialError::NoObservedValue { origin_index: row })
    }

    pub fn latest_diagonal(&self) -> Result<Vec<(usize, f64)>> {
        (0..self.rows).map(|row| self.latest_observed(row)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Triangle;

    #[test]
    fn returns_latest_observed_by_row() {
        let triangle = Triangle::from_rows(
            vec![
                vec![Some(100.0), Some(150.0), Some(180.0)],
                vec![Some(110.0), Some(160.0), None],
            ],
            true,
        )
        .unwrap();

        assert_eq!(triangle.latest_observed(0).unwrap(), (2, 180.0));
        assert_eq!(triangle.latest_observed(1).unwrap(), (1, 160.0));
    }
}
