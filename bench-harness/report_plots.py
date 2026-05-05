#!/usr/bin/env python3

import argparse
import csv
import json
import re
import statistics
from pathlib import Path

import matplotlib.pyplot as plt


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def latest_run_dir(output_root: Path) -> Path:
    run_dirs = sorted([p for p in output_root.iterdir() if p.is_dir()])
    if not run_dirs:
        raise FileNotFoundError(f"no benchmark runs in {output_root}")
    return run_dirs[-1]


def bench_median(bench: dict, family: str, metric: str):
    return bench.get(family, {}).get(metric, {}).get("median")


def write_csv(path: Path, rows: list[dict], fieldnames: list[str]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow(row)


def operation_rows(ops_aggregated: dict) -> list[dict]:
    rows = []
    for bench in ops_aggregated.get("benchmarks", []):
        bench_id = bench.get("benchmark_id", "")
        if not bench_id.startswith("operation-bnchmrk-r0:"):
            continue

        prove = bench_median(bench, "timings", "prove_seconds")
        if prove is None:
            continue

        op_name = bench_id.split(":", maxsplit=1)[1]
        rows.append(
            {
                "operation": op_name,
                "prove_seconds_median": float(prove),
                "verify_seconds_median": float(
                    bench_median(bench, "timings", "verify_seconds") or 0.0
                ),
                "user_cycles_median": float(
                    bench_median(bench, "cycles", "user_cycles") or 0.0
                ),
                "total_cycles_median": float(
                    bench_median(bench, "cycles", "total_cycles") or 0.0
                ),
            }
        )

    rows.sort(key=lambda row: row["prove_seconds_median"], reverse=True)
    return rows


def ctr_block_rows(main_aggregated: dict) -> tuple[list[dict], list[dict]]:
    ctr_rows = []
    hmac_rows = []

    ctr_pattern = re.compile(r"^aes-ctr-(\d+)blk$")
    hmac_pattern = re.compile(r"^aes-ctr-hmac-(\d+)blk$")

    for bench in main_aggregated.get("benchmarks", []):
        bench_id = bench.get("benchmark_id", "")
        prove = bench_median(bench, "timings", "prove_seconds")
        verify = bench_median(bench, "timings", "verify_seconds")
        total = bench_median(bench, "timings", "total_seconds")
        user_cycles = bench_median(bench, "cycles", "user_cycles")
        total_cycles = bench_median(bench, "cycles", "total_cycles")

        if total is None or user_cycles is None or total_cycles is None:
            continue

        match_ctr = ctr_pattern.match(bench_id)
        if match_ctr:
            ctr_rows.append(
                {
                    "benchmark_id": bench_id,
                    "blocks": int(match_ctr.group(1)),
                    "prove_seconds_median": float(prove or 0.0),
                    "verify_seconds_median": float(verify or 0.0),
                    "total_seconds_median": float(total),
                    "user_cycles_median": float(user_cycles),
                    "total_cycles_median": float(total_cycles),
                }
            )
            continue

        match_hmac = hmac_pattern.match(bench_id)
        if match_hmac:
            hmac_rows.append(
                {
                    "benchmark_id": bench_id,
                    "blocks": int(match_hmac.group(1)),
                    "prove_seconds_median": float(prove or 0.0),
                    "verify_seconds_median": float(verify or 0.0),
                    "total_seconds_median": float(total),
                    "user_cycles_median": float(user_cycles),
                    "total_cycles_median": float(total_cycles),
                }
            )

    ctr_rows.sort(key=lambda row: row["blocks"])
    hmac_rows.sort(key=lambda row: row["blocks"])
    return ctr_rows, hmac_rows


def proof_size_rows(main_run_dir: Path) -> list[dict]:
    raw_dir = main_run_dir / "raw"
    ctr_pattern = re.compile(r"^aes-ctr-(\d+)blk$")
    hmac_pattern = re.compile(r"^aes-ctr-hmac-(\d+)blk$")

    grouped: dict[str, dict[str, list[float] | int | str]] = {}

    for raw_file in sorted(raw_dir.glob("*.json")):
        record = read_json(raw_file)
        if record.get("status") != "ok":
            continue

        for metric in record.get("metrics", []):
            benchmark_id = metric.get("benchmark_id", "")
            params = metric.get("params", {})
            proof_bytes = params.get("proof_bytes")
            full_receipt_bytes = params.get("full_receipt_bytes")
            if proof_bytes is None or full_receipt_bytes is None:
                continue

            mode = None
            blocks = None

            match_ctr = ctr_pattern.match(benchmark_id)
            if match_ctr:
                mode = "ctr"
                blocks = int(match_ctr.group(1))

            match_hmac = hmac_pattern.match(benchmark_id)
            if match_hmac:
                mode = "ctr_hmac"
                blocks = int(match_hmac.group(1))

            if mode is None or blocks is None:
                continue

            key = f"{mode}:{blocks}"
            item = grouped.setdefault(
                key,
                {
                    "mode": mode,
                    "blocks": blocks,
                    "proof_values": [],
                    "full_receipt_values": [],
                },
            )
            item["proof_values"].append(float(proof_bytes))
            item["full_receipt_values"].append(float(full_receipt_bytes))

    rows = []
    for item in grouped.values():
        proof_values = item["proof_values"]
        full_values = item["full_receipt_values"]
        rows.append(
            {
                "mode": item["mode"],
                "blocks": item["blocks"],
                "proof_bytes_median": statistics.median(proof_values),
                "full_receipt_bytes_median": statistics.median(full_values),
            }
        )

    rows.sort(key=lambda row: (row["mode"], row["blocks"]))
    return rows


def save_operations_plot(rows: list[dict], out_png: Path) -> None:
    labels = [row["operation"] for row in rows]
    values = [row["prove_seconds_median"] for row in rows]

    plt.figure(figsize=(11, 5))
    plt.bar(labels, values, color="#4e79a7")
    plt.xticks(rotation=45, ha="right")
    plt.ylabel("Median Prove Time (s)")
    plt.title("Operation Benchmark Directional Ranking")
    plt.tight_layout()
    plt.savefig(out_png, dpi=180)
    plt.close()


def save_lowmc_vs_aes_plot(main_aggregated: dict, out_png: Path) -> list[dict]:
    bench_map = {
        bench["benchmark_id"]: bench for bench in main_aggregated.get("benchmarks", [])
    }

    lowmc_id = "lowmc-r0" if "lowmc-r0" in bench_map else "lowmc-r0-optimised"
    ids = ["aes-r0", lowmc_id]

    rows = []
    labels = []
    values = []

    label_map = {
        "aes-r0": "aes-r0",
        "lowmc-r0": "lowmc-r0",
        "lowmc-r0-optimised": "lowmc-r0-optimised",
    }

    for bench_id in ids:
        bench = bench_map.get(bench_id)
        if bench is None:
            continue
        prove = bench_median(bench, "timings", "prove_seconds")
        if prove is None:
            continue

        labels.append(label_map[bench_id])
        values.append(float(prove))
        rows.append(
            {
                "benchmark_id": bench_id,
                "prove_seconds_median": float(prove),
                "verify_seconds_median": float(
                    bench_median(bench, "timings", "verify_seconds") or 0.0
                ),
                "total_cycles_median": float(
                    bench_median(bench, "cycles", "total_cycles") or 0.0
                ),
                "user_cycles_median": float(
                    bench_median(bench, "cycles", "user_cycles") or 0.0
                ),
            }
        )

    if not values:
        return rows

    plt.figure(figsize=(8, 5))
    plt.bar(labels, values, color=["#59a14f", "#e15759"])
    plt.ylabel("Median Prove Time (s)")
    plt.title("LowMC vs AES Median Proving Time")
    plt.tight_layout()
    plt.savefig(out_png, dpi=180)
    plt.close()

    return rows


def save_ctr_scaling_plots(
    ctr_rows: list[dict], out_time_png: Path, out_cycles_png: Path, out_combined_png: Path
) -> None:
    blocks = [row["blocks"] for row in ctr_rows]
    time_values = [row["total_seconds_median"] for row in ctr_rows]
    cycle_values = [row["user_cycles_median"] for row in ctr_rows]

    plt.figure(figsize=(8, 5))
    plt.plot(blocks, time_values, marker="o", linewidth=2, color="#4e79a7")
    plt.xlabel("Blocks")
    plt.ylabel("Median Total Time (s)")
    plt.title("AES-CTR Time vs Blocks")
    plt.grid(axis="y", linestyle="--", linewidth=0.6, alpha=0.35)
    plt.tight_layout()
    plt.savefig(out_time_png, dpi=180)
    plt.close()

    plt.figure(figsize=(8, 5))
    plt.plot(blocks, cycle_values, marker="o", linewidth=2, color="#f28e2b")
    plt.xlabel("Blocks")
    plt.ylabel("Median User Cycles")
    plt.title("AES-CTR User Cycles vs Blocks")
    plt.grid(axis="y", linestyle="--", linewidth=0.6, alpha=0.35)
    plt.tight_layout()
    plt.savefig(out_cycles_png, dpi=180)
    plt.close()

    fig, (ax_time, ax_cycles) = plt.subplots(2, 1, figsize=(8, 7), sharex=True)

    ax_time.plot(blocks, time_values, marker="o", linewidth=2, color="#4e79a7")
    ax_time.set_ylabel("Median Total Time (s)")
    ax_time.set_title("AES-CTR Scaling by Block Count")
    ax_time.grid(axis="y", linestyle="--", linewidth=0.6, alpha=0.35)

    ax_cycles.plot(blocks, cycle_values, marker="o", linewidth=2, color="#f28e2b")
    ax_cycles.set_xlabel("Blocks")
    ax_cycles.set_ylabel("Median User Cycles")
    ax_cycles.grid(axis="y", linestyle="--", linewidth=0.6, alpha=0.35)

    fig.tight_layout()
    fig.savefig(out_combined_png, dpi=180)
    plt.close(fig)


def save_auth_overhead_plot(rows: list[dict], out_png: Path) -> None:
    blocks = [row["blocks"] for row in rows]
    time_pct = [row["total_seconds_delta_percent"] for row in rows]
    cycles_pct = [row["user_cycles_delta_percent"] for row in rows]

    x_positions = list(range(len(blocks)))
    width = 0.38

    plt.figure(figsize=(9, 5))
    plt.bar(
        [x - width / 2 for x in x_positions],
        time_pct,
        width=width,
        label="Total time overhead %",
        color="#4e79a7",
    )
    plt.bar(
        [x + width / 2 for x in x_positions],
        cycles_pct,
        width=width,
        label="User cycles overhead %",
        color="#f28e2b",
    )
    plt.xticks(x_positions, [str(block) for block in blocks])
    plt.xlabel("Blocks")
    plt.ylabel("Overhead vs CTR (%)")
    plt.title("CTR+HMAC Overhead vs CTR by Block Size")
    plt.legend()
    plt.grid(axis="y", linestyle="--", linewidth=0.6, alpha=0.35)
    plt.tight_layout()
    plt.savefig(out_png, dpi=180)
    plt.close()


def save_proof_size_plot(rows: list[dict], out_png: Path) -> None:
    ctr_rows = [row for row in rows if row["mode"] == "ctr"]
    hmac_rows = [row for row in rows if row["mode"] == "ctr_hmac"]

    ctr_blocks = [row["blocks"] for row in ctr_rows]
    hmac_blocks = [row["blocks"] for row in hmac_rows]

    fig, (ax_proof, ax_receipt) = plt.subplots(2, 1, figsize=(9, 7), sharex=True)

    ax_proof.plot(
        ctr_blocks,
        [row["proof_bytes_median"] for row in ctr_rows],
        marker="o",
        linewidth=2,
        label="CTR proof bytes",
        color="#4e79a7",
    )
    ax_proof.plot(
        hmac_blocks,
        [row["proof_bytes_median"] for row in hmac_rows],
        marker="o",
        linewidth=2,
        label="CTR+HMAC proof bytes",
        color="#e15759",
    )
    ax_proof.set_ylabel("Median Seal Size (bytes)")
    ax_proof.set_title("Proof Size Reporting by Block Size")
    ax_proof.grid(axis="y", linestyle="--", linewidth=0.6, alpha=0.35)
    ax_proof.legend()

    ax_receipt.plot(
        ctr_blocks,
        [row["full_receipt_bytes_median"] for row in ctr_rows],
        marker="o",
        linewidth=2,
        label="CTR full receipt bytes",
        color="#59a14f",
    )
    ax_receipt.plot(
        hmac_blocks,
        [row["full_receipt_bytes_median"] for row in hmac_rows],
        marker="o",
        linewidth=2,
        label="CTR+HMAC full receipt bytes",
        color="#f28e2b",
    )
    ax_receipt.set_xlabel("Blocks")
    ax_receipt.set_ylabel("Median Full Receipt Size (bytes)")
    ax_receipt.grid(axis="y", linestyle="--", linewidth=0.6, alpha=0.35)
    ax_receipt.legend()

    fig.tight_layout()
    fig.savefig(out_png, dpi=180)
    plt.close(fig)


def auth_overhead_rows(ctr_rows: list[dict], hmac_rows: list[dict]) -> list[dict]:
    ctr_by_blocks = {row["blocks"]: row for row in ctr_rows}
    hmac_by_blocks = {row["blocks"]: row for row in hmac_rows}

    rows = []
    for blocks in sorted(set(ctr_by_blocks) & set(hmac_by_blocks)):
        ctr = ctr_by_blocks[blocks]
        hmac = hmac_by_blocks[blocks]

        total_delta = hmac["total_seconds_median"] - ctr["total_seconds_median"]
        user_cycles_delta = hmac["user_cycles_median"] - ctr["user_cycles_median"]

        rows.append(
            {
                "blocks": blocks,
                "ctr_total_seconds_median": ctr["total_seconds_median"],
                "ctr_hmac_total_seconds_median": hmac["total_seconds_median"],
                "total_seconds_delta": total_delta,
                "total_seconds_delta_percent": (total_delta / ctr["total_seconds_median"])
                * 100.0,
                "ctr_user_cycles_median": ctr["user_cycles_median"],
                "ctr_hmac_user_cycles_median": hmac["user_cycles_median"],
                "user_cycles_delta": user_cycles_delta,
                "user_cycles_delta_percent": (user_cycles_delta / ctr["user_cycles_median"])
                * 100.0,
            }
        )

    return rows


def write_manifest(path: Path, main_run_dir: Path, ops_run_dir: Path) -> None:
    payload = {
        "main_run_dir": str(main_run_dir),
        "ops_run_dir": str(ops_run_dir),
    }
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Build report-ready benchmark plots")
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
        default="artifacts/benchmarks/report-benchmarks/plots",
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

    out_dir = (repo_root / args.output_dir).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    main_aggregated = read_json(main_run_dir / "aggregated.json")
    ops_aggregated = read_json(ops_run_dir / "aggregated.json")

    ops_rows = operation_rows(ops_aggregated)
    write_csv(
        out_dir / "operations_directional.csv",
        ops_rows,
        [
            "operation",
            "prove_seconds_median",
            "verify_seconds_median",
            "user_cycles_median",
            "total_cycles_median",
        ],
    )
    save_operations_plot(ops_rows, out_dir / "operations_directional_prove_time.png")

    lowmc_aes_rows = save_lowmc_vs_aes_plot(
        main_aggregated,
        out_dir / "lowmc_vs_aes_prove_time.png",
    )
    write_csv(
        out_dir / "lowmc_vs_aes_prove_time.csv",
        lowmc_aes_rows,
        [
            "benchmark_id",
            "prove_seconds_median",
            "verify_seconds_median",
            "total_cycles_median",
            "user_cycles_median",
        ],
    )

    ctr_rows, hmac_rows = ctr_block_rows(main_aggregated)
    write_csv(
        out_dir / "ctr_scaling.csv",
        ctr_rows,
        [
            "benchmark_id",
            "blocks",
            "prove_seconds_median",
            "verify_seconds_median",
            "total_seconds_median",
            "user_cycles_median",
            "total_cycles_median",
        ],
    )
    save_ctr_scaling_plots(
        ctr_rows,
        out_dir / "ctr_time_vs_blocks.png",
        out_dir / "ctr_cycles_vs_blocks.png",
        out_dir / "ctr_time_and_cycles_vs_blocks.png",
    )

    overhead_rows = auth_overhead_rows(ctr_rows, hmac_rows)
    write_csv(
        out_dir / "auth_overhead_ctr_vs_ctr_hmac.csv",
        overhead_rows,
        [
            "blocks",
            "ctr_total_seconds_median",
            "ctr_hmac_total_seconds_median",
            "total_seconds_delta",
            "total_seconds_delta_percent",
            "ctr_user_cycles_median",
            "ctr_hmac_user_cycles_median",
            "user_cycles_delta",
            "user_cycles_delta_percent",
        ],
    )
    save_auth_overhead_plot(
        overhead_rows,
        out_dir / "auth_overhead_ctr_vs_ctr_hmac.png",
    )

    size_rows = proof_size_rows(main_run_dir)
    write_csv(
        out_dir / "proof_size_ctr_vs_ctr_hmac.csv",
        size_rows,
        [
            "mode",
            "blocks",
            "proof_bytes_median",
            "full_receipt_bytes_median",
        ],
    )
    save_proof_size_plot(
        size_rows,
        out_dir / "proof_size_ctr_vs_ctr_hmac.png",
    )

    write_manifest(out_dir / "plot_inputs.json", main_run_dir, ops_run_dir)

    print(f"[report-plots] wrote report plots to {out_dir.relative_to(repo_root)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
