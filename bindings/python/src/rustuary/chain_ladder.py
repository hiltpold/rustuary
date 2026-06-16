from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass
from typing import Any, overload

from .triangle import Triangle


def _load_rust_extension() -> Any:
    try:
        from . import _rust  # type: ignore[import-untyped]
    except ImportError as exc:
        raise ImportError(
            "ChainLadder requires the compiled rustuary._rust extension. "
            "Run `uv run maturin develop` from bindings/python."
        ) from exc
    return _rust


@dataclass(frozen=True, slots=True)
class ChainLadder:
    """Actuary-facing chain-ladder model backed by the Rust core."""

    tail_factor: float = 1.0

    @overload
    def fit_predict(self, triangle: Triangle, /) -> dict[str, Any]: ...

    @overload
    def fit_predict(
        self,
        triangle: None = None,
        /,
        *,
        origin_periods: Iterable[int],
        development_ages: Iterable[int],
        rows: Iterable[Iterable[float | int | None]],
        cumulative: bool = True,
    ) -> dict[str, Any]: ...

    def fit_predict(
        self,
        triangle: Triangle | None = None,
        /,
        *,
        origin_periods: Iterable[int] | None = None,
        development_ages: Iterable[int] | None = None,
        rows: Iterable[Iterable[float | int | None]] | None = None,
        cumulative: bool = True,
    ) -> dict[str, Any]:
        """Run chain ladder on a mapped ``Triangle`` or canonical dense axes.

        For mapped ``Triangle`` inputs, Python reshapes canonical long-form
        cells into dense axes and rows. All actuarial calculation and validation
        is delegated to ``rustuary-core`` through the compiled extension.
        """
        if triangle is not None:
            if origin_periods is not None or development_ages is not None or rows is not None:
                raise TypeError("triangle cannot be combined with dense triangle arguments")
            origin_periods, development_ages, rows, cumulative = _dense_from_triangle(triangle)
        elif origin_periods is None or development_ages is None or rows is None:
            raise TypeError(
                "fit_predict requires either a Triangle or origin_periods, "
                "development_ages, and rows"
            )

        rust = _load_rust_extension()
        return rust.chain_ladder(
            origin_periods=[int(period) for period in origin_periods],
            development_ages=[int(age) for age in development_ages],
            rows=[
                [None if value is None else float(value) for value in row]
                for row in rows
            ],
            cumulative=cumulative,
            tail_factor=float(self.tail_factor),
        )


def _dense_from_triangle(
    triangle: Triangle,
) -> tuple[list[int], list[int], list[list[float | None]], bool]:
    required_fields = ("origin_period", "development_age", "amount", "is_cumulative")
    missing_fields = [field for field in required_fields if field not in triangle.data.column_names]
    if missing_fields:
        fields = ", ".join(missing_fields)
        raise ValueError(f"Triangle data is missing canonical fields: {fields}")

    records = triangle.data.select(required_fields).to_pylist()
    if not records:
        raise ValueError("Triangle data must contain at least one canonical cell")

    basis_values: set[bool] = set()
    cell_values: dict[tuple[int, int], float | None] = {}
    origin_values: set[int] = set()
    development_values: set[int] = set()

    for record in records:
        origin_period = _integer_axis_value(record["origin_period"], "origin_period")
        development_age = _integer_axis_value(record["development_age"], "development_age")
        amount = _amount_value(record["amount"])
        cumulative = record["is_cumulative"]

        if not isinstance(cumulative, bool):
            raise ValueError("Triangle data field is_cumulative must contain boolean values")

        key = (origin_period, development_age)
        if key in cell_values:
            raise ValueError(
                "Triangle data contains duplicate cells for "
                f"origin_period={origin_period}, development_age={development_age}"
            )

        basis_values.add(cumulative)
        cell_values[key] = amount
        origin_values.add(origin_period)
        development_values.add(development_age)

    if len(basis_values) != 1:
        raise ValueError("Triangle data must use a single cumulative or incremental basis")

    origin_periods = sorted(origin_values)
    development_ages = sorted(development_values)
    rows = [
        [cell_values.get((origin_period, development_age)) for development_age in development_ages]
        for origin_period in origin_periods
    ]

    return origin_periods, development_ages, rows, basis_values.pop()


def _integer_axis_value(value: Any, field_name: str) -> int:
    if value is None or isinstance(value, bool):
        raise ValueError(f"Triangle data field {field_name} must contain integer values")
    if isinstance(value, float) and not value.is_integer():
        raise ValueError(f"Triangle data field {field_name} must contain integer values")

    try:
        return int(value)
    except (TypeError, ValueError) as exc:
        raise ValueError(
            f"Triangle data field {field_name} must contain integer values"
        ) from exc


def _amount_value(value: Any) -> float | None:
    if value is None:
        return None
    if isinstance(value, bool):
        raise ValueError("Triangle data field amount must contain numeric values")

    try:
        return float(value)
    except (TypeError, ValueError) as exc:
        raise ValueError("Triangle data field amount must contain numeric values") from exc
