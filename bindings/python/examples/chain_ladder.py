import rustuary as ry

triangle = ry.Triangle.from_frame(
    [
        {"accident_year": 2020, "development_month": 12, "paid": 100.0},
        {"accident_year": 2020, "development_month": 24, "paid": 180.0},
        {"accident_year": 2020, "development_month": 36, "paid": 240.0},
        {"accident_year": 2021, "development_month": 12, "paid": 120.0},
        {"accident_year": 2021, "development_month": 24, "paid": 210.0},
        {"accident_year": 2022, "development_month": 12, "paid": 150.0},
    ],
    origin="accident_year",
    development="development_month",
    value="paid",
)

model = ry.ChainLadder(tail_factor=1.0)
result = model.fit_predict(triangle)

print(result.summary())
print(result.diagnostics()["selected_factors"])
