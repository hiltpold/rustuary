use rustuary_core::{DevelopmentAge, OriginPeriod, Triangle, TriangleBasis};

const ABSOLUTE_TOLERANCE: f64 = 1e-9;

fn assert_close(actual: f64, expected: f64) {
    let difference = (actual - expected).abs();
    assert!(
        difference <= ABSOLUTE_TOLERANCE,
        "actual={actual}, expected={expected}, difference={difference}"
    );
}

#[test]
fn golden_incremental_and_cumulative_triangle_conversion() {
    // Expected values are hand-calculated row-wise prefix sums and differences.
    let incremental = Triangle::new(
        vec![OriginPeriod(2020), OriginPeriod(2021), OriginPeriod(2022)],
        vec![DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
        vec![
            vec![Some(100.0), Some(80.0), Some(60.0)],
            vec![Some(120.0), Some(90.0), None],
            vec![Some(150.0), None, None],
        ],
        TriangleBasis::Incremental,
    )
    .expect("golden incremental triangle should be valid");

    let cumulative = incremental
        .to_cumulative()
        .expect("golden incremental triangle should convert");
    let expected_cumulative = [
        [Some(100.0), Some(180.0), Some(240.0)],
        [Some(120.0), Some(210.0), None],
        [Some(150.0), None, None],
    ];

    for (origin_index, row) in expected_cumulative.iter().enumerate() {
        for (development_index, expected) in row.iter().enumerate() {
            match expected {
                Some(expected) => assert_close(
                    cumulative
                        .get(origin_index, development_index)
                        .expect("expected cumulative cell should be observed"),
                    *expected,
                ),
                None => assert_eq!(cumulative.get(origin_index, development_index), None),
            }
        }
    }

    let round_trip = cumulative
        .to_incremental()
        .expect("golden cumulative triangle should convert");
    for origin_index in 0..incremental.row_count() {
        for development_index in 0..incremental.col_count() {
            match incremental.get(origin_index, development_index) {
                Some(expected) => assert_close(
                    round_trip
                        .get(origin_index, development_index)
                        .expect("expected incremental cell should be observed"),
                    expected,
                ),
                None => assert_eq!(round_trip.get(origin_index, development_index), None),
            }
        }
    }
}
