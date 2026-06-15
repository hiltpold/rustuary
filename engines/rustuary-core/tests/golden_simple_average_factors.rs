use rustuary_core::{
    select_simple_average_factors, select_volume_weighted_factors, DevelopmentAge,
    DevelopmentFactorMethod, OriginPeriod, Triangle, TriangleBasis,
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
fn golden_simple_average_factors_mean_individual_link_ratios() {
    // The first interval has ratios 2.0 and 1.0, so its simple average is 1.5.
    // Its volume-weighted selection is 500 / 400 = 1.25, proving the methods differ.
    let triangle = Triangle::new(
        vec![OriginPeriod(2020), OriginPeriod(2021), OriginPeriod(2022)],
        vec![DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
        vec![
            vec![Some(100.0), Some(200.0), Some(300.0)],
            vec![Some(300.0), Some(300.0), None],
            vec![Some(150.0), None, None],
        ],
        TriangleBasis::Cumulative,
    )
    .expect("golden cumulative triangle should be valid");

    let simple_average = select_simple_average_factors(&triangle)
        .expect("golden simple-average factors should calculate");
    let volume_weighted = select_volume_weighted_factors(&triangle)
        .expect("comparison volume-weighted factors should calculate");

    assert_eq!(simple_average.len(), 2);
    assert_eq!(
        simple_average[0].method,
        DevelopmentFactorMethod::SimpleAverage
    );
    assert_eq!(simple_average[0].observation_count, 2);
    assert_close(simple_average[0].numerator, 3.0);
    assert_close(simple_average[0].denominator, 2.0);
    assert_close(simple_average[0].factor, 1.5);
    assert_close(volume_weighted[0].factor, 1.25);

    assert_eq!(
        simple_average[1].method,
        DevelopmentFactorMethod::SimpleAverage
    );
    assert_eq!(simple_average[1].observation_count, 1);
    assert_close(simple_average[1].factor, 1.5);
}
