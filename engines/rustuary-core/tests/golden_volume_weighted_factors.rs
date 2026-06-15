use rustuary_core::{
    select_volume_weighted_factors, DevelopmentAge, DevelopmentFactorMethod, OriginPeriod,
    Triangle, TriangleBasis,
};

const ABSOLUTE_TOLERANCE: f64 = 1e-9;

fn assert_close(actual: f64, expected: f64) {
    let difference = (actual - expected).abs();
    assert!(
        difference <= ABSOLUTE_TOLERANCE,
        "actual={actual}, expected={expected}, difference={difference}"
    );
}

#[test]
fn golden_volume_weighted_factors_match_hand_calculated_aggregates() {
    // Expected selections are sum(later cumulative) / sum(earlier cumulative).
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

    let factors = select_volume_weighted_factors(&triangle)
        .expect("golden volume-weighted factors should calculate");

    assert_eq!(factors.len(), 2);
    assert_eq!(factors[0].method, DevelopmentFactorMethod::VolumeWeighted);
    assert_eq!(factors[0].observation_count, 2);
    assert_close(factors[0].numerator, 180.0 + 210.0);
    assert_close(factors[0].denominator, 100.0 + 120.0);
    assert_close(factors[0].factor, (180.0 + 210.0) / (100.0 + 120.0));

    assert_eq!(factors[1].method, DevelopmentFactorMethod::VolumeWeighted);
    assert_eq!(factors[1].observation_count, 1);
    assert_close(factors[1].numerator, 240.0);
    assert_close(factors[1].denominator, 180.0);
    assert_close(factors[1].factor, 240.0 / 180.0);
}
