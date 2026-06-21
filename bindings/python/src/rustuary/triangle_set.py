from __future__ import annotations

from collections.abc import Iterator, Mapping
from dataclasses import dataclass
from typing import Any, cast

JsonValue = Any


@dataclass(frozen=True, slots=True)
class TriangleSet:
    """Notebook-friendly wrapper around Rust-built triangle-set output."""

    _payload: Mapping[str, JsonValue]

    def __post_init__(self) -> None:
        payload = _validated_payload(self._payload)
        object.__setattr__(self, "_payload", payload)

    def __iter__(self) -> Iterator[dict[str, JsonValue]]:
        return iter(self.triangles())

    def __len__(self) -> int:
        return len(self.triangles())

    def to_dict(self) -> dict[str, JsonValue]:
        """Return a detached copy of the raw triangle-set payload."""
        return _copy_json_value(dict(self._payload))

    def diagnostics(self) -> dict[str, JsonValue]:
        """Return build diagnostics from the Rust triangle construction engine."""
        diagnostics = self._payload["diagnostics"]
        if not isinstance(diagnostics, Mapping):
            raise ValueError("TriangleSet payload field `diagnostics` must be a mapping")
        return _copy_json_value(dict(diagnostics))

    def triangles(self) -> list[dict[str, JsonValue]]:
        """Return built triangle payloads in deterministic Rust key order."""
        triangles = self._payload["triangles"]
        if not isinstance(triangles, list):
            raise ValueError("TriangleSet payload field `triangles` must be a list")
        return cast(list[dict[str, JsonValue]], _copy_json_value(triangles))


def _validated_payload(payload: Mapping[str, JsonValue]) -> dict[str, JsonValue]:
    if not isinstance(payload, Mapping):
        raise TypeError("TriangleSet payload must be a mapping")
    if "diagnostics" not in payload:
        raise ValueError("TriangleSet payload is missing `diagnostics`")
    if "triangles" not in payload:
        raise ValueError("TriangleSet payload is missing `triangles`")
    return _copy_json_value(dict(payload))


def _copy_json_value(value: JsonValue) -> JsonValue:
    if isinstance(value, dict):
        return {key: _copy_json_value(item) for key, item in value.items()}
    if isinstance(value, list):
        return [_copy_json_value(item) for item in value]
    return value
