#!/usr/bin/env python3

import argparse
from collections import defaultdict
import json
from pathlib import Path

import matplotlib.pyplot as plt


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def latest_run_dir(output_root: Path) -> Path:
    run_dirs = sorted([p for p in output_root.iterdir() if p.is_dir()])
    if not run_dirs:
        raise FileNotFoundError(f"no benchmark runs in {output_root}")
    return run_dirs[-1]


def bar_plot(
    labels, values, title, ylabel, out_png: Path, out_svg: Path, log_scale=False
):
    plt.figure(figsize=(11, 5))
    plt.bar(labels, values)
    plt.xticks(rotation=45, ha="right")
    plt.title(title)
    plt.ylabel(ylabel)
    if log_scale:
        plt.yscale("log")
    plt.tight_layout()
    plt.savefig(out_png, dpi=180)
    plt.savefig(out_svg)
    plt.close()


def bar_with_runs_plot(
    labels,
    medians,
    run_values,
    title,
    ylabel,
    out_png: Path,
    out_svg: Path,
    log_scale=False,
):
    plt.figure(figsize=(11, 5))
    x_positions = list(range(len(labels)))
    plt.bar(x_positions, medians, label="Median", color="#9ecae1")

    has_runs = False
    run_label_added = False
    for idx, values in enumerate(run_values):
        if not values:
            continue
        has_runs = True
        if len(values) == 1:
            xs = [idx]
        else:
            spread = 0.28
            step = (2 * spread) / (len(values) - 1)
            xs = [idx - spread + (i * step) for i in range(len(values))]

        plt.scatter(
            xs,
            values,
            color="#1f4e79",
            edgecolors="white",
            linewidths=0.5,
            zorder=3,
            label="Runs" if not run_label_added else None,
        )
        run_label_added = True

    plt.xticks(x_positions, labels, rotation=45, ha="right")
    plt.title(title)
    plt.ylabel(ylabel)
    if log_scale:
        plt.yscale("log")
    if has_runs:
        plt.legend()
    plt.tight_layout()
    plt.savefig(out_png, dpi=180)
    plt.savefig(out_svg)
    plt.close()


def collect_timing_runs(raw_dir: Path):
    timing_runs = defaultdict(
        lambda: {"prove_seconds": [], "verify_seconds": [], "total_seconds": []}
    )
    if not raw_dir.exists():
        return timing_runs

    for raw_file in sorted(raw_dir.glob("*.json")):
        record = read_json(raw_file)
        if record.get("status") != "ok":
            continue

        for metric in record.get("metrics", []):
            if metric.get("status", "ok") != "ok":
                continue

            benchmark_id = metric.get("benchmark_id")
            if not benchmark_id:
                continue

            timings = metric.get("timings", {})
            for key in ("prove_seconds", "verify_seconds", "total_seconds"):
                value = timings.get(key)
                if value is not None:
                    timing_runs[benchmark_id][key].append(float(value))

    return timing_runs


def main() -> int:
    parser = argparse.ArgumentParser(description="Plot benchmark aggregate JSON")
    parser.add_argument("--run-dir", default=None)
    parser.add_argument("--output-root", default="artifacts/benchmarks")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    output_root = (repo_root / args.output_root).resolve()
    run_dir = (
        (repo_root / args.run_dir).resolve()
        if args.run_dir
        else latest_run_dir(output_root)
    )

    aggregate_path = run_dir / "aggregated.json"
    data = read_json(aggregate_path)
    plots_dir = run_dir / "plots"
    plots_dir.mkdir(parents=True, exist_ok=True)

    benchmarks = data.get("benchmarks", [])

    prove_rows = []
    verify_rows = []
    total_rows = []
    cycles_rows = []
    success_rows = []

    timing_runs = collect_timing_runs(run_dir / "raw")

    for b in benchmarks:
        benchmark_id = b["benchmark_id"]
        label = b.get("algorithm", benchmark_id)
        success_rows.append((benchmark_id, label, b.get("success_rate", 0.0) * 100.0))

        prove = b.get("timings", {}).get("prove_seconds", {}).get("median")
        if prove is not None:
            prove_rows.append((benchmark_id, label, prove))

        verify = b.get("timings", {}).get("verify_seconds", {}).get("median")
        if verify is not None:
            verify_rows.append((benchmark_id, label, verify))

        total = b.get("timings", {}).get("total_seconds", {}).get("median")
        if total is not None:
            total_rows.append((benchmark_id, label, total))

        total_cycles = b.get("cycles", {}).get("total_cycles", {}).get("median")
        if total_cycles is not None:
            cycles_rows.append((benchmark_id, label, total_cycles))

    prove_rows.sort(key=lambda x: x[2], reverse=True)
    verify_rows.sort(key=lambda x: x[2], reverse=True)
    total_rows.sort(key=lambda x: x[2], reverse=True)
    cycles_rows.sort(key=lambda x: x[2], reverse=True)
    success_rows.sort(key=lambda x: x[2])

    if prove_rows:
        bar_plot(
            [r[1] for r in prove_rows],
            [r[2] for r in prove_rows],
            "Median Prove Time by Benchmark",
            "Seconds",
            plots_dir / "prove_time_by_algorithm.png",
            plots_dir / "prove_time_by_algorithm.svg",
        )

    if verify_rows:
        bar_plot(
            [r[1] for r in verify_rows],
            [r[2] for r in verify_rows],
            "Median Verify Time by Benchmark",
            "Seconds",
            plots_dir / "verify_time_by_algorithm.png",
            plots_dir / "verify_time_by_algorithm.svg",
        )

    if total_rows:
        bar_plot(
            [r[1] for r in total_rows],
            [r[2] for r in total_rows],
            "Median Total Time by Benchmark",
            "Seconds",
            plots_dir / "total_time_by_algorithm.png",
            plots_dir / "total_time_by_algorithm.svg",
        )

        bar_with_runs_plot(
            [r[1] for r in total_rows],
            [r[2] for r in total_rows],
            [timing_runs.get(r[0], {}).get("total_seconds", []) for r in total_rows],
            "Total Time by Benchmark (All Runs + Median)",
            "Seconds",
            plots_dir / "total_time_all_runs_by_algorithm.png",
            plots_dir / "total_time_all_runs_by_algorithm.svg",
        )

    if cycles_rows:
        bar_plot(
            [r[1] for r in cycles_rows],
            [r[2] for r in cycles_rows],
            "Median Total Cycles by Benchmark",
            "Cycles",
            plots_dir / "total_cycles_by_algorithm.png",
            plots_dir / "total_cycles_by_algorithm.svg",
            log_scale=True,
        )

    if success_rows:
        bar_plot(
            [r[1] for r in success_rows],
            [r[2] for r in success_rows],
            "Success Rate by Benchmark",
            "Success Rate (%)",
            plots_dir / "success_rate_by_algorithm.png",
            plots_dir / "success_rate_by_algorithm.svg",
        )

    print(f"[plot] wrote plots to {plots_dir.relative_to(repo_root)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
