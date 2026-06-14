use rustuary_core::{ChainLadder, Triangle};

fn assert_close(actual: f64, expected: f64) {
    let diff = (actual - expected).abs();
    assert!(diff < 1e-9, "actual={actual}, expected={expected}, diff={diff}");
}

#[test]
fn golden_chain_ladder_basic_triangle() {
    let triangle = Triangle::from_rows(
        vec![
            vec![Some(100.0), Some(180.0), Some(240.0)],
            vec![Some(120.0), Some(210.0), None],
            vec![Some(150.0), None, None],
        ],
        true,
    )
    .unwrap();

    let result = ChainLadder::new(1.0).unwrap().fit_predict(&triangle).unwrap();

    assert_close(result.age_to_age_factors[0], 390.0 / 220.0);
    assert_close(result.age_to_age_factors[1], 240.0 / 180.0);
    assert_close(result.origins[0].reserve, 0.0);
}
