from __future__ import annotations

from collections.abc import Iterable, Mapping
from dataclasses import dataclass
from typing import Any

import pyarrow as pa

from ._dataframe import to_arrow_table
from .errors import ColumnMappingError
from .triangle_definition import SegmentDefinition, TriangleDefinition


@dataclass(frozen=True, slots=True)
class TriangleBuilder:
    """Prepare raw claim/event records for triangle construction."""

    definition: TriangleDefinition

    def __post_init__(self) -> None:
        if not isinstance(self.definition, TriangleDefinition):
            raise TypeError("definition must be a TriangleDefinition")

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
