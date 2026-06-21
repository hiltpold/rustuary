import pytest

from rustuary import TriangleSet


def sample_triangle_set_payload():
    return {
        "diagnostics": {
            "source_record_count": 3,
            "triangle_count": 1,
            "cumulative_conversion_applied": True,
        },
        "triangles": [
            {
                "key": {
                    "portfolio_id": "Motor",
                    "segments": [
                        {"name": "country", "value": "CH"},
                        {"name": "coverage", "value": "MTPL"},
                    ],
                    "measure": "paid",
                    "display_path": "Motor/CH/MTPL",
                },
                "origin_periods": [2024, 2025],
                "development_ages": [12, 24],
                "rows": [[100.0, 150.0], [80.0, None]],
                "basis": "cumulative",
                "diagnostics": {
                    "source_record_count": 3,
                    "cumulative_conversion_applied": True,
                },
            }
        ],
    }


def test_triangle_set_wraps_rust_payload():
    payload = sample_triangle_set_payload()
    triangle_set = TriangleSet(payload)

    assert len(triangle_set) == 1
    assert triangle_set.diagnostics() == payload["diagnostics"]
    assert triangle_set.triangles() == payload["triangles"]
    assert list(triangle_set) == payload["triangles"]
    assert triangle_set.to_dict() == payload


def test_triangle_set_returns_detached_payload_copies():
    payload = sample_triangle_set_payload()
    triangle_set = TriangleSet(payload)

    payload["diagnostics"]["source_record_count"] = 999
    payload["triangles"][0]["rows"][0][0] = 999.0
    copied = triangle_set.to_dict()
    copied["triangles"][0]["rows"][0][0] = 500.0

    assert triangle_set.diagnostics()["source_record_count"] == 3
    assert triangle_set.triangles()[0]["rows"][0][0] == 100.0


def test_triangle_set_validates_required_payload_fields():
    with pytest.raises(ValueError, match="diagnostics"):
        TriangleSet({"triangles": []})
    with pytest.raises(ValueError, match="triangles"):
        TriangleSet({"diagnostics": {}})


def test_triangle_set_rejects_wrong_payload_type():
    with pytest.raises(TypeError, match="mapping"):
        TriangleSet(["not", "a", "payload"])
