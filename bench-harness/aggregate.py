#!/usr/bin/env python3

import argparse
import json
import math
import statistics
from datetime import datetime, timezone
from pathlib import Path


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def latest_run_dir(output_root: Path) -> Path:
    run_dirs = sorted([p for p in output_root.iterdir() if p.is_dir()])
    if not run_dirs:
        raise FileNotFoundError(f"no benchmark runs in {output_root}")
    return run_dirs[-1]


T_CRITICAL_975 = {
    1: 12.706,
    2: 4.303,
    3: 3.182,
    4: 2.776,
    5: 2.571,
    6: 2.447,
    7: 2.365,
    8: 2.306,
    9: 2.262,
    10: 2.228,
    11: 2.201,
    12: 2.179,
    13: 2.160,
    14: 2.145,
    15: 2.131,
    16: 2.120,
    17: 2.110,
    18: 2.101,
    19: 2.093,
    20: 2.086,
    21: 2.080,
    22: 2.074,
    23: 2.069,
    24: 2.064,
    25: 2.060,
    26: 2.056,
    27: 2.052,
    28: 2.048,
    29: 2.045,
    30: 2.042,
}


def t_critical_975(sample_count: int) -> float:
    if sample_count <= 1:
        return 0.0
    return T_CRITICAL_975.get(sample_count - 1, 1.96)


def percentile(sorted_values: list[float], fraction: float) -> float:
    if len(sorted_values) == 1:
        return sorted_values[0]

    position = fraction * (len(sorted_values) - 1)
    lower = math.floor(position)
    upper = math.ceil(position)
    if lower == upper:
        return sorted_values[lower]

    weight = position - lower
    return sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight


def stat_block(values: list[float]) -> dict:
    if not values:
        return {}
    sorted_vals = sorted(values)
    count = len(values)
    mean = statistics.fmean(values)
    stddev = statistics.stdev(values) if count > 1 else 0.0
    variance = statistics.variance(values) if count > 1 else 0.0
    sem = stddev / math.sqrt(count) if count > 1 else 0.0
    ci95_margin = t_critical_975(count) * sem
    q1 = percentile(sorted_vals, 0.25)
    q3 = percentile(sorted_vals, 0.75)
    idx_95 = min(len(sorted_vals) - 1, math.ceil(0.95 * len(sorted_vals)) - 1)
    return {
        "count": count,
        "mean": mean,
        "median": statistics.median(values),
        "p95": sorted_vals[idx_95],
        "stddev": stddev,
        "variance": variance,
        "sem": sem,
        "ci95_lower": mean - ci95_margin,
        "ci95_upper": mean + ci95_margin,
        "ci95_margin": ci95_margin,
        "q1": q1,
        "q3": q3,
        "iqr": q3 - q1,
        "cv_percent": (stddev / mean * 100.0) if mean else None,
        "min": min(values),
        "max": max(values),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Aggregate benchmark raw JSON")
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
    raw_dir = run_dir / "raw"

    entries = {}

    for raw_file in sorted(raw_dir.glob("*.json")):
        record = read_json(raw_file)
        target_id = record["target_id"]
        status = record.get("status", "error")

        if status != "ok":
            item = entries.setdefault(
                target_id,
                {
                    "benchmark_id": target_id,
                    "algorithm": target_id,
                    "mode": "zk",
                    "params": {},
                    "success_count": 0,
                    "timeout_count": 0,
                    "error_count": 0,
                    "parse_error_count": 0,
                    "prove_seconds": [],
                    "verify_seconds": [],
                    "total_seconds": [],
                    "total_cycles": [],
                    "user_cycles": [],
                    "paging_cycles": [],
                    "reserved_cycles": [],
                },
            )
            if status == "timeout":
                item["timeout_count"] += 1
            elif status == "parse_error":
                item["parse_error_count"] += 1
            else:
                item["error_count"] += 1
            continue

        for metric in record.get("metrics", []):
            bench_id = metric["benchmark_id"]
            item = entries.setdefault(
                bench_id,
                {
                    "benchmark_id": bench_id,
                    "algorithm": metric.get("algorithm", bench_id),
                    "mode": metric.get("mode", "zk"),
                    "params": metric.get("params", {}),
                    "success_count": 0,
                    "timeout_count": 0,
                    "error_count": 0,
                    "parse_error_count": 0,
                    "prove_seconds": [],
                    "verify_seconds": [],
                    "total_seconds": [],
                    "total_cycles": [],
                    "user_cycles": [],
                    "paging_cycles": [],
                    "reserved_cycles": [],
                },
            )
            item["success_count"] += 1

            timings = metric.get("timings", {})
            cycles = metric.get("cycles", {})

            prove = timings.get("prove_seconds")
            verify = timings.get("verify_seconds")
            total = timings.get("total_seconds")
            if prove is not None:
                item["prove_seconds"].append(float(prove))
            if verify is not None:
                item["verify_seconds"].append(float(verify))
            if total is not None:
                item["total_seconds"].append(float(total))

            for key in (
                "total_cycles",
                "user_cycles",
                "paging_cycles",
                "reserved_cycles",
            ):
                val = cycles.get(key)
                if val is not None:
                    item[key].append(float(val))

    benchmarks = []
    for item in entries.values():
        attempted = (
            item["success_count"]
            + item["timeout_count"]
            + item["error_count"]
            + item["parse_error_count"]
        )
        benchmarks.append(
            {
                "benchmark_id": item["benchmark_id"],
                "algorithm": item["algorithm"],
                "mode": item["mode"],
                "params": item["params"],
                "success_count": item["success_count"],
                "timeout_count": item["timeout_count"],
                "error_count": item["error_count"],
                "parse_error_count": item["parse_error_count"],
                "attempted": attempted,
                "success_rate": (item["success_count"] / attempted)
                if attempted
                else 0.0,
                "timings": {
                    "prove_seconds": stat_block(item["prove_seconds"]),
                    "verify_seconds": stat_block(item["verify_seconds"]),
                    "total_seconds": stat_block(item["total_seconds"]),
                },
                "cycles": {
                    "total_cycles": stat_block(item["total_cycles"]),
                    "user_cycles": stat_block(item["user_cycles"]),
                    "paging_cycles": stat_block(item["paging_cycles"]),
                    "reserved_cycles": stat_block(item["reserved_cycles"]),
                },
            }
        )

    benchmarks.sort(key=lambda b: b["benchmark_id"])

    out = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "run_dir": str(run_dir.relative_to(repo_root)),
        "benchmark_count": len(benchmarks),
        "benchmarks": benchmarks,
    }

    output_path = run_dir / "aggregated.json"
    output_path.write_text(json.dumps(out, indent=2), encoding="utf-8")

    print(f"[aggregate] wrote {output_path.relative_to(repo_root)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
