from __future__ import annotations

from collections.abc import Mapping

import pyarrow as pa

from .errors import ColumnMappingError
from .mapping import ClaimsMapping, MappingScalar, MappingValue


def _constant_array(value: MappingScalar, row_count: int) -> pa.Array:
    return pa.repeat(pa.scalar(value), row_count)


def _explicit_constant(value: Mapping[str, MappingScalar]) -> MappingScalar:
    if set(value) != {"const"}:
        raise ValueError("constant mapping objects must contain only a `const` field")
    return value["const"]


def _column_or_constant(
    table: pa.Table,
    *,
    canonical_field: str,
    value: MappingValue,
) -> pa.Array | pa.ChunkedArray:
    if isinstance(value, Mapping):
        return _constant_array(_explicit_constant(value), table.num_rows)
    if isinstance(value, str) and value in table.column_names:
        return _source_column(
            table,
            canonical_field=canonical_field,
            source_column=value,
        )
    return _constant_array(value, table.num_rows)


def _source_column(
    table: pa.Table,
    *,
    canonical_field: str,
    source_column: str,
) -> pa.ChunkedArray:
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
    return table.column(source_column)


def normalize_claims_table(table: pa.Table, mapping: ClaimsMapping) -> pa.Table:
    """Select and rename mapped claims data into canonical Arrow fields."""
    columns: list[pa.Array | pa.ChunkedArray] = []
    names: list[str] = []

    def append_source(canonical_name: str, source_name: str) -> None:
        columns.append(
            _source_column(
                table,
                canonical_field=canonical_name,
                source_column=source_name,
            )
        )
        names.append(canonical_name)

    def append_optional(canonical_name: str, value: MappingValue | None) -> None:
        if value is None:
            return
        columns.append(
            _column_or_constant(
                table,
                canonical_field=canonical_name,
                value=value,
            )
        )
        names.append(canonical_name)

    append_optional("portfolio_id", mapping.portfolio)
    append_optional("valuation_date", mapping.valuation_date)
    append_source("origin_period", mapping.origin)
    append_source("development_age", mapping.development)
    append_optional("measure", mapping.measure)
    append_source("amount", mapping.value)
    append_optional("currency", mapping.currency)

    if isinstance(mapping.cumulative, bool):
        columns.append(_constant_array(mapping.cumulative, table.num_rows))
    else:
        columns.append(
            _source_column(
                table,
                canonical_field="is_cumulative",
                source_column=mapping.cumulative,
            )
        )
    names.append("is_cumulative")

    return pa.Table.from_arrays(columns, names=names)
