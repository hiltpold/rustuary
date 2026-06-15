from __future__ import annotations

from collections.abc import Mapping, Sequence
from importlib import import_module
from typing import Any

import pyarrow as pa


def _inherits_from_package(data: object, package: str) -> bool:
    return any(cls.__module__.partition(".")[0] == package for cls in type(data).__mro__)


def _from_pandas(data: object) -> pa.Table | None:
    if not _inherits_from_package(data, "pandas"):
        return None

    pandas = import_module("pandas")
    if not isinstance(data, pandas.DataFrame):
        return None

    table = pa.Table.from_pandas(data, preserve_index=False)
    return table.replace_schema_metadata(None)


def _from_polars(data: object) -> pa.Table | None:
    if not _inherits_from_package(data, "polars"):
        return None

    polars = import_module("polars")
    if not isinstance(data, polars.DataFrame):
        return None

    table = data.to_arrow()
    if not isinstance(table, pa.Table):
        raise TypeError("polars.DataFrame.to_arrow() must return a pyarrow.Table")
    return table


def _from_records(data: object) -> pa.Table | None:
    if not isinstance(data, Sequence) or isinstance(data, (str, bytes, bytearray)):
        return None

    records = list(data)
    if not all(isinstance(record, Mapping) for record in records):
        return None
    return pa.Table.from_pylist(records)


def to_arrow_table(data: Any) -> pa.Table:
    """Convert a supported dataframe-like input into a PyArrow table."""
    if isinstance(data, pa.Table):
        return data
    if isinstance(data, pa.RecordBatch):
        return pa.Table.from_batches([data])
    if isinstance(data, pa.RecordBatchReader):
        return data.read_all()

    pandas_table = _from_pandas(data)
    if pandas_table is not None:
        return pandas_table

    polars_table = _from_polars(data)
    if polars_table is not None:
        return polars_table

    records_table = _from_records(data)
    if records_table is not None:
        return records_table

    raise TypeError(
        "data must be a pandas.DataFrame, polars.DataFrame, pyarrow.Table, "
        "pyarrow.RecordBatch, pyarrow.RecordBatchReader, or sequence of records"
    )
