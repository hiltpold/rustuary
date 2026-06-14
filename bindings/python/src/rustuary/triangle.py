from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .mapping import ClaimsMapping, DevelopmentUnit, MappingValue


@dataclass(frozen=True)
class Triangle:
    """Python-side triangle placeholder.

    This class will normalize pandas, polars, or pyarrow inputs before calling
    the Rust engine. It intentionally avoids duplicating actuarial formulas.
    """

    data: Any
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

    @classmethod
    def from_frame(
        cls,
        data: Any,
        *,
        origin: str,
        development: str,
        value: str,
        cumulative: bool | str = True,
        portfolio: MappingValue | None = None,
        valuation_date: MappingValue | None = None,
        measure: MappingValue | None = None,
        currency: MappingValue | None = None,
        origin_type: str | None = None,
        development_unit: DevelopmentUnit | None = None,
    ) -> "Triangle":
        mapping = ClaimsMapping(
            origin=origin,
            development=development,
            value=value,
            cumulative=cumulative,
            portfolio=portfolio,
            valuation_date=valuation_date,
            measure=measure,
            currency=currency,
            origin_type=origin_type,
            development_unit=development_unit,
        )

        return cls(
            data=data,
            origin=mapping.origin,
            development=mapping.development,
            value=mapping.value,
            cumulative=mapping.cumulative,
            portfolio=mapping.portfolio,
            valuation_date=mapping.valuation_date,
            measure=mapping.measure,
            currency=mapping.currency,
            origin_type=mapping.origin_type,
            development_unit=mapping.development_unit,
        )
