from __future__ import annotations

from collections.abc import Iterable, Mapping
from dataclasses import dataclass
from decimal import Decimal
from typing import Any, cast

import pyarrow as pa

from ._dataframe import to_arrow_table
from .errors import ColumnMappingError
from .triangle_definition import SegmentDefinition, TriangleDefinition
from .triangle_set import TriangleSet


@dataclass(frozen=True, slots=True)
class TriangleBuilder:
    """Prepare raw claim/event records for triangle construction."""

    definition: TriangleDefinition

    def __post_init__(self) -> None:
        if not isinstance(self.definition, TriangleDefinition):
            raise TypeError("definition must be a TriangleDefinition")

    @classmethod
    def from_frame(cls, data: Any, *, definition: TriangleDefinition) -> TriangleSet:
        """Build a Rust-backed ``TriangleSet`` from a raw claim/event dataframe."""
        if not isinstance(definition, TriangleDefinition):
            raise TypeError("definition must be a TriangleDefinition")
        builder = cls(definition)
        payload = builder._build_payload(data)
        return TriangleSet(
            payload,
            audit_input={"triangle_definition": definition.to_dict()},
        )

    def required_source_columns(self) -> tuple[str, ...]:
        """Return source columns required by the builder definition."""
        return tuple(
            dict.fromkeys(
                column.source_column
                for column in _definition_source_columns(self.definition)
            )
        )

    def validate_frame(self, data: Any) -> pa.Table:
        """Convert input to Arrow and validate required source columns."""
        table = to_arrow_table(data)
        for source_column in _definition_source_columns(self.definition):
            _require_source_column(
                table,
                canonical_field=source_column.canonical_field,
                source_column=source_column.source_column,
            )
        return table

    def _build_payload(self, data: Any) -> dict[str, Any]:
        """Build a raw Rust triangle-set payload from an adapted dataframe."""
        table = self.validate_frame(data)
        request = _build_request(self.definition)
        records = _canonical_records(table, self.definition)
        rust = _load_rust_extension()
        return rust.build_triangle_set(request, records)


@dataclass(frozen=True, slots=True)
class _DefinitionSourceColumn:
    canonical_field: str
    source_column: str


def _definition_source_columns(
    definition: TriangleDefinition,
) -> Iterable[_DefinitionSourceColumn]:
    yield _DefinitionSourceColumn("origin_period", definition.origin_date)
    yield _DefinitionSourceColumn("development_age", definition.development_date)
    if definition.amount is not None:
        yield _DefinitionSourceColumn("amount", definition.amount)

    yield from _mapping_source_column("portfolio_id", definition.portfolio_id)
    yield from _mapping_source_column("measure", definition.measure)
    for segment in definition.segments:
        if isinstance(segment, SegmentDefinition):
            yield from _mapping_source_column(f"segments.{segment.name}", segment.source)
    if definition.valuation_date is not None:
        yield from _mapping_source_column("valuation_date", definition.valuation_date)
    if definition.currency is not None:
        yield from _mapping_source_column("currency", definition.currency)


def _mapping_source_column(
    canonical_field: str,
    value: object,
) -> Iterable[_DefinitionSourceColumn]:
    if isinstance(value, str):
        yield _DefinitionSourceColumn(canonical_field, value)
    elif isinstance(value, Mapping):
        return


def _require_source_column(
    table: pa.Table,
    *,
    canonical_field: str,
    source_column: str,
) -> None:
    match_count = table.column_names.count(source_column)
    if match_count == 0:
        raise ColumnMappingError(
            canonical_field=canonical_field,
            source_column=source_column,
            available_columns=table.column_names,
            reason=f"`{source_column}` is not present in the dataframe",
        )
    if match_count > 1:
        raise ColumnMappingError(
            canonical_field=canonical_field,
            source_column=source_column,
            available_columns=table.column_names,
            reason=f"`{source_column}` appears {match_count} times in the dataframe",
        )


def _load_rust_extension() -> Any:
    try:
        from . import _rust  # type: ignore[attr-defined, import-untyped]
    except ImportError as exc:
        raise ImportError(
            "TriangleBuilder requires the compiled rustuary._rust extension. "
            "Run `uv run maturin develop` from bindings/python."
        ) from exc
    return _rust


def _build_request(definition: TriangleDefinition) -> dict[str, Any]:
    return {
        "triangle_definition_id": definition.triangle_definition_id,
        "schema_version": definition.schema_version,
        "aggregation": definition.aggregation,
        "bucket_months": definition.bucket_months,
        "output_kind": definition.output_kind,
        "segment_names": [segment.name for segment in _segments(definition)],
    }


def _canonical_records(
    table: pa.Table,
    definition: TriangleDefinition,
) -> list[dict[str, Any]]:
    return [
        _canonical_record(row, definition, row_index)
        for row_index, row in enumerate(table.to_pylist())
    ]


def _canonical_record(
    row: Mapping[str, Any],
    definition: TriangleDefinition,
    row_index: int,
) -> dict[str, Any]:
    record = {
        "origin_date": _required_date(
            row[definition.origin_date],
            canonical_field="origin_date",
            source_column=definition.origin_date,
            row_index=row_index,
        ),
        "development_date": _required_date(
            row[definition.development_date],
            canonical_field="development_date",
            source_column=definition.development_date,
            row_index=row_index,
        ),
        "amount": None if definition.amount is None else row[definition.amount],
        "portfolio_id": _required_string(
            _resolve_mapping_value(row, definition.portfolio_id),
            canonical_field="portfolio_id",
            source_column=_source_column_name(definition.portfolio_id),
            row_index=row_index,
        ),
        "segments": [
            {
                "name": segment.name,
                "value": _required_string(
                    _resolve_mapping_value(row, segment.source),
                    canonical_field=f"segments.{segment.name}",
                    source_column=_source_column_name(segment.source),
                    row_index=row_index,
                ),
            }
            for segment in _segments(definition)
        ],
        "measure": _required_string(
            _resolve_mapping_value(row, definition.measure),
            canonical_field="measure",
            source_column=_source_column_name(definition.measure),
            row_index=row_index,
        ),
        "valuation_date": (
            None
            if definition.valuation_date is None
            else _resolve_mapping_value(row, definition.valuation_date)
        ),
        "currency": (
            None
            if definition.currency is None
            else _optional_string(
                _resolve_mapping_value(row, definition.currency),
                canonical_field="currency",
                source_column=_source_column_name(definition.currency),
                row_index=row_index,
            )
        ),
    }
    return record


def _resolve_mapping_value(row: Mapping[str, Any], value: object) -> Any:
    if isinstance(value, Mapping):
        return _constant_mapping_value(value)
    if isinstance(value, str):
        return row[value]
    return value


def _segments(definition: TriangleDefinition) -> tuple[SegmentDefinition, ...]:
    return cast(tuple[SegmentDefinition, ...], definition.segments)


def _constant_mapping_value(value: Mapping[str, object]) -> object:
    if set(value) != {"const"}:
        raise ValueError("constant mapping objects must contain only a `const` field")
    return value["const"]


def _source_column_name(value: object) -> str | None:
    if isinstance(value, str):
        return value
    return None


def _required_date(
    value: Any,
    *,
    canonical_field: str,
    source_column: str,
    row_index: int,
) -> Any:
    if value is None:
        raise ValueError(
            f"canonical field `{canonical_field}` mapped from source column "
            f"`{source_column}` is null at row {row_index}"
        )
    return value


def _required_string(
    value: Any,
    *,
    canonical_field: str,
    source_column: str | None,
    row_index: int,
) -> str:
    if value is None:
        location = _mapping_location(canonical_field, source_column)
        raise ValueError(f"{location} is null at row {row_index}")
    if isinstance(value, (list, tuple, dict)):
        location = _mapping_location(canonical_field, source_column)
        raise ValueError(f"{location} must resolve to a scalar value at row {row_index}")
    if isinstance(value, Decimal):
        return str(value)
    return str(value)


def _optional_string(
    value: Any,
    *,
    canonical_field: str,
    source_column: str | None,
    row_index: int,
) -> str | None:
    if value is None:
        return None
    return _required_string(
        value,
        canonical_field=canonical_field,
        source_column=source_column,
        row_index=row_index,
    )


def _mapping_location(canonical_field: str, source_column: str | None) -> str:
    if source_column is None:
        return f"canonical field `{canonical_field}`"
    return (
        f"canonical field `{canonical_field}` mapped from source column "
        f"`{source_column}`"
    )
