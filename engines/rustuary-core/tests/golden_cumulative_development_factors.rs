use rustuary_core::{cumulative_development_factor_diagnostics, DevelopmentAge, FixedTailFactor};

const ABSOLUTE_TOLERANCE: f64 = 1e-9;

fn assert_close(actual: f64, expected: f64) {
    let difference = (actual - expected).abs();
    assert!(
        difference <= ABSOLUTE_TOLERANCE,
        "actual={actual}, expected={expected}, difference={difference}"
    );
}

#[test]
fn golden_cumulative_development_factors_include_tail() {
    // Expected CDF at each development age is the product of remaining selected
    // age-to-age factors multiplied by the fixed tail factor.
    let tail_factor = FixedTailFactor::new(1.1).expect("golden fixed tail factor should be valid");

    let diagnostics = cumulative_development_factor_diagnostics(
        &[DevelopmentAge(12), DevelopmentAge(24), DevelopmentAge(36)],
        &[2.0, 1.5],
        &tail_factor,
    )
    .expect("golden CDF diagnostics should calculate");

    assert_eq!(diagnostics.len(), 3);
    assert_eq!(diagnostics[0].development_age, DevelopmentAge(12));
    assert_eq!(
        diagnostics[0].next_development_age,
        Some(DevelopmentAge(24))
    );
    assert_eq!(diagnostics[0].age_to_age_factor, Some(2.0));
    assert_close(diagnostics[0].remaining_factor_product, 3.0);
    assert_close(diagnostics[0].tail_factor, 1.1);
    assert_close(diagnostics[0].cdf, 3.3);

    assert_eq!(diagnostics[1].development_age, DevelopmentAge(24));
    assert_eq!(
        diagnostics[1].next_development_age,
        Some(DevelopmentAge(36))
    );
    assert_eq!(diagnostics[1].age_to_age_factor, Some(1.5));
    assert_close(diagnostics[1].remaining_factor_product, 1.5);
    assert_close(diagnostics[1].cdf, 1.65);

    assert_eq!(diagnostics[2].development_age, DevelopmentAge(36));
    assert_eq!(diagnostics[2].next_development_age, None);
    assert_eq!(diagnostics[2].age_to_age_factor, None);
    assert_close(diagnostics[2].remaining_factor_product, 1.0);
    assert_close(diagnostics[2].cdf, 1.1);
}
