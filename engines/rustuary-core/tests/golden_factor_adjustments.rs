use rustuary_core::{
    select_development_factors, DevelopmentAge, DevelopmentFactorMethod, DevelopmentFactorOverride,
    DevelopmentFactorSelectionAssumptions, LinkRatioExclusion, OriginPeriod, Triangle,
    TriangleBasis,
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
fn golden_exclusion_and_override_preserve_calculated_and_selected_factors() {
    // Excluding origin 2020 leaves 300 / 300 = 1.0. The explicit override
    // then changes the final selected factor to 1.25.
    let triangle = Triangle::new(
        vec![OriginPeriod(2020), OriginPeriod(2021), OriginPeriod(2022)],
        vec![DevelopmentAge(12), DevelopmentAge(24)],
        vec![
            vec![Some(100.0), Some(200.0)],
            vec![Some(300.0), Some(300.0)],
            vec![Some(150.0), None],
        ],
        TriangleBasis::Cumulative,
    )
    .expect("golden cumulative triangle should be valid");
    let assumptions = DevelopmentFactorSelectionAssumptions {
        exclusions: vec![LinkRatioExclusion {
            origin_period: OriginPeriod(2020),
            from_development_age: DevelopmentAge(12),
            rationale: "One-time claim settlement distorted development".to_owned(),
        }],
        overrides: vec![DevelopmentFactorOverride {
            from_development_age: DevelopmentAge(12),
            factor: 1.25,
            rationale: "Selected actuarial judgment".to_owned(),
        }],
    };

    let factors = select_development_factors(
        &triangle,
        DevelopmentFactorMethod::VolumeWeighted,
        &assumptions,
    )
    .expect("golden adjusted factor should calculate");

    assert_eq!(factors.len(), 1);
    assert_eq!(factors[0].observation_count, 1);
    assert_eq!(factors[0].exclusions.len(), 1);
    assert_close(factors[0].numerator, 300.0);
    assert_close(factors[0].denominator, 300.0);
    assert_close(factors[0].calculated_factor, 1.0);
    assert_close(factors[0].factor, 1.25);
    assert_close(factors[0].exclusions[0].link_ratio.ratio, 2.0);
    assert_eq!(
        factors[0]
            .applied_override
            .as_ref()
            .expect("golden override should be retained")
            .rationale,
        "Selected actuarial judgment"
    );
}
