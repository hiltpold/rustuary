from __future__ import annotations

from collections.abc import Iterator, Mapping, Sequence
from dataclasses import dataclass
from typing import Any, cast

JsonValue = Any


@dataclass(frozen=True, slots=True)
class TriangleSet:
    """Notebook-friendly wrapper around Rust-built triangle-set output."""

    _payload: Mapping[str, JsonValue]
    audit_input: Mapping[str, JsonValue] | None = None

    def __post_init__(self) -> None:
        payload = _validated_payload(self._payload)
        object.__setattr__(self, "_payload", payload)
        if self.audit_input is not None:
            object.__setattr__(self, "audit_input", _copy_json_value(dict(self.audit_input)))

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
        return _copy_triangles(self._payload)

    def keys(self) -> list[dict[str, JsonValue]]:
        """Return deterministic grouping keys for the built triangles."""
        return [_triangle_key(triangle) for triangle in self.triangles()]

    def get(
        self,
        key: Mapping[str, JsonValue] | None = None,
        *,
        portfolio_id: str | None = None,
        measure: str | None = None,
        segments: Mapping[str, str] | Sequence[Mapping[str, str]] | None = None,
    ) -> dict[str, JsonValue] | None:
        """Return the triangle matching a key or named key fields."""
        target_key = _target_key(
            key,
            portfolio_id=portfolio_id,
            measure=measure,
            segments=segments,
        )
        match_segment_names = key is None and isinstance(segments, Mapping)
        for triangle in self.triangles():
            triangle_key = _triangle_key(triangle)
            if match_segment_names and _keys_equal_by_segment_names(
                triangle_key,
                target_key,
            ):
                return triangle
            if not match_segment_names and _keys_equal(triangle_key, target_key):
                return triangle
        return None

    def tree(self) -> dict[str, JsonValue]:
        """Return a folder-style display tree derived from portfolio and segments."""
        tree: dict[str, JsonValue] = {}
        for triangle_index, key in enumerate(self.keys()):
            folder = tree
            for part in _display_path_parts(key):
                next_folder = folder.setdefault(part, {})
                if not isinstance(next_folder, dict):
                    raise ValueError(f"TriangleSet tree path conflicts at `{part}`")
                folder = next_folder
            triangles = folder.setdefault("_triangles", [])
            if not isinstance(triangles, list):
                raise ValueError("TriangleSet tree path conflicts at `_triangles`")
            triangles.append(
                {
                    "measure": key["measure"],
                    "triangle_index": triangle_index,
                    "key": _copy_json_value(key),
                }
            )
        return tree

    def audit_trail(self) -> dict[str, JsonValue]:
        """Return triangle-definition input evidence and build diagnostics."""
        return {
            "input": _copy_json_value(self.audit_input),
            "diagnostics": self.diagnostics(),
        }


def _copy_triangles(payload: Mapping[str, JsonValue]) -> list[dict[str, JsonValue]]:
    triangles = payload["triangles"]
    if not isinstance(triangles, list):
        raise ValueError("TriangleSet payload field `triangles` must be a list")
    return cast(list[dict[str, JsonValue]], _copy_json_value(triangles))


def _triangle_key(triangle: Mapping[str, JsonValue]) -> dict[str, JsonValue]:
    key = triangle.get("key")
    if not isinstance(key, Mapping):
        raise ValueError("TriangleSet triangle payload is missing mapping field `key`")
    return cast(dict[str, JsonValue], _copy_json_value(dict(key)))


def _target_key(
    key: Mapping[str, JsonValue] | None,
    *,
    portfolio_id: str | None,
    measure: str | None,
    segments: Mapping[str, str] | Sequence[Mapping[str, str]] | None,
) -> dict[str, JsonValue]:
    named_fields = [portfolio_id is not None, measure is not None, segments is not None]
    if key is not None and any(named_fields):
        raise TypeError("key cannot be combined with portfolio_id, measure, or segments")
    if key is not None:
        return cast(dict[str, JsonValue], _copy_json_value(dict(key)))
    if portfolio_id is None or measure is None:
        raise TypeError("get requires either key or portfolio_id and measure")
    return {
        "portfolio_id": portfolio_id,
        "segments": _segment_items(segments),
        "measure": measure,
    }


def _segment_items(
    segments: Mapping[str, str] | Sequence[Mapping[str, str]] | None,
) -> list[dict[str, str]]:
    if segments is None:
        return []
    if isinstance(segments, Mapping):
        return [{"name": str(name), "value": str(value)} for name, value in segments.items()]
    return [
        {
            "name": str(segment["name"]),
            "value": str(segment["value"]),
        }
        for segment in segments
    ]


def _keys_equal(left: Mapping[str, JsonValue], right: Mapping[str, JsonValue]) -> bool:
    return (
        left.get("portfolio_id") == right.get("portfolio_id")
        and left.get("measure") == right.get("measure")
        and _segment_items_for_key(left) == _segment_items_for_key(right)
    )


def _keys_equal_by_segment_names(
    left: Mapping[str, JsonValue],
    right: Mapping[str, JsonValue],
) -> bool:
    return (
        left.get("portfolio_id") == right.get("portfolio_id")
        and left.get("measure") == right.get("measure")
        and dict(_segment_items_for_key(left)) == dict(_segment_items_for_key(right))
    )


def _segment_items_for_key(key: Mapping[str, JsonValue]) -> list[tuple[str, str]]:
    segments = key.get("segments", [])
    if not isinstance(segments, list):
        raise ValueError("TriangleSet key field `segments` must be a list")
    items = []
    for segment in segments:
        if not isinstance(segment, Mapping):
            raise ValueError("TriangleSet key segments must be mappings")
        items.append((str(segment["name"]), str(segment["value"])))
    return items


def _display_path_parts(key: Mapping[str, JsonValue]) -> list[str]:
    portfolio_id = key.get("portfolio_id")
    if portfolio_id is None:
        raise ValueError("TriangleSet key is missing `portfolio_id`")
    return [str(portfolio_id)] + [value for _, value in _segment_items_for_key(key)]


def _validated_payload(payload: Mapping[str, JsonValue]) -> dict[str, JsonValue]:
    if not isinstance(payload, Mapping):
        raise TypeError("TriangleSet payload must be a mapping")
    if "diagnostics" not in payload:
        raise ValueError("TriangleSet payload is missing `diagnostics`")
    if "triangles" not in payload:
        raise ValueError("TriangleSet payload is missing `triangles`")
    _copy_triangles(payload)
    return _copy_json_value(dict(payload))


def _copy_json_value(value: JsonValue) -> JsonValue:
    if isinstance(value, dict):
        return {key: _copy_json_value(item) for key, item in value.items()}
    if isinstance(value, list):
        return [_copy_json_value(item) for item in value]
    return value
