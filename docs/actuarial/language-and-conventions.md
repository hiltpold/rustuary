# Language and Conventions

This document defines the canonical actuarial language used across Rustuary
contracts, adapters, metadata, UI labels, and audit evidence.

## Portfolio and Segments

`portfolio_id` is the main reserving class, also called the primary actuarial
reserving unit. It is the first grouping dimension used to build a homogeneous
triangle. Examples include a reserving class, line of business, product group,
or any comparable actuarial unit selected by the workflow.

`segments` are optional ordered drill-down dimensions below `portfolio_id`.
Examples include `country`, `channel`, `coverage`, currency grouping, or any
workflow-approved business subdivision. A triangle may have zero segments.

Segment order is meaningful. It is defined by `TriangleDefinition` and controls
display grouping and deterministic keys. For example, the ordered segment list
`country`, `channel`, `coverage` is different from `coverage`, `country`,
`channel` for display-tree purposes even when the source data columns are the
same.

The canonical triangle grouping key is:

```text
portfolio_id + ordered segment values + measure
```

`segment_path` is not canonical input and must not be treated as independent
truth. Display or folder paths are derived from structured values:

```text
portfolio_id / segment_1_value / segment_2_value / ...
```

The UI may present a folder tree similar to legacy actuarial tools, but the
persisted and reproducible contract remains structured data:

- `portfolio_id`
- ordered `segments`
- `measure`
- valuation and assumption metadata
- source mapping and `TriangleDefinition`

## Triangle Definition

`TriangleDefinition` describes how raw claim or event records are grouped into
one or more canonical triangles. Every persisted triangle must be traceable to
the `TriangleDefinition` that produced it, including the ordered segment list
and the source columns or constants used for canonical fields.

The Rust engine consumes validated canonical triangles. For raw triangle
building, it consumes typed canonical claim/event records after adapters resolve
source columns and constants; it does not read dataframes or external source
column names directly.
