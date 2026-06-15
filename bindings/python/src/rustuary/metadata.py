from __future__ import annotations

import json
from collections.abc import Mapping
from dataclasses import dataclass, field, fields
from datetime import date, datetime
from decimal import Decimal
from typing import Any

from .mapping import ClaimsMapping

CLAIMS_SCHEMA_NAME = "claims_triangle"
CLAIMS_SCHEMA_VERSION = "1"


def _json_safe(value: Any) -> Any:
    if isinstance(value, datetime):
        return value.isoformat()
    if isinstance(value, date):
        return value.isoformat()
    if isinstance(value, Decimal):
        return str(value)
    if isinstance(value, Mapping):
        return {str(key): _json_safe(item) for key, item in value.items()}
    if value is None or isinstance(value, (str, int, float, bool)):
        return value
    raise TypeError(f"unsupported model-run metadata value: {type(value).__name__}")


@dataclass(frozen=True, slots=True, init=False)
class ModelRunMetadata:
    """Serializable metadata available before a reserving model run exists."""

    canonical_schema: str
    canonical_schema_version: str
    _claims_mapping_json: str = field(repr=False)

    def __init__(
        self,
        *,
        claims_mapping: Mapping[str, Any],
        canonical_schema: str = CLAIMS_SCHEMA_NAME,
        canonical_schema_version: str = CLAIMS_SCHEMA_VERSION,
    ) -> None:
        mapping_json = json.dumps(
            _json_safe(claims_mapping),
            allow_nan=False,
            sort_keys=True,
            separators=(",", ":"),
        )
        object.__setattr__(self, "canonical_schema", canonical_schema)
        object.__setattr__(self, "canonical_schema_version", canonical_schema_version)
        object.__setattr__(self, "_claims_mapping_json", mapping_json)

    @classmethod
    def from_claims_mapping(cls, mapping: ClaimsMapping) -> ModelRunMetadata:
        snapshot = {field.name: getattr(mapping, field.name) for field in fields(mapping)}
        return cls(claims_mapping=snapshot)

    @property
    def claims_mapping(self) -> dict[str, Any]:
        """Return a detached copy of the persisted claims mapping."""
        value = json.loads(self._claims_mapping_json)
        if not isinstance(value, dict):
            raise TypeError("persisted claims mapping must be a JSON object")
        return value

    def to_dict(self) -> dict[str, Any]:
        """Return a JSON-safe model-run metadata payload."""
        return {
            "canonical_schema": self.canonical_schema,
            "canonical_schema_version": self.canonical_schema_version,
            "claims_mapping": self.claims_mapping,
        }
