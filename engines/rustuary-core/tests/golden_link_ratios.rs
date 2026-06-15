use rustuary_core::{link_ratios, DevelopmentAge, OriginPeriod, Triangle, TriangleBasis};

const ABSOLUTE_TOLERANCE: f64 = 1e-9;

fn assert_close(actual: f64, expected: f64) {
    let difference = (actual - expected).abs();
    assert!(
        difference <= ABSOLUTE_TOLERANCE,
        "actual={actual}, expected={expected}, difference={difference}"
    );
}

#[test]
fn golden_link_ratios_match_hand_calculated_age_to_age_values() {
    // Expected ratios are the later cumulative cell divided by the earlier cell.
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

    let ratios = link_ratios(&triangle).expect("golden link ratios should calculate");

    assert_eq!(ratios.len(), 3);
    assert_eq!(ratios[0].origin_period, OriginPeriod(2020));
    assert_eq!(ratios[0].from_development_age, DevelopmentAge(12));
    assert_eq!(ratios[0].to_development_age, DevelopmentAge(24));
    assert_close(ratios[0].ratio, 180.0 / 100.0);

    assert_eq!(ratios[1].origin_period, OriginPeriod(2020));
    assert_eq!(ratios[1].from_development_age, DevelopmentAge(24));
    assert_eq!(ratios[1].to_development_age, DevelopmentAge(36));
    assert_close(ratios[1].ratio, 240.0 / 180.0);

    assert_eq!(ratios[2].origin_period, OriginPeriod(2021));
    assert_eq!(ratios[2].from_development_age, DevelopmentAge(12));
    assert_eq!(ratios[2].to_development_age, DevelopmentAge(24));
    assert_close(ratios[2].ratio, 210.0 / 120.0);
}
