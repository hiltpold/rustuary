from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass
from typing import Any


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

    def fit_predict(
        self,
        *,
        origin_periods: Iterable[int],
        development_ages: Iterable[int],
        rows: Iterable[Iterable[float | int | None]],
        cumulative: bool = True,
    ) -> dict[str, Any]:
        """Run chain ladder on a canonical dense triangle.

        The Python layer materializes user-friendly iterables into the dense
        shape required by the binding. All actuarial calculation and validation
        is delegated to ``rustuary-core`` through the compiled extension.
        """
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
