use rustuary_core::{DevelopmentAge, LatestDiagonalEntry, OriginPeriod, Triangle, TriangleBasis};

const ABSOLUTE_TOLERANCE: f64 = 1e-9;

fn assert_entry(actual: LatestDiagonalEntry, expected: LatestDiagonalEntry) {
    assert_eq!(actual.origin_index, expected.origin_index);
    assert_eq!(actual.origin_period, expected.origin_period);
    assert_eq!(actual.development_index, expected.development_index);
    assert_eq!(actual.development_age, expected.development_age);

    let difference = (actual.value - expected.value).abs();
    assert!(
        difference <= ABSOLUTE_TOLERANCE,
        "actual={}, expected={}, difference={difference}",
        actual.value,
        expected.value
    );
}

#[test]
fn golden_latest_diagonal_uses_last_observed_cell_per_origin() {
    // Expected entries are read directly from the last observed cell in each row.
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
    .expect("golden cumulative triangle should be valid");

    let diagonal = triangle
        .latest_diagonal()
        .expect("golden triangle should have one latest cell per origin");
    let expected = [
        LatestDiagonalEntry {
            origin_index: 0,
            origin_period: OriginPeriod(2020),
            development_index: 2,
            development_age: DevelopmentAge(36),
            value: 240.0,
        },
        LatestDiagonalEntry {
            origin_index: 1,
            origin_period: OriginPeriod(2021),
            development_index: 1,
            development_age: DevelopmentAge(24),
            value: 210.0,
        },
        LatestDiagonalEntry {
            origin_index: 2,
            origin_period: OriginPeriod(2022),
            development_index: 0,
            development_age: DevelopmentAge(12),
            value: 150.0,
        },
    ];

    assert_eq!(diagonal.len(), expected.len());
    for (actual, expected) in diagonal.into_iter().zip(expected) {
        assert_entry(actual, expected);
    }
}
