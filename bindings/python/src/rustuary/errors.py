from __future__ import annotations

from collections.abc import Sequence


class ColumnMappingError(ValueError):
    """A source column cannot be resolved to its canonical field."""

    def __init__(
        self,
        *,
        canonical_field: str,
        source_column: str,
        available_columns: Sequence[str],
        reason: str,
    ) -> None:
        self.canonical_field = canonical_field
        self.source_column = source_column
        self.available_columns = tuple(available_columns)
        self.reason = reason

        available = ", ".join(self.available_columns) or "(none)"
        super().__init__(
            f"canonical field `{canonical_field}` is mapped from source column "
            f"`{source_column}`, but {reason}. Available columns: {available}."
        )
