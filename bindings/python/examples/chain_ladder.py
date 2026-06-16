import rustuary as ry

model = ry.ChainLadder(tail_factor=1.0)
result = model.fit_predict(
    origin_periods=[2020, 2021, 2022],
    development_ages=[12, 24, 36],
    rows=[
        [100.0, 180.0, 240.0],
        [120.0, 210.0, None],
        [150.0, None, None],
    ],
)

print(result["origins"])
