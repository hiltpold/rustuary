from __future__ import annotations

from dataclasses import dataclass
from typing import Any, overload

from .mapping import ClaimsMapping, DevelopmentUnit, MappingValue


class _Unset:
    __slots__ = ()


_UNSET = _Unset()


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
    @overload
    def from_frame(
        cls,
        data: Any,
        *,
        mapping: ClaimsMapping,
    ) -> "Triangle": ...

    @classmethod
    @overload
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
    ) -> "Triangle": ...

    @classmethod
    def from_frame(
        cls,
        data: Any,
        *,
        mapping: ClaimsMapping | None = None,
        origin: str | _Unset = _UNSET,
        development: str | _Unset = _UNSET,
        value: str | _Unset = _UNSET,
        cumulative: bool | str | _Unset = _UNSET,
        portfolio: MappingValue | None | _Unset = _UNSET,
        valuation_date: MappingValue | None | _Unset = _UNSET,
        measure: MappingValue | None | _Unset = _UNSET,
        currency: MappingValue | None | _Unset = _UNSET,
        origin_type: str | None | _Unset = _UNSET,
        development_unit: DevelopmentUnit | None | _Unset = _UNSET,
    ) -> "Triangle":
        """Create a triangle using named mapping fields or a reusable mapping.

        Pass either ``mapping=ClaimsMapping(...)`` or the required ``origin``,
        ``development``, and ``value`` fields. The two forms cannot be mixed.
        Dataframe normalization is performed by a later adapter stage.
        """
        named_arguments = {
            "origin": origin,
            "development": development,
            "value": value,
            "cumulative": cumulative,
            "portfolio": portfolio,
            "valuation_date": valuation_date,
            "measure": measure,
            "currency": currency,
            "origin_type": origin_type,
            "development_unit": development_unit,
        }

        if mapping is not None:
            if not isinstance(mapping, ClaimsMapping):
                raise TypeError("mapping must be a ClaimsMapping")

            mixed_fields = [
                name
                for name, argument in named_arguments.items()
                if not isinstance(argument, _Unset)
            ]
            if mixed_fields:
                fields = ", ".join(mixed_fields)
                raise TypeError(f"mapping cannot be combined with named mapping fields: {fields}")
            resolved_mapping = mapping
        else:
            missing_fields = [
                name
                for name in ("origin", "development", "value")
                if isinstance(named_arguments[name], _Unset)
            ]
            if missing_fields:
                fields = ", ".join(missing_fields)
                raise TypeError(f"missing required named mapping fields: {fields}")

            assert not isinstance(origin, _Unset)
            assert not isinstance(development, _Unset)
            assert not isinstance(value, _Unset)
            resolved_mapping = ClaimsMapping(
                origin=origin,
                development=development,
                value=value,
                cumulative=True if isinstance(cumulative, _Unset) else cumulative,
                portfolio=None if isinstance(portfolio, _Unset) else portfolio,
                valuation_date=(None if isinstance(valuation_date, _Unset) else valuation_date),
                measure=None if isinstance(measure, _Unset) else measure,
                currency=None if isinstance(currency, _Unset) else currency,
                origin_type=None if isinstance(origin_type, _Unset) else origin_type,
                development_unit=(
                    None if isinstance(development_unit, _Unset) else development_unit
                ),
            )

        return cls(
            data=data,
            origin=resolved_mapping.origin,
            development=resolved_mapping.development,
            value=resolved_mapping.value,
            cumulative=resolved_mapping.cumulative,
            portfolio=resolved_mapping.portfolio,
            valuation_date=resolved_mapping.valuation_date,
            measure=resolved_mapping.measure,
            currency=resolved_mapping.currency,
            origin_type=resolved_mapping.origin_type,
            development_unit=resolved_mapping.development_unit,
        )
