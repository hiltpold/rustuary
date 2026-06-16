use rustuary_core::{ChainLadder, DevelopmentAge, OriginPeriod, Triangle, TriangleBasis};

const ABSOLUTE_TOLERANCE: f64 = 1e-9;
const BASIC_CHAIN_LADDER_GOLDEN: &str = include_str!("../../../data/golden/chain_ladder_basic.csv");

#[derive(Debug, Clone, PartialEq)]
struct ExpectedOriginResult {
    origin_index: usize,
    origin_period: OriginPeriod,
    latest_development_index: usize,
    latest_development_age: DevelopmentAge,
    latest_observed: f64,
    remaining_factor_product: f64,
    tail_factor: f64,
    cdf_to_ultimate: f64,
    ultimate: f64,
    reserve: f64,
}

fn assert_close(actual: f64, expected: f64) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= ABSOLUTE_TOLERANCE,
        "actual={actual}, expected={expected}, diff={diff}"
    );
}

fn parse_basic_chain_ladder_golden() -> Vec<ExpectedOriginResult> {
    let mut lines = BASIC_CHAIN_LADDER_GOLDEN.lines();
    assert_eq!(
        lines.next(),
        Some(
            "origin_index,origin_period,latest_development_index,latest_development_age,latest_observed,remaining_factor_product,tail_factor,cdf_to_ultimate,ultimate,reserve"
        )
    );

    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let fields = line.split(',').collect::<Vec<_>>();
            assert_eq!(fields.len(), 10, "invalid golden row: {line}");

            ExpectedOriginResult {
                origin_index: fields[0].parse().expect("origin_index should be a usize"),
                origin_period: OriginPeriod(
                    fields[1].parse().expect("origin_period should be an i32"),
                ),
                latest_development_index: fields[2]
                    .parse()
                    .expect("latest_development_index should be a usize"),
                latest_development_age: DevelopmentAge(
                    fields[3]
                        .parse()
                        .expect("latest_development_age should be a u32"),
                ),
                latest_observed: fields[4]
                    .parse()
                    .expect("latest_observed should be finite f64"),
                remaining_factor_product: fields[5]
                    .parse()
                    .expect("remaining_factor_product should be finite f64"),
                tail_factor: fields[6].parse().expect("tail_factor should be finite f64"),
                cdf_to_ultimate: fields[7]
                    .parse()
                    .expect("cdf_to_ultimate should be finite f64"),
                ultimate: fields[8].parse().expect("ultimate should be finite f64"),
                reserve: fields[9].parse().expect("reserve should be finite f64"),
            }
        })
        .collect()
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

    let expected_origins = parse_basic_chain_ladder_golden();

    assert_close(result.age_to_age_factors[0], 390.0 / 220.0);
    assert_close(result.age_to_age_factors[1], 240.0 / 180.0);
    assert_eq!(result.origins.len(), expected_origins.len());

    for (actual, expected) in result.origins.iter().zip(expected_origins) {
        assert_eq!(actual.origin_index, expected.origin_index);
        assert_eq!(actual.origin_period, expected.origin_period);
        assert_eq!(
            actual.latest_development_index,
            expected.latest_development_index
        );
        assert_eq!(
            actual.latest_development_age,
            expected.latest_development_age
        );
        assert_close(actual.latest_observed, expected.latest_observed);
        assert_close(
            actual.remaining_factor_product,
            expected.remaining_factor_product,
        );
        assert_close(actual.tail_factor, expected.tail_factor);
        assert_close(actual.cdf_to_ultimate, expected.cdf_to_ultimate);
        assert_close(actual.ultimate, expected.ultimate);
        assert_close(actual.reserve, expected.reserve);
    }
}
