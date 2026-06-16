use rustuary_core::{ChainLadder, DevelopmentAge, OriginPeriod, Triangle, TriangleBasis};

fn assert_close(actual: f64, expected: f64) {
    let diff = (actual - expected).abs();
    assert!(
        diff < 1e-9,
        "actual={actual}, expected={expected}, diff={diff}"
    );
}

#[test]
fn golden_chain_ladder_basic_triangle() {
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
    .unwrap();

    let result = ChainLadder::new(1.0)
        .unwrap()
        .fit_predict(&triangle)
        .unwrap();

    assert_close(result.age_to_age_factors[0], 390.0 / 220.0);
    assert_close(result.age_to_age_factors[1], 240.0 / 180.0);
    assert_eq!(result.origins[1].origin_period, OriginPeriod(2021));
    assert_eq!(result.origins[1].latest_development_age, DevelopmentAge(24));
    assert_close(result.origins[1].latest_observed, 210.0);
    assert_close(result.origins[1].remaining_factor_product, 240.0 / 180.0);
    assert_close(result.origins[1].tail_factor, 1.0);
    assert_close(result.origins[1].cdf_to_ultimate, 240.0 / 180.0);
    assert_close(result.origins[1].ultimate, 210.0 * (240.0 / 180.0));
    assert_close(result.origins[1].reserve, 210.0 * (240.0 / 180.0) - 210.0);
    assert_close(result.origins[0].reserve, 0.0);
}
