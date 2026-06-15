import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
WORKBENCH_PATH = REPO_ROOT / "notebooks" / "01_chain_ladder_workbench.ipynb"


def test_chain_ladder_workbench_executes_current_public_api(monkeypatch):
    notebook = json.loads(WORKBENCH_PATH.read_text())
    code_cells = [
        "".join(cell["source"])
        for cell in notebook["cells"]
        if cell["cell_type"] == "code"
    ]
    code = "\n".join(code_cells)

    assert "triangle.validate(" not in code
    assert "ry.ChainLadder(" not in code
    assert "ry.ClaimsMapping(" in code
    assert "ry.Triangle.from_frame(" in code

    monkeypatch.chdir(REPO_ROOT / "notebooks")
    namespace = {"__name__": "__main__"}
    for index, cell in enumerate(code_cells):
        exec(compile(cell, f"{WORKBENCH_PATH.name}:cell-{index}", "exec"), namespace)

    triangle = namespace["triangle"]
    assert triangle.data.num_rows == 6
    assert namespace["model_run_metadata"]["claims_mapping"]["currency"] == {"const": "CHF"}
