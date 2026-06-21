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


def reversed_segment_triangle_set_payload():
    payload = sample_triangle_set_payload()
    key = payload["triangles"][0]["key"]
    key["segments"] = [
        {"name": "coverage", "value": "MTPL"},
        {"name": "country", "value": "CH"},
    ]
    key["display_path"] = "Motor/MTPL/CH"
    return payload


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


def test_triangle_set_keys_and_get_return_matching_triangle():
    payload = sample_triangle_set_payload()
    triangle_set = TriangleSet(payload)

    keys = triangle_set.keys()

    assert keys == [payload["triangles"][0]["key"]]
    assert triangle_set.get(keys[0]) == payload["triangles"][0]
    assert (
        triangle_set.get(
            portfolio_id="Motor",
            measure="paid",
            segments={"country": "CH", "coverage": "MTPL"},
        )
        == payload["triangles"][0]
    )
    assert (
        triangle_set.get(
            portfolio_id="Motor",
            measure="paid",
            segments={"coverage": "MTPL", "country": "CH"},
        )
        == payload["triangles"][0]
    )
    assert (
        triangle_set.get(
            portfolio_id="Motor",
            measure="paid",
            segments=[
                {"name": "country", "value": "CH"},
                {"name": "coverage", "value": "MTPL"},
            ],
        )
        == payload["triangles"][0]
    )
    assert (
        triangle_set.get(
            portfolio_id="Motor",
            measure="paid",
            segments=[
                {"name": "coverage", "value": "MTPL"},
                {"name": "country", "value": "CH"},
            ],
        )
        is None
    )
    assert triangle_set.get(portfolio_id="Motor", measure="incurred") is None


def test_triangle_set_get_rejects_ambiguous_arguments():
    triangle_set = TriangleSet(sample_triangle_set_payload())

    with pytest.raises(TypeError, match="cannot be combined"):
        triangle_set.get(triangle_set.keys()[0], portfolio_id="Motor")
    with pytest.raises(TypeError, match="portfolio_id and measure"):
        triangle_set.get(portfolio_id="Motor")


def test_triangle_set_tree_is_derived_from_segment_order():
    triangle_set = TriangleSet(sample_triangle_set_payload())
    reversed_triangle_set = TriangleSet(reversed_segment_triangle_set_payload())

    assert triangle_set.tree() == {
        "Motor": {
            "CH": {
                "MTPL": {
                    "_triangles": [
                        {
                            "measure": "paid",
                            "triangle_index": 0,
                            "key": sample_triangle_set_payload()["triangles"][0]["key"],
                        }
                    ]
                }
            }
        }
    }
    assert list(reversed_triangle_set.tree()["Motor"]) == ["MTPL"]
    assert reversed_triangle_set.keys()[0]["segments"] == [
        {"name": "coverage", "value": "MTPL"},
        {"name": "country", "value": "CH"},
    ]


def test_triangle_set_audit_trail_returns_detached_triangle_definition_metadata():
    audit_input = {"triangle_definition": {"triangle_definition_id": "paid-claims-v1"}}
    triangle_set = TriangleSet(sample_triangle_set_payload(), audit_input=audit_input)

    audit_input["triangle_definition"]["triangle_definition_id"] = "changed"
    audit_trail = triangle_set.audit_trail()
    audit_trail["input"]["triangle_definition"]["triangle_definition_id"] = "also-changed"

    assert (
        triangle_set.audit_trail()["input"]["triangle_definition"]["triangle_definition_id"]
        == "paid-claims-v1"
    )
    assert triangle_set.audit_trail()["diagnostics"] == sample_triangle_set_payload()[
        "diagnostics"
    ]


def test_triangle_set_validates_required_payload_fields():
    with pytest.raises(ValueError, match="diagnostics"):
        TriangleSet({"triangles": []})
    with pytest.raises(ValueError, match="triangles"):
        TriangleSet({"diagnostics": {}})


def test_triangle_set_rejects_wrong_payload_type():
    with pytest.raises(TypeError, match="mapping"):
        TriangleSet(["not", "a", "payload"])
