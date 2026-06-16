# Golden Fixtures

Golden fixtures in this directory are tiny synthetic examples. They are safe to
share and must not contain client data.

## Tolerance Policy

Rust actuarial golden tests use an absolute tolerance of `1e-9` for floating
point comparisons and no relative tolerance unless a test states otherwise.
Expected values should be written with enough decimal precision to make the
hand calculation reproducible. Changing an expected value requires a short note
in the relevant test, changelog, or model-governance entry explaining why the
expected behavior changed.

## `chain_ladder_basic.csv`

Expected values are hand-calculated from this synthetic cumulative triangle:

| Origin | 12 | 24 | 36 |
|---|---:|---:|---:|
| 2020 | 100 | 180 | 240 |
| 2021 | 120 | 210 | |
| 2022 | 150 | | |

The chain ladder uses volume-weighted selected factors and a fixed tail factor
of `1.0`:

- 12 -> 24: `(180 + 210) / (100 + 120) = 390 / 220`
- 24 -> 36: `240 / 180`
- CDF to ultimate: product of remaining selected factors multiplied by the
  fixed tail factor
- Ultimate: `latest_observed * cdf_to_ultimate`
- Reserve: `ultimate - latest_observed`
