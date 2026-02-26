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


def stat_block(values: list[float]) -> dict:
    if not values:
        return {}
    sorted_vals = sorted(values)
    idx_95 = min(len(sorted_vals) - 1, math.ceil(0.95 * len(sorted_vals)) - 1)
    return {
        "count": len(values),
        "mean": statistics.fmean(values),
        "median": statistics.median(values),
        "p95": sorted_vals[idx_95],
        "stddev": statistics.stdev(values) if len(values) > 1 else 0.0,
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
