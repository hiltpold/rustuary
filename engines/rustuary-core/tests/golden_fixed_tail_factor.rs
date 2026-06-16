use rustuary_core::{
    ChainLadder, DevelopmentAge, FixedTailFactor, OriginPeriod, Triangle, TriangleBasis,
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
fn golden_fixed_tail_factor_flows_into_cdfs_and_reserves() {
    // Expected tail impact is direct multiplication after the last selected
    // age-to-age factor. The fully mature origin therefore reserves
    // latest_observed * (tail - 1.0).
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
    let tail_factor = FixedTailFactor::with_rationale(1.1, "Selected fixed tail")
        .expect("golden fixed tail should be valid");

    let result = ChainLadder::with_fixed_tail(tail_factor)
        .fit_predict(&triangle)
        .expect("golden chain ladder should calculate");

    assert_close(result.tail_factor.factor(), 1.1);
    assert_eq!(result.tail_factor.rationale(), Some("Selected fixed tail"));
    assert_close(result.cdfs[2], 1.1);
    assert_close(result.cdfs[1], (240.0 / 180.0) * 1.1);
    assert_close(result.origins[0].ultimate, 240.0 * 1.1);
    assert_close(result.origins[0].reserve, 24.0);
    assert_close(result.origins[0].remaining_factor_product, 1.0);
    assert_close(result.origins[0].tail_factor, 1.1);
}
