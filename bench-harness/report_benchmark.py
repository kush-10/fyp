#!/usr/bin/env python3

import argparse
import json
import math
import os
import statistics
from datetime import datetime, timezone
from pathlib import Path

try:
    from scipy import stats

    SCIPY_AVAILABLE = True
except Exception:  # pragma: no cover - optional dependency fallback
    stats = None
    SCIPY_AVAILABLE = False


MAIN_ORDER = [
    "aes-r0",
    "lowmc-r0-optimised",
    "aes-ctr-1blk",
    "aes-ctr-hmac-1blk",
    "aes-ctr-4blk",
    "aes-ctr-hmac-4blk",
    "aes-ctr-16blk",
    "aes-ctr-hmac-16blk",
    "aes-ctr-64blk",
    "aes-ctr-hmac-64blk",
]


def color_enabled() -> bool:
    return "NO_COLOR" not in os.environ


def c(text: str, code: str) -> str:
    if not color_enabled():
        return text
    return f"\033[{code}m{text}\033[0m"


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2), encoding="utf-8")


def latest_run_dir(output_root: Path) -> Path:
    run_dirs = sorted([p for p in output_root.iterdir() if p.is_dir()])
    if not run_dirs:
        raise FileNotFoundError(f"no benchmark runs in {output_root}")
    return run_dirs[-1]


def stat_block(summary: dict, family: str, metric: str) -> dict:
    return summary.get(family, {}).get(metric, {})


def stat_value(summary: dict, family: str, metric: str, name: str):
    return stat_block(summary, family, metric).get(name)


def fmt_seconds(value) -> str:
    if value is None:
        return "-"
    return f"{float(value):.3f}"


def fmt_float(value, digits: int = 3) -> str:
    if value is None:
        return "-"
    if isinstance(value, float) and not math.isfinite(value):
        return "-"
    return f"{float(value):.{digits}f}"


def fmt_int(value) -> str:
    if value is None:
        return "-"
    return f"{int(round(float(value))):,}"


def fmt_p(value) -> str:
    if value is None:
        return "-"
    if value < 0.001:
        return "<0.001"
    return f"{value:.3f}"


def fmt_ci(block: dict) -> str:
    if not block:
        return "-"
    return f"{fmt_seconds(block.get('ci95_lower'))}..{fmt_seconds(block.get('ci95_upper'))}"


def fmt_proof_bytes(params: dict) -> str:
    proof = params.get("proof_bytes")
    if proof is None:
        return "-"
    return f"{proof / 1024.0:.1f} KiB"


def ns_per_trace_cycle(summary: dict):
    total_seconds = stat_value(summary, "timings", "total_seconds", "median")
    total_cycles = stat_value(summary, "cycles", "total_cycles", "median")
    if total_seconds is None or not total_cycles:
        return None
    return float(total_seconds) * 1_000_000_000.0 / float(total_cycles)


def table(title: str, headers: list[str], rows: list[list[str]]) -> None:
    if not rows:
        return

    widths = [len(header) for header in headers]
    for row in rows:
        for i, cell in enumerate(row):
            widths[i] = max(widths[i], len(cell))

    border = "+" + "+".join("-" * (width + 2) for width in widths) + "+"
    print()
    print(c(title, "1;36"))
    print(c(border, "36"))
    print(
        c(
            "| "
            + " | ".join(header.ljust(widths[i]) for i, header in enumerate(headers))
            + " |",
            "1;36",
        )
    )
    print(c(border, "36"))
    for row in rows:
        print("| " + " | ".join(cell.ljust(widths[i]) for i, cell in enumerate(row)) + " |")
    print(c(border, "36"))


def benchmark_map(aggregated: dict) -> dict:
    return {bench["benchmark_id"]: bench for bench in aggregated.get("benchmarks", [])}


def ordered_main_rows(main_aggregated: dict) -> list[dict]:
    benches = benchmark_map(main_aggregated)
    ordered = [benches[bench_id] for bench_id in MAIN_ORDER if bench_id in benches]
    seen = {bench["benchmark_id"] for bench in ordered}
    ordered.extend(
        bench
        for bench in main_aggregated.get("benchmarks", [])
        if bench["benchmark_id"] not in seen
    )
    return ordered


def summary_rows(aggregated: dict, ordered_rows: list[dict]) -> list[list[str]]:
    rows = []
    for bench in ordered_rows:
        prove = stat_block(bench, "timings", "prove_seconds")
        total_cycles = stat_value(bench, "cycles", "total_cycles", "median")
        user_cycles = stat_value(bench, "cycles", "user_cycles", "median")
        rows.append(
            [
                bench["benchmark_id"],
                str(bench.get("success_count", 0)),
                fmt_seconds(prove.get("mean")),
                fmt_seconds(prove.get("median")),
                fmt_seconds(prove.get("stddev")),
                fmt_ci(prove),
                fmt_int(total_cycles),
                fmt_int(user_cycles),
                fmt_float(ns_per_trace_cycle(bench), 1),
                fmt_proof_bytes(bench.get("params", {})),
            ]
        )
    return rows


def collect_raw_values(run_dir: Path) -> dict:
    values: dict[str, dict[str, dict[str, list[float]]]] = {}
    raw_dir = run_dir / "raw"
    for raw_file in sorted(raw_dir.glob("*.json")):
        record = read_json(raw_file)
        if record.get("status") != "ok":
            continue
        for metric in record.get("metrics", []):
            if metric.get("status", "ok") != "ok":
                continue
            bench_id = metric.get("benchmark_id")
            if not bench_id:
                continue
            item = values.setdefault(bench_id, {"timings": {}, "cycles": {}})
            for family in ("timings", "cycles"):
                for key, value in metric.get(family, {}).items():
                    if value is not None:
                        item[family].setdefault(key, []).append(float(value))
    return values


def mean(values: list[float]):
    return statistics.fmean(values) if values else None


def median(values: list[float]):
    return statistics.median(values) if values else None


def welch_p(left: list[float], right: list[float]):
    if not SCIPY_AVAILABLE or len(left) < 2 or len(right) < 2:
        return None
    result = stats.ttest_ind(left, right, equal_var=False, alternative="two-sided")
    return float(result.pvalue) if math.isfinite(float(result.pvalue)) else None


def mann_whitney_p(left: list[float], right: list[float]):
    if not SCIPY_AVAILABLE or len(left) < 2 or len(right) < 2:
        return None
    result = stats.mannwhitneyu(left, right, alternative="two-sided", method="auto")
    return float(result.pvalue) if math.isfinite(float(result.pvalue)) else None


def cohens_d(left: list[float], right: list[float]):
    if len(left) < 2 or len(right) < 2:
        return None
    left_var = statistics.variance(left)
    right_var = statistics.variance(right)
    pooled = ((len(left) - 1) * left_var + (len(right) - 1) * right_var) / (
        len(left) + len(right) - 2
    )
    if pooled <= 0:
        return None
    return (statistics.fmean(right) - statistics.fmean(left)) / math.sqrt(pooled)


def holm_adjust(p_values: list) -> list:
    indexed = [(i, p) for i, p in enumerate(p_values) if p is not None]
    adjusted = [None for _ in p_values]
    previous = 0.0
    total = len(indexed)

    for rank, (idx, p_value) in enumerate(sorted(indexed, key=lambda item: item[1])):
        corrected = min((total - rank) * p_value, 1.0)
        corrected = max(corrected, previous)
        adjusted[idx] = corrected
        previous = corrected

    return adjusted


def significance_label(p_value) -> str:
    if p_value is None:
        return "n/a"
    if p_value < 0.001:
        return "***"
    if p_value < 0.01:
        return "**"
    if p_value < 0.05:
        return "*"
    return "ns"


def build_comparisons(raw_values: dict) -> list[dict]:
    specs = [("aes-r0", "lowmc-r0-optimised", "timings", "prove_seconds")]
    for blocks in (1, 4, 16, 64):
        specs.append(
            (
                f"aes-ctr-{blocks}blk",
                f"aes-ctr-hmac-{blocks}blk",
                "timings",
                "prove_seconds",
            )
        )
        specs.append(
            (
                f"aes-ctr-{blocks}blk",
                f"aes-ctr-hmac-{blocks}blk",
                "timings",
                "total_seconds",
            )
        )

    comparisons = []
    for left_id, right_id, family, metric in specs:
        left = raw_values.get(left_id, {}).get(family, {}).get(metric, [])
        right = raw_values.get(right_id, {}).get(family, {}).get(metric, [])
        if not left or not right:
            continue

        left_mean = mean(left)
        right_mean = mean(right)
        left_median = median(left)
        right_median = median(right)
        comparisons.append(
            {
                "left_id": left_id,
                "right_id": right_id,
                "metric": metric,
                "left_n": len(left),
                "right_n": len(right),
                "left_mean": left_mean,
                "right_mean": right_mean,
                "mean_delta_right_minus_left": right_mean - left_mean,
                "median_ratio_right_over_left": (right_median / left_median)
                if left_median
                else None,
                "welch_t_p_two_sided": welch_p(left, right),
                "mann_whitney_p_two_sided": mann_whitney_p(left, right),
                "cohens_d_right_minus_left": cohens_d(left, right),
            }
        )

    adjusted = holm_adjust([row["welch_t_p_two_sided"] for row in comparisons])
    for row, p_adjusted in zip(comparisons, adjusted):
        row["welch_t_p_holm"] = p_adjusted
        row["significance"] = significance_label(p_adjusted)

    return comparisons


def comparison_rows(comparisons: list[dict]) -> list[list[str]]:
    rows = []
    for item in comparisons:
        rows.append(
            [
                f"{item['right_id']} vs {item['left_id']}",
                item["metric"],
                f"{item['left_n']}/{item['right_n']}",
                fmt_seconds(item["mean_delta_right_minus_left"]),
                fmt_float(item["median_ratio_right_over_left"], 3),
                fmt_p(item["welch_t_p_two_sided"]),
                fmt_p(item["welch_t_p_holm"]),
                fmt_p(item["mann_whitney_p_two_sided"]),
                fmt_float(item["cohens_d_right_minus_left"], 2),
                item["significance"],
            ]
        )
    return rows


def operation_rows(ops_aggregated: dict) -> list[dict]:
    rows = []
    for bench in ops_aggregated.get("benchmarks", []):
        if bench.get("benchmark_id", "").startswith("operation-bnchmrk-r0:"):
            rows.append(bench)
    rows.sort(
        key=lambda bench: stat_value(bench, "timings", "prove_seconds", "median") or 0.0,
        reverse=True,
    )
    return rows


def print_provenance(main_run_dir: Path, ops_run_dir: Path, repo_root: Path) -> None:
    manifest = read_json(main_run_dir / "run_manifest.json")
    lock = manifest.get("benchmark_lock", {})
    git = lock.get("git", {})
    dirty = "dirty" if git.get("dirty") else "clean"
    commit = git.get("short_commit") or "unknown"
    branch = git.get("branch") or "unknown"
    print(c("Report Benchmark Campaign", "1;35"))
    print(f"Commit lock : {c(commit, '1;32')} on {branch} ({dirty})")
    print(f"Main run    : {main_run_dir.relative_to(repo_root)}")
    print(f"Ops run     : {ops_run_dir.relative_to(repo_root)}")
    print(f"Stats tests : {'SciPy enabled' if SCIPY_AVAILABLE else 'SciPy unavailable'}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Print report benchmark tables and statistical comparisons"
    )
    parser.add_argument("--main-run-dir", default=None)
    parser.add_argument("--ops-run-dir", default=None)
    parser.add_argument(
        "--main-output-root",
        default="artifacts/benchmarks/report-benchmarks/main",
    )
    parser.add_argument(
        "--ops-output-root",
        default="artifacts/benchmarks/report-benchmarks/ops",
    )
    parser.add_argument(
        "--output-dir",
        default="artifacts/benchmarks/report-benchmarks/analysis",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    main_run_dir = (
        (repo_root / args.main_run_dir).resolve()
        if args.main_run_dir
        else latest_run_dir((repo_root / args.main_output_root).resolve())
    )
    ops_run_dir = (
        (repo_root / args.ops_run_dir).resolve()
        if args.ops_run_dir
        else latest_run_dir((repo_root / args.ops_output_root).resolve())
    )

    main_aggregated = read_json(main_run_dir / "aggregated.json")
    ops_aggregated = read_json(ops_run_dir / "aggregated.json")
    raw_values = collect_raw_values(main_run_dir)
    comparisons = build_comparisons(raw_values)

    print_provenance(main_run_dir, ops_run_dir, repo_root)
    table(
        "Main Benchmark Summary",
        [
            "Benchmark",
            "n",
            "Mean Prove s",
            "Median s",
            "Stddev s",
            "95% CI s",
            "Trace Cycles",
            "User Cycles",
            "ns/Trace Cyc",
            "Proof",
        ],
        summary_rows(main_aggregated, ordered_main_rows(main_aggregated)),
    )
    table(
        "Operation Benchmark Summary",
        [
            "Benchmark",
            "n",
            "Mean Prove s",
            "Median s",
            "Stddev s",
            "95% CI s",
            "Trace Cycles",
            "User Cycles",
            "ns/Trace Cyc",
            "Proof",
        ],
        summary_rows(ops_aggregated, operation_rows(ops_aggregated)),
    )
    table(
        "Statistical Comparisons",
        [
            "Comparison",
            "Metric",
            "n",
            "Mean Delta s",
            "Median Ratio",
            "Welch p",
            "Holm p",
            "MWU p",
            "Cohen d",
            "Sig",
        ],
        comparison_rows(comparisons),
    )

    analysis = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "main_run_dir": str(main_run_dir.relative_to(repo_root)),
        "ops_run_dir": str(ops_run_dir.relative_to(repo_root)),
        "scipy_available": SCIPY_AVAILABLE,
        "notes": {
            "tests": "Welch two-sided t-test plus Mann-Whitney U; Welch p-values are Holm corrected across listed comparisons.",
            "direction": "Mean deltas and ratios are right target minus/over left target.",
        },
        "comparisons": comparisons,
    }
    out_dir = (repo_root / args.output_dir).resolve()
    output_path = out_dir / "latest_report_analysis.json"
    write_json(output_path, analysis)
    write_json(main_run_dir / "statistical_analysis.json", analysis)
    print()
    print(c(f"[report-benchmark] wrote {output_path.relative_to(repo_root)}", "1;32"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
