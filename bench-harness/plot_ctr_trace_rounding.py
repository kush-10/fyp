#!/usr/bin/env python3

import argparse
import csv
import json
import math
import re
from pathlib import Path

import matplotlib.pyplot as plt


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def latest_run_dir(output_root: Path) -> Path:
    run_dirs = sorted([path for path in output_root.iterdir() if path.is_dir()])
    if not run_dirs:
        raise FileNotFoundError(f"no benchmark runs in {output_root}")
    return run_dirs[-1]


def load_ctr_rows(aggregated: dict) -> list[dict]:
    rows = []
    pattern = re.compile(r"^aes-ctr-(\d+)blk$")

    for bench in aggregated.get("benchmarks", []):
        benchmark_id = bench.get("benchmark_id", "")
        match = pattern.match(benchmark_id)
        if not match:
            continue

        blocks = int(match.group(1))
        prove_seconds = (
            bench.get("timings", {}).get("prove_seconds", {}).get("median")
        )
        user_cycles = bench.get("cycles", {}).get("user_cycles", {}).get("median")
        total_cycles = bench.get("cycles", {}).get("total_cycles", {}).get("median")

        if prove_seconds is None or user_cycles is None or total_cycles is None:
            continue

        rows.append(
            {
                "blocks": blocks,
                "prove_seconds_median": float(prove_seconds),
                "user_cycles_median": float(user_cycles),
                "total_cycles_median": float(total_cycles),
            }
        )

    rows.sort(key=lambda row: row["blocks"])
    return rows


def write_csv(path: Path, rows: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "blocks",
                "prove_seconds_median",
                "user_cycles_median",
                "total_cycles_median",
            ],
        )
        writer.writeheader()
        for row in rows:
            writer.writerow(row)


def save_plot(path: Path, rows: list[dict]) -> None:
    blocks = [row["blocks"] for row in rows]
    prove_seconds = [row["prove_seconds_median"] for row in rows]
    user_cycles = [row["user_cycles_median"] for row in rows]
    total_cycles = [row["total_cycles_median"] for row in rows]

    fig, (ax_cycles, ax_time) = plt.subplots(
        2,
        1,
        figsize=(9, 7),
        sharex=True,
        gridspec_kw={"height_ratios": [2, 1]},
    )

    ax_cycles.plot(
        blocks,
        user_cycles,
        marker="o",
        linewidth=2,
        color="#4e79a7",
        label="Guest cycles (user)",
    )
    ax_cycles.step(
        blocks,
        total_cycles,
        where="mid",
        linewidth=2,
        color="#f28e2b",
        label="Trace cycles (total)",
    )

    for block, cycles in zip(blocks, total_cycles):
        k = int(round(math.log2(cycles)))
        ax_cycles.annotate(
            f"2^{k}",
            (block, cycles),
            textcoords="offset points",
            xytext=(0, 6),
            ha="center",
            fontsize=8,
            color="#f28e2b",
        )

    ax_cycles.set_ylabel("Cycles")
    ax_cycles.set_title("AES-CTR 1--16 Blocks: Guest Cycles, Trace Cycles, and Prover Time")
    ax_cycles.grid(axis="y", linestyle="--", linewidth=0.6, alpha=0.35)
    ax_cycles.legend(loc="upper left")

    ax_time.plot(
        blocks,
        prove_seconds,
        marker="o",
        linewidth=2,
        color="#59a14f",
    )
    ax_time.set_xlabel("Blocks")
    ax_time.set_ylabel("Prove Time (s)")
    ax_time.grid(axis="y", linestyle="--", linewidth=0.6, alpha=0.35)

    fig.tight_layout()
    path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(path, dpi=180)
    plt.close(fig)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Plot AES-CTR trace-rounding behavior for 1..16 blocks"
    )
    parser.add_argument("--run-dir", default=None)
    parser.add_argument(
        "--output-root",
        default="artifacts/benchmarks/report-benchmarks/ctr-1to16",
    )
    parser.add_argument(
        "--png",
        default="docs/Figures/report-benchmarks/ctr_trace_rounding_1_to_16.png",
    )
    parser.add_argument(
        "--csv",
        default="docs/Figures/report-benchmarks/ctr_trace_rounding_1_to_16.csv",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    output_root = (repo_root / args.output_root).resolve()
    run_dir = (
        (repo_root / args.run_dir).resolve()
        if args.run_dir
        else latest_run_dir(output_root)
    )

    aggregated = read_json(run_dir / "aggregated.json")
    rows = load_ctr_rows(aggregated)
    if not rows:
        raise ValueError("no aes-ctr-<n>blk rows found in aggregated benchmark data")

    write_csv((repo_root / args.csv).resolve(), rows)
    save_plot((repo_root / args.png).resolve(), rows)

    print(f"[ctr-rounding-plot] run_dir={run_dir.relative_to(repo_root)}")
    print(f"[ctr-rounding-plot] wrote {Path(args.png)}")
    print(f"[ctr-rounding-plot] wrote {Path(args.csv)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
