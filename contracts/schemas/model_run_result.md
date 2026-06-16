# Model run result logical schema

Target physical format: Parquet.

| Column | Type | Notes |
|---|---|---|
| model_run_id | string | Stable run ID |
| portfolio_id | string | Portfolio or segment |
| valuation_date | date | Valuation date |
| origin_period | string | Accident/underwriting/report year or period |
| method | string | Candidate or selected method label |
| result_kind | string | `candidate` or `selected` |
| latest_observed | decimal/double | Latest cumulative value used for projection |
| cdf_to_ultimate | double | Remaining CDF including tail, selected from the development-age CDF diagnostics |
| tail_factor | double | Positive finite fixed tail factor applied; rationale should be retained in diagnostics when supplied |
| apriori_ultimate | decimal/double | Prior ultimate, where applicable |
| ultimate | decimal/double | Estimated ultimate, `latest_observed * cdf_to_ultimate` |
| reserve | decimal/double | Ultimate less latest observed |
| weight | double | Selection/blend weight |
| rationale | string | Actuarial rationale for selection |
| diagnostics_json | string | Small structured diagnostic payload, including CDF diagnostics; large diagnostics should be separate artifact |
