from __future__ import annotations

import json
from collections.abc import Iterable, Mapping
from dataclasses import dataclass, field
from datetime import date, datetime
from decimal import Decimal
from typing import Any, Literal, TypeAlias, cast

from .mapping import MappingValue

Aggregation: TypeAlias = Literal["sum", "count"]
OutputKind: TypeAlias = Literal["incremental", "cumulative"]

_AGGREGATIONS = frozenset(("sum", "count"))
_OUTPUT_KINDS = frozenset(("incremental", "cumulative"))


@dataclass(frozen=True, slots=True)
class SegmentDefinition:
    """Ordered segment mapping below the main reserving class."""

    name: str
    source: MappingValue

    def __post_init__(self) -> None:
        _validate_non_empty_string(self.name, "segment name")
        _validate_mapping_value(self.source, "segment source")

    def to_dict(self) -> dict[str, Any]:
        """Return a JSON-safe segment definition."""
        return {
            "name": self.name,
            "source": _json_safe(self.source),
        }


@dataclass(frozen=True, slots=True)
class TriangleDefinition:
    """Define how raw claim/event records become canonical triangles."""

    triangle_definition_id: str
    origin_date: str
    development_date: str
    measure: MappingValue
    portfolio_id: MappingValue
    amount: str | None = None
    aggregation: Aggregation = "sum"
    bucket_months: int = 12
    output_kind: OutputKind = "cumulative"
    segments: Iterable[SegmentDefinition | Mapping[str, Any]] = field(default_factory=tuple)
    schema_version: str = "1"
    valuation_date: MappingValue | None = None
    currency: MappingValue | None = None

    def __post_init__(self) -> None:
        _validate_non_empty_string(self.triangle_definition_id, "triangle_definition_id")
        _validate_non_empty_string(self.schema_version, "schema_version")
        _validate_source_column(self.origin_date, "origin_date")
        _validate_source_column(self.development_date, "development_date")
        _validate_mapping_value(self.measure, "measure")
        _validate_mapping_value(self.portfolio_id, "portfolio_id")
        if self.valuation_date is not None:
            _validate_mapping_value(self.valuation_date, "valuation_date")
        if self.currency is not None:
            _validate_mapping_value(self.currency, "currency")

        if self.aggregation not in _AGGREGATIONS:
            allowed = ", ".join(sorted(_AGGREGATIONS))
            raise ValueError(f"aggregation must be one of: {allowed}")
        if self.aggregation == "sum":
            _validate_source_column(self.amount, "amount")
        elif self.amount is not None:
            raise ValueError("amount must be omitted when aggregation is count")
        if self.output_kind not in _OUTPUT_KINDS:
            allowed = ", ".join(sorted(_OUTPUT_KINDS))
            raise ValueError(f"output_kind must be one of: {allowed}")
        if (
            isinstance(self.bucket_months, bool)
            or not isinstance(self.bucket_months, int)
            or not 1 <= self.bucket_months <= 12
        ):
            raise ValueError("bucket_months must be an integer between 1 and 12")

        object.__setattr__(
            self,
            "segments",
            tuple(_coerce_segment_definition(segment) for segment in self.segments),
        )

    def to_dict(self) -> dict[str, Any]:
        """Return a detached, JSON-safe triangle definition."""
        segments = cast(tuple[SegmentDefinition, ...], self.segments)
        payload: dict[str, Any] = {
            "triangle_definition_id": self.triangle_definition_id,
            "schema_version": self.schema_version,
            "origin_date": self.origin_date,
            "development_date": self.development_date,
            "measure": _json_safe(self.measure),
            "aggregation": self.aggregation,
            "bucket_months": self.bucket_months,
            "output_kind": self.output_kind,
            "portfolio_id": _json_safe(self.portfolio_id),
            "segments": [segment.to_dict() for segment in segments],
        }
        if self.amount is not None:
            payload["amount"] = self.amount
        if self.valuation_date is not None:
            payload["valuation_date"] = _json_safe(self.valuation_date)
        if self.currency is not None:
            payload["currency"] = _json_safe(self.currency)
        return json.loads(json.dumps(payload, allow_nan=False, sort_keys=True))


def _coerce_segment_definition(
    value: SegmentDefinition | Mapping[str, Any],
) -> SegmentDefinition:
    if isinstance(value, SegmentDefinition):
        return value
    if not isinstance(value, Mapping):
        raise TypeError("segments must contain SegmentDefinition or mapping objects")
    if set(value) != {"name", "source"}:
        raise ValueError("segment mappings must contain only `name` and `source`")
    return SegmentDefinition(name=value["name"], source=value["source"])


def _validate_source_column(value: object, field_name: str) -> None:
    if not isinstance(value, str):
        raise TypeError(f"{field_name} must be a source column name")
    _validate_non_empty_string(value, field_name)


def _validate_non_empty_string(value: object, field_name: str) -> None:
    if not isinstance(value, str):
        raise TypeError(f"{field_name} must be a string")
    if not value.strip():
        raise ValueError(f"{field_name} must be non-empty")


def _validate_mapping_value(value: object, field_name: str) -> None:
    if isinstance(value, Mapping):
        _constant_mapping_value(value, field_name)
        return
    if isinstance(value, str):
        _validate_non_empty_string(value, field_name)
        return
    if isinstance(value, (date, datetime, Decimal, int, float, bool)):
        return
    raise TypeError(f"{field_name} must be a source column or constant")


def _constant_mapping_value(value: Mapping[str, object], field_name: str) -> object:
    if set(value) != {"const"}:
        raise ValueError(f"{field_name} constant mapping must contain only a `const` field")
    item = value["const"]
    if not isinstance(item, (date, datetime, Decimal, str, int, float, bool)):
        raise TypeError(f"{field_name} constant must be a scalar value")
    if isinstance(item, str):
        _validate_non_empty_string(item, field_name)
    return item


def _json_safe(value: object) -> object:
    if isinstance(value, datetime):
        return value.isoformat()
    if isinstance(value, date):
        return value.isoformat()
    if isinstance(value, Decimal):
        return str(value)
    if isinstance(value, Mapping):
        return {str(key): _json_safe(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_json_safe(item) for item in value]
    if value is None or isinstance(value, (str, int, float, bool)):
        return value
    raise TypeError(f"unsupported triangle definition value: {type(value).__name__}")
