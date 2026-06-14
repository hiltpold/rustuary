from rustuary import Triangle


def test_triangle_from_frame_stores_mapping():
    triangle = Triangle.from_frame(
        data=[{"accident_year": 2024, "development_month": 12, "paid": 100.0}],
        origin="accident_year",
        development="development_month",
        value="paid",
    )

    assert triangle.origin == "accident_year"
    assert triangle.development == "development_month"
    assert triangle.value == "paid"
