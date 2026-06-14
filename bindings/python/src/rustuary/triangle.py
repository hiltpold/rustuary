from __future__ import annotations

from dataclasses import dataclass
from typing import Any


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
    cumulative: bool = True

    @classmethod
    def from_frame(
        cls,
        data: Any,
        *,
        origin: str,
        development: str,
        value: str,
        cumulative: bool = True,
    ) -> "Triangle":
        return cls(
            data=data,
            origin=origin,
            development=development,
            value=value,
            cumulative=cumulative,
        )
