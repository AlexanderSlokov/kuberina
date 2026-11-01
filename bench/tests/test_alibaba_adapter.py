"""End-to-end test of the trace adapter on a fixture log of a dozen lines (X-2).

No network and no simulator run: the fixtures hold five pods, one of them a tuned
clone, offered to two policies that decide differently.
"""

import csv
from pathlib import Path

import yaml

from alibaba.adapter import main
from alibaba.demand_check import emitted_gpu_milli

FIXTURES = Path(__file__).parent / "fixtures" / "alibaba"
LOGGED_ARRIVED_GPU_MILLI = 17810  # last [Alloc] line of fgd.simon.log


def run_adapter(out: Path, pods: Path = FIXTURES / "pods.csv") -> int:
    return main(
        [
            "--nodes",
            str(FIXTURES / "nodes.csv"),
            "--pods",
            str(pods),
            "--log",
            f"FGD={FIXTURES / 'fgd.simon.log'}",
            "--log",
            f"Random={FIXTURES / 'random.simon.log'}",
            "--out",
            str(out),
        ]
    )


def test_writes_the_three_documents(tmp_path):
    assert run_adapter(tmp_path) == 0
    assert sorted(p.name for p in tmp_path.iterdir()) == [
        "infra.yaml",
        "sequence.csv",
        "workloads.yaml",
    ]


def test_emitted_gpu_demand_equals_logged_arrived_gpu_milli(tmp_path):
    run_adapter(tmp_path)
    workloads = yaml.safe_load((tmp_path / "workloads.yaml").read_text())
    assert emitted_gpu_milli(workloads) == LOGGED_ARRIVED_GPU_MILLI


def test_infra_lists_every_trace_node(tmp_path):
    run_adapter(tmp_path)
    infra = yaml.safe_load((tmp_path / "infra.yaml").read_text())
    assert [n["name"] for n in infra["nodes"]] == [
        "openb-node-0000",
        "openb-node-0001",
        "openb-node-0002",
    ]


def test_sequence_has_one_row_per_arrival_with_both_policies(tmp_path):
    run_adapter(tmp_path)
    with (tmp_path / "sequence.csv").open() as stream:
        rows = list(csv.DictReader(stream))
    assert [(r["node_FGD"], r["node_Random"]) for r in rows] == [
        ("openb-node-0002", "openb-node-0002"),
        ("openb-node-0000", "openb-node-0000"),
        ("openb-node-0002", ""),
        ("openb-node-0001", "openb-node-0002"),
        ("", ""),
    ]


def test_malformed_log_argument_names_the_expected_shape(tmp_path, capsys):
    status = main(
        [
            "--nodes",
            "n.csv",
            "--pods",
            "p.csv",
            "--log",
            "simon.log",
            "--out",
            str(tmp_path),
        ]
    )
    assert status == 1
    assert "POLICY=PATH" in capsys.readouterr().err


def test_demand_mismatch_writes_nothing_and_names_the_arrival(tmp_path, capsys):
    altered = tmp_path / "pods.csv"
    altered.write_text(
        (FIXTURES / "pods.csv")
        .read_text()
        .replace("3152,5600,1,810", "3152,5600,1,800")
    )
    out = tmp_path / "out"
    assert run_adapter(out, pods=altered) == 1
    assert not out.exists()
    assert "arrival index 1" in capsys.readouterr().err


def test_missing_input_file_is_reported_without_a_traceback(tmp_path, capsys):
    assert run_adapter(tmp_path / "out", pods=tmp_path / "absent.csv") == 1
    assert "absent.csv" in capsys.readouterr().err


def test_every_arrival_equals_its_sequence_row(tmp_path):
    run_adapter(tmp_path)
    workloads = yaml.safe_load((tmp_path / "workloads.yaml").read_text())
    with (tmp_path / "sequence.csv").open() as stream:
        rows = list(csv.DictReader(stream))
    declarations = workloads["namespaces"]["paib-gpu"]
    assert [d["arrival"] for d in declarations] == [
        int(r["arrival_index"]) for r in rows
    ]
