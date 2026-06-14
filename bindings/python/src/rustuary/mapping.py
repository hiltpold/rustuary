from __future__ import annotations

from dataclasses import dataclass
from datetime import date, datetime
from decimal import Decimal
from typing import Literal, Mapping, TypeAlias

MappingScalar: TypeAlias = str | int | float | bool | date | datetime | Decimal
MappingValue: TypeAlias = MappingScalar | Mapping[str, MappingScalar]
DevelopmentUnit: TypeAlias = Literal["months", "quarters", "years"]

_DEVELOPMENT_UNITS = frozenset(("months", "quarters", "years"))


def _validate_source_column(value: object, field_name: str) -> None:
    if not isinstance(value, str):
        raise TypeError(f"{field_name} must be a source column name")
    if not value.strip():
        raise ValueError(f"{field_name} must be a non-empty source column name")


@dataclass(frozen=True, slots=True)
class ClaimsMapping:
    """Map external claims columns and constants to canonical claims fields."""

    origin: str
    development: str
    value: str
    cumulative: bool | str = True
    portfolio: MappingValue | None = None
    valuation_date: MappingValue | None = None
    measure: MappingValue | None = None
    currency: MappingValue | None = None
    origin_type: str | None = None
    development_unit: DevelopmentUnit | None = None

    def __post_init__(self) -> None:
        _validate_source_column(self.origin, "origin")
        _validate_source_column(self.development, "development")
        _validate_source_column(self.value, "value")

        if isinstance(self.cumulative, str):
            _validate_source_column(self.cumulative, "cumulative")
        elif not isinstance(self.cumulative, bool):
            raise TypeError("cumulative must be a boolean or source column name")

        if self.development_unit is not None and self.development_unit not in _DEVELOPMENT_UNITS:
            allowed = ", ".join(sorted(_DEVELOPMENT_UNITS))
            raise ValueError(f"development_unit must be one of: {allowed}")


@dataclass(frozen=True, slots=True)
class ExposureMapping:
    """Map external exposure columns and constants to canonical exposure fields."""

    origin: str
    value: str
    exposure_measure: MappingValue
    portfolio: MappingValue | None = None
    valuation_date: MappingValue | None = None
    currency: MappingValue | None = None

    def __post_init__(self) -> None:
        _validate_source_column(self.origin, "origin")
        _validate_source_column(self.value, "value")

        if self.exposure_measure is None:
            raise TypeError("exposure_measure must be a source column or constant")
        if isinstance(self.exposure_measure, str) and not self.exposure_measure.strip():
            raise ValueError("exposure_measure must be a non-empty source column or constant")
