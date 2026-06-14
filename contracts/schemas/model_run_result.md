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
| latest_observed | decimal/double | Latest cumulative value |
| cdf_to_ultimate | double | Remaining CDF including tail |
| tail_factor | double | Tail factor applied |
| apriori_ultimate | decimal/double | Prior ultimate, where applicable |
| ultimate | decimal/double | Estimated ultimate |
| reserve | decimal/double | Ultimate less observed |
| weight | double | Selection/blend weight |
| rationale | string | Actuarial rationale for selection |
| diagnostics_json | string | Small structured diagnostic payload; large diagnostics should be separate artifact |
