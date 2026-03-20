#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class Target:
    id: str
    workdir: str
    command: list[str]


TARGETS: list[Target] = [
    Target(
        id="lowmc-r0",
        workdir="lowmc-r0",
        command=["cargo", "run", "-p", "host", "--release", "--", "--json"],
    ),
    Target(
        id="lowmc-r0-optimised",
        workdir="lowmc-r0-optimised",
        command=["cargo", "run", "-p", "host", "--release", "--", "--json"],
    ),
]


def utc_now_compact() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%SZ")


def try_parse_json(stdout: str) -> dict[str, Any] | None:
    text = stdout.strip()
    if not text:
        return None

    try:
        payload = json.loads(text)
        if isinstance(payload, dict):
            return payload
    except json.JSONDecodeError:
        pass

    for line in reversed(text.splitlines()):
        line = line.strip()
        if not line:
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(payload, dict):
            return payload
    return None


def require_number(value: Any, name: str, target_id: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise RuntimeError(
            f"{target_id}: expected numeric field '{name}', got {value!r}"
        )
    return float(value)


def run_command(
    command: list[str],
    cwd: Path,
    env: dict[str, str],
    timeout_sec: int,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        timeout=timeout_sec,
        check=False,
    )


def parse_pprof_top(pprof_text: str, limit: int) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line in pprof_text.splitlines():
        tokens = line.split()
        if len(tokens) < 6:
            continue
        if not (
            tokens[1].endswith("%")
            and tokens[2].endswith("%")
            and tokens[4].endswith("%")
        ):
            continue

        try:
            flat_percent = float(tokens[1][:-1])
            sum_percent = float(tokens[2][:-1])
            cum_percent = float(tokens[4][:-1])
        except ValueError:
            continue

        function_name = " ".join(tokens[5:]).strip()
        if not function_name:
            continue

        rows.append(
            {
                "function": function_name,
                "flat_percent": flat_percent,
                "sum_percent": sum_percent,
                "cum_percent": cum_percent,
            }
        )

    if not rows:
        raise RuntimeError("failed to parse any function rows from pprof output")

    return rows[:limit]


def estimate_rows(
    rows: list[dict[str, Any]],
    user_cycles: int,
    prove_seconds: float,
) -> list[dict[str, Any]]:
    estimated: list[dict[str, Any]] = []
    for idx, row in enumerate(rows, start=1):
        flat_share = row["flat_percent"] / 100.0
        cum_share = row["cum_percent"] / 100.0
        estimated.append(
            {
                **row,
                "rank": idx,
                "est_flat_user_cycles": int(round(user_cycles * flat_share)),
                "est_cum_user_cycles": int(round(user_cycles * cum_share)),
                "est_flat_prove_seconds": prove_seconds * flat_share,
                "est_cum_prove_seconds": prove_seconds * cum_share,
            }
        )
    return estimated


def run_target(
    repo_root: Path,
    out_dir: Path,
    target: Target,
    top_n: int,
    timeout_sec: int,
) -> dict[str, Any]:
    logs_dir = out_dir / "logs"
    pprof_dir = out_dir / "pprof"
    profiles_dir = out_dir / "profiles"
    raw_dir = out_dir / "raw"
    logs_dir.mkdir(parents=True, exist_ok=True)
    pprof_dir.mkdir(parents=True, exist_ok=True)
    profiles_dir.mkdir(parents=True, exist_ok=True)
    raw_dir.mkdir(parents=True, exist_ok=True)

    stdout_path = logs_dir / f"{target.id}.stdout.log"
    stderr_path = logs_dir / f"{target.id}.stderr.log"
    profile_path = profiles_dir / f"{target.id}.pprof.pb.gz"
    host_json_path = raw_dir / f"{target.id}.host.json"
    pprof_flat_path = pprof_dir / f"{target.id}.top.flat.txt"
    pprof_cum_path = pprof_dir / f"{target.id}.top.cum.txt"

    env = os.environ.copy()
    env.pop("NO_RISC0", None)
    env["RISC0_PPROF_OUT"] = str(profile_path)
    env.setdefault("RISC0_PPROF_ENABLE_INLINE_FUNCTIONS", "1")

    try:
        completed = run_command(
            command=target.command,
            cwd=repo_root / target.workdir,
            env=env,
            timeout_sec=timeout_sec,
        )
    except subprocess.TimeoutExpired as exc:
        timeout_stdout = exc.stdout or ""
        timeout_stderr = exc.stderr or ""
        if isinstance(timeout_stdout, bytes):
            timeout_stdout = timeout_stdout.decode("utf-8", errors="replace")
        if isinstance(timeout_stderr, bytes):
            timeout_stderr = timeout_stderr.decode("utf-8", errors="replace")
        stdout_path.write_text(timeout_stdout, encoding="utf-8")
        stderr_path.write_text(timeout_stderr, encoding="utf-8")
        raise RuntimeError(
            f"{target.id}: command timed out after {timeout_sec}s; see {stdout_path} and {stderr_path}"
        )

    stdout_path.write_text(completed.stdout, encoding="utf-8")
    stderr_path.write_text(completed.stderr, encoding="utf-8")

    if completed.returncode != 0:
        raise RuntimeError(
            f"{target.id}: command failed with code {completed.returncode}; "
            f"see {stdout_path} and {stderr_path}"
        )

    payload = try_parse_json(completed.stdout)
    if payload is None:
        raise RuntimeError(
            f"{target.id}: command succeeded but JSON output was not found; see {stdout_path}"
        )

    host_json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    if payload.get("mode") != "zk":
        raise RuntimeError(
            f"{target.id}: expected zk mode JSON, got mode={payload.get('mode')!r}"
        )

    timings = payload.get("timings", {})
    cycles = payload.get("cycles", {})
    if not isinstance(timings, dict) or not isinstance(cycles, dict):
        raise RuntimeError(f"{target.id}: malformed JSON payload")

    prove_seconds = require_number(
        timings.get("prove_seconds"), "timings.prove_seconds", target.id
    )
    verify_seconds = require_number(
        timings.get("verify_seconds"), "timings.verify_seconds", target.id
    )
    total_seconds = require_number(
        timings.get("total_seconds"), "timings.total_seconds", target.id
    )

    user_cycles = int(
        require_number(cycles.get("user_cycles"), "cycles.user_cycles", target.id)
    )
    total_cycles = int(
        require_number(cycles.get("total_cycles"), "cycles.total_cycles", target.id)
    )
    paging_cycles = int(
        require_number(cycles.get("paging_cycles"), "cycles.paging_cycles", target.id)
    )
    reserved_cycles = int(
        require_number(
            cycles.get("reserved_cycles"), "cycles.reserved_cycles", target.id
        )
    )

    if not profile_path.exists():
        raise RuntimeError(
            f"{target.id}: expected profile at {profile_path}, but it was not created"
        )

    pprof_flat_cmd = [
        "go",
        "tool",
        "pprof",
        "-top",
        f"-nodecount={max(100, top_n)}",
        str(profile_path),
    ]
    pprof_cum_cmd = [
        "go",
        "tool",
        "pprof",
        "-top",
        "-cum",
        f"-nodecount={max(100, top_n)}",
        str(profile_path),
    ]

    pprof_flat = run_command(
        command=pprof_flat_cmd,
        cwd=repo_root,
        env=os.environ.copy(),
        timeout_sec=timeout_sec,
    )
    if pprof_flat.returncode != 0:
        raise RuntimeError(
            f"{target.id}: go tool pprof flat view failed with code {pprof_flat.returncode}: "
            f"{pprof_flat.stderr.strip()}"
        )
    pprof_flat_path.write_text(pprof_flat.stdout, encoding="utf-8")

    pprof_cum = run_command(
        command=pprof_cum_cmd,
        cwd=repo_root,
        env=os.environ.copy(),
        timeout_sec=timeout_sec,
    )
    if pprof_cum.returncode != 0:
        raise RuntimeError(
            f"{target.id}: go tool pprof cumulative view failed with code {pprof_cum.returncode}: "
            f"{pprof_cum.stderr.strip()}"
        )
    pprof_cum_path.write_text(pprof_cum.stdout, encoding="utf-8")

    parsed_flat = parse_pprof_top(pprof_flat.stdout, max(100, top_n))
    parsed_cumulative = parse_pprof_top(pprof_cum.stdout, max(100, top_n))

    estimated_flat = estimate_rows(parsed_flat, user_cycles, prove_seconds)
    estimated_cumulative = estimate_rows(parsed_cumulative, user_cycles, prove_seconds)

    top_flat = estimated_flat[:top_n]
    top_cumulative = estimated_cumulative[:top_n]
    top_lowmc_flat = [
        row for row in estimated_flat if "lowmc_core::" in row["function"]
    ][:top_n]
    top_lowmc_cumulative = [
        row for row in estimated_cumulative if "lowmc_core::" in row["function"]
    ][:top_n]

    return {
        "target_id": target.id,
        "workdir": target.workdir,
        "command": target.command,
        "timings": {
            "prove_seconds": prove_seconds,
            "verify_seconds": verify_seconds,
            "total_seconds": total_seconds,
        },
        "cycles": {
            "total_cycles": total_cycles,
            "user_cycles": user_cycles,
            "paging_cycles": paging_cycles,
            "reserved_cycles": reserved_cycles,
        },
        "artifacts": {
            "stdout_log": display_path(stdout_path, repo_root),
            "stderr_log": display_path(stderr_path, repo_root),
            "host_json": display_path(host_json_path, repo_root),
            "profile": display_path(profile_path, repo_root),
            "pprof_top_flat": display_path(pprof_flat_path, repo_root),
            "pprof_top_cumulative": display_path(pprof_cum_path, repo_root),
        },
        "top_flat": top_flat,
        "top_cumulative": top_cumulative,
        "top_lowmc_flat": top_lowmc_flat,
        "top_lowmc_cumulative": top_lowmc_cumulative,
    }


def format_int(value: int) -> str:
    return f"{value:,}"


def display_path(path: Path, repo_root: Path) -> str:
    try:
        return str(path.relative_to(repo_root))
    except ValueError:
        return str(path)


def write_text_report(
    path: Path, run_id: str, results: list[dict[str, Any]], top_n: int
) -> None:
    lines: list[str] = []
    lines.append("LowMC function-level breakdown")
    lines.append(f"run_id: {run_id}")
    lines.append(
        "runtime estimates are prove-time shares derived from user-cycle percentages"
    )
    lines.append("")

    for result in results:
        timings = result["timings"]
        cycles = result["cycles"]
        lines.append(f"[{result['target_id']}]")
        lines.append(
            "  prove={:.3f}s verify={:.3f}s total={:.3f}s | user_cycles={} total_cycles={}".format(
                timings["prove_seconds"],
                timings["verify_seconds"],
                timings["total_seconds"],
                format_int(cycles["user_cycles"]),
                format_int(cycles["total_cycles"]),
            )
        )
        lines.append("")

        lines.append(f"  top {top_n} by flat cycles")
        lines.append(
            "    {:>4} {:>7} {:>7} {:>14} {:>13}  {}".format(
                "rank",
                "flat%",
                "cum%",
                "est_flat_cycles",
                "est_flat_s",
                "function",
            )
        )
        for row in result["top_flat"]:
            lines.append(
                "    {:>4} {:>6.2f}% {:>6.2f}% {:>14} {:>13.6f}  {}".format(
                    row["rank"],
                    row["flat_percent"],
                    row["cum_percent"],
                    format_int(row["est_flat_user_cycles"]),
                    row["est_flat_prove_seconds"],
                    row["function"],
                )
            )

        lines.append("")
        lines.append(f"  top {top_n} by cumulative cycles")
        lines.append(
            "    {:>4} {:>7} {:>7} {:>14} {:>13}  {}".format(
                "rank",
                "flat%",
                "cum%",
                "est_cum_cycles",
                "est_cum_s",
                "function",
            )
        )
        for row in result["top_cumulative"]:
            lines.append(
                "    {:>4} {:>6.2f}% {:>6.2f}% {:>14} {:>13.6f}  {}".format(
                    row["rank"],
                    row["flat_percent"],
                    row["cum_percent"],
                    format_int(row["est_cum_user_cycles"]),
                    row["est_cum_prove_seconds"],
                    row["function"],
                )
            )

        if result["top_lowmc_flat"]:
            lines.append("")
            lines.append(f"  lowmc_core::* by flat cycles (top {top_n})")
            lines.append(
                "    {:>4} {:>7} {:>7} {:>14} {:>13}  {}".format(
                    "rank",
                    "flat%",
                    "cum%",
                    "est_flat_cycles",
                    "est_flat_s",
                    "function",
                )
            )
            for row in result["top_lowmc_flat"]:
                lines.append(
                    "    {:>4} {:>6.2f}% {:>6.2f}% {:>14} {:>13.6f}  {}".format(
                        row["rank"],
                        row["flat_percent"],
                        row["cum_percent"],
                        format_int(row["est_flat_user_cycles"]),
                        row["est_flat_prove_seconds"],
                        row["function"],
                    )
                )

        if result["top_lowmc_cumulative"]:
            lines.append("")
            lines.append(f"  lowmc_core::* by cumulative cycles (top {top_n})")
            lines.append(
                "    {:>4} {:>7} {:>7} {:>14} {:>13}  {}".format(
                    "rank",
                    "flat%",
                    "cum%",
                    "est_cum_cycles",
                    "est_cum_s",
                    "function",
                )
            )
            for row in result["top_lowmc_cumulative"]:
                lines.append(
                    "    {:>4} {:>6.2f}% {:>6.2f}% {:>14} {:>13.6f}  {}".format(
                        row["rank"],
                        row["flat_percent"],
                        row["cum_percent"],
                        format_int(row["est_cum_user_cycles"]),
                        row["est_cum_prove_seconds"],
                        row["function"],
                    )
                )

        lines.append("")

    path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def write_csv_report(path: Path, results: list[dict[str, Any]]) -> None:
    fieldnames = [
        "target_id",
        "view",
        "rank",
        "function",
        "flat_percent",
        "cum_percent",
        "est_flat_user_cycles",
        "est_cum_user_cycles",
        "est_flat_prove_seconds",
        "est_cum_prove_seconds",
    ]
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for result in results:
            target_id = result["target_id"]
            for view in (
                "top_flat",
                "top_cumulative",
                "top_lowmc_flat",
                "top_lowmc_cumulative",
            ):
                for row in result[view]:
                    writer.writerow(
                        {
                            "target_id": target_id,
                            "view": view,
                            "rank": row["rank"],
                            "function": row["function"],
                            "flat_percent": row["flat_percent"],
                            "cum_percent": row["cum_percent"],
                            "est_flat_user_cycles": row["est_flat_user_cycles"],
                            "est_cum_user_cycles": row["est_cum_user_cycles"],
                            "est_flat_prove_seconds": row["est_flat_prove_seconds"],
                            "est_cum_prove_seconds": row["est_cum_prove_seconds"],
                        }
                    )


def write_json_report(
    path: Path, run_id: str, top_n: int, results: list[dict[str, Any]]
) -> None:
    payload = {
        "run_id": run_id,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "top_n": top_n,
        "note": "function runtime is estimated as prove_seconds * cycle_share",
        "results": results,
    }
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def ensure_go_pprof_available() -> None:
    proc = subprocess.run(
        ["go", "tool", "pprof", "-h"],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            "go tool pprof is required but unavailable; install Go and ensure it is on PATH"
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="One-shot function-level cycle/runtime breakdown for LowMC variants"
    )
    parser.add_argument(
        "--top",
        type=int,
        default=25,
        help="number of functions to keep for each pprof view (default: 25)",
    )
    parser.add_argument(
        "--timeout-sec",
        type=int,
        default=2400,
        help="timeout per target command in seconds (default: 2400)",
    )
    parser.add_argument(
        "--output-dir",
        default=None,
        help="optional output directory (default: artifacts/lowmc-function-breakdown/<timestamp>)",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.top <= 0:
        raise RuntimeError("--top must be greater than zero")
    if args.timeout_sec <= 0:
        raise RuntimeError("--timeout-sec must be greater than zero")

    ensure_go_pprof_available()

    repo_root = Path(__file__).resolve().parents[1]
    run_id = utc_now_compact()

    if args.output_dir:
        out_dir = Path(args.output_dir).expanduser().resolve()
    else:
        out_dir = (
            repo_root / "artifacts" / "lowmc-function-breakdown" / run_id
        ).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"[breakdown] run_id={run_id}")
    print(f"[breakdown] output_dir={display_path(out_dir, repo_root)}")

    results: list[dict[str, Any]] = []
    for target in TARGETS:
        print(f"[breakdown] running target {target.id}")
        result = run_target(
            repo_root=repo_root,
            out_dir=out_dir,
            target=target,
            top_n=args.top,
            timeout_sec=args.timeout_sec,
        )
        results.append(result)
        print(
            "[breakdown] {} done (prove={:.3f}s, user_cycles={})".format(
                target.id,
                result["timings"]["prove_seconds"],
                format_int(result["cycles"]["user_cycles"]),
            )
        )

    json_path = out_dir / "breakdown.json"
    csv_path = out_dir / "breakdown.csv"
    txt_path = out_dir / "breakdown.txt"

    write_json_report(json_path, run_id=run_id, top_n=args.top, results=results)
    write_csv_report(csv_path, results=results)
    write_text_report(txt_path, run_id=run_id, results=results, top_n=args.top)

    print(f"[breakdown] wrote {display_path(json_path, repo_root)}")
    print(f"[breakdown] wrote {display_path(csv_path, repo_root)}")
    print(f"[breakdown] wrote {display_path(txt_path, repo_root)}")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as exc:
        print(f"[breakdown] error: {exc}", file=sys.stderr)
        raise SystemExit(1)
