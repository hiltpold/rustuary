import rustuary as ry

claims = [
    {"accident_year": 2022, "development_month": 12, "paid": 100.0},
    {"accident_year": 2022, "development_month": 24, "paid": 180.0},
]

triangle = ry.Triangle.from_frame(
    claims,
    origin="accident_year",
    development="development_month",
    value="paid",
)

print(triangle)
