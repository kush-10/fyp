#!/usr/bin/env python3

import argparse
import json
import os
import platform
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

try:
    tomllib = __import__("tomllib")
except ModuleNotFoundError:
    import tomli as tomllib


@dataclass
class Target:
    id: str
    enabled: bool
    workdir: Path
    command: list[str]
    trials: int
    timeout_sec: int


def load_config(config_path: Path, repo_root: Path) -> tuple[dict, list[Target]]:
    with config_path.open("rb") as f:
        cfg = tomllib.load(f)

    defaults = cfg.get("defaults", {})
    default_trials = int(defaults.get("trials", 3))
    default_timeout = int(defaults.get("timeout_sec", 300))

    targets = []
    for raw in cfg.get("targets", []):
        target = Target(
            id=raw["id"],
            enabled=bool(raw.get("enabled", True)),
            workdir=(repo_root / raw["workdir"]).resolve(),
            command=list(raw["command"]),
            trials=int(raw.get("trials", default_trials)),
            timeout_sec=int(raw.get("timeout_sec", default_timeout)),
        )
        targets.append(target)

    return defaults, targets


def utc_now_compact() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%SZ")


def try_parse_json(stdout: str):
    text = stdout.strip()
    if not text:
        return None

    try:
        return json.loads(text)
    except json.JSONDecodeError:
        pass

    for line in reversed(text.splitlines()):
        line = line.strip()
        if not line:
            continue
        try:
            return json.loads(line)
        except json.JSONDecodeError:
            continue
    return None


def flatten_metrics(target_id: str, payload):
    metrics = []
    if not isinstance(payload, dict):
        return metrics

    if isinstance(payload.get("results"), list):
        base_algorithm = payload.get("algorithm", target_id)
        mode = payload.get("mode", "zk")
        base_params = payload.get("params", {})
        for op_result in payload["results"]:
            operation = str(op_result.get("operation", "unknown"))
            slug = operation.lower().replace(" ", "_")
            metrics.append(
                {
                    "benchmark_id": f"{target_id}:{slug}",
                    "target_id": target_id,
                    "algorithm": f"{base_algorithm}:{operation}",
                    "mode": mode,
                    "status": op_result.get("status", "ok"),
                    "params": base_params,
                    "timings": op_result.get("timings", {}),
                    "cycles": op_result.get("cycles", {}),
                }
            )
        return metrics

    metrics.append(
        {
            "benchmark_id": payload.get("benchmark_id", target_id),
            "target_id": target_id,
            "algorithm": payload.get("algorithm", target_id),
            "mode": payload.get("mode", "zk"),
            "status": payload.get("status", "ok"),
            "params": payload.get("params", {}),
            "timings": payload.get("timings", {}),
            "cycles": payload.get("cycles", {}),
        }
    )
    return metrics


def run_one_target(target: Target, trial: int, out_dir: Path, repo_root: Path) -> dict:
    logs_dir = out_dir / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)

    stamp = utc_now_compact()
    stdout_path = logs_dir / f"{target.id}.trial{trial}.stdout.log"
    stderr_path = logs_dir / f"{target.id}.trial{trial}.stderr.log"

    started = datetime.now(timezone.utc)
    base_record = {
        "timestamp_utc": started.isoformat(),
        "target_id": target.id,
        "trial": trial,
        "command": target.command,
        "workdir": str(target.workdir.relative_to(repo_root)),
        "timeout_sec": target.timeout_sec,
        "stdout_path": str(stdout_path.relative_to(repo_root)),
        "stderr_path": str(stderr_path.relative_to(repo_root)),
        "metrics": [],
    }

    try:
        proc = subprocess.run(
            target.command,
            cwd=target.workdir,
            capture_output=True,
            text=True,
            timeout=target.timeout_sec,
            check=False,
            env=os.environ.copy(),
        )
        stdout_path.write_text(proc.stdout, encoding="utf-8")
        stderr_path.write_text(proc.stderr, encoding="utf-8")

        if proc.returncode != 0:
            base_record.update(
                {
                    "status": "error",
                    "error": f"process exited with code {proc.returncode}",
                    "return_code": proc.returncode,
                }
            )
            return base_record

        payload = try_parse_json(proc.stdout)
        if payload is None:
            base_record.update(
                {
                    "status": "parse_error",
                    "error": "process succeeded but JSON output could not be parsed",
                    "return_code": proc.returncode,
                }
            )
            return base_record

        base_record.update(
            {
                "status": "ok",
                "error": None,
                "return_code": proc.returncode,
                "metrics": flatten_metrics(target.id, payload),
            }
        )
        return base_record

    except subprocess.TimeoutExpired as e:
        timeout_stdout = e.stdout or ""
        timeout_stderr = e.stderr or ""
        if isinstance(timeout_stdout, bytes):
            timeout_stdout = timeout_stdout.decode("utf-8", errors="replace")
        if isinstance(timeout_stderr, bytes):
            timeout_stderr = timeout_stderr.decode("utf-8", errors="replace")
        stdout_path.write_text(timeout_stdout, encoding="utf-8")
        stderr_path.write_text(timeout_stderr, encoding="utf-8")
        base_record.update(
            {
                "status": "timeout",
                "error": f"timed out after {target.timeout_sec}s",
                "return_code": None,
            }
        )
        return base_record


def write_json(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Run zk benchmark harness")
    parser.add_argument("--config", default="bench-harness/config.toml")
    parser.add_argument("--output-dir", default=None)
    parser.add_argument("--list", action="store_true")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    config_path = (repo_root / args.config).resolve()

    defaults, targets = load_config(config_path, repo_root)
    enabled = [t for t in targets if t.enabled]

    if args.list:
        for t in enabled:
            print(
                f"{t.id} | trials={t.trials} | timeout={t.timeout_sec}s | cwd={t.workdir.relative_to(repo_root)}"
            )
        return 0

    output_root = repo_root / defaults.get("output_root", "artifacts/benchmarks")
    run_id = utc_now_compact()
    run_dir = Path(args.output_dir) if args.output_dir else output_root / run_id
    run_dir = run_dir.resolve()
    run_dir.mkdir(parents=True, exist_ok=True)

    raw_dir = run_dir / "raw"
    raw_dir.mkdir(parents=True, exist_ok=True)

    run_manifest = {
        "run_id": run_id,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config_path": str(config_path.relative_to(repo_root)),
        "output_dir": str(run_dir.relative_to(repo_root)),
        "python": sys.version,
        "platform": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
            "processor": platform.processor(),
        },
        "targets": [
            {
                "id": t.id,
                "enabled": t.enabled,
                "trials": t.trials,
                "timeout_sec": t.timeout_sec,
                "workdir": str(t.workdir.relative_to(repo_root)),
                "command": t.command,
            }
            for t in targets
        ],
    }
    write_json(run_dir / "run_manifest.json", run_manifest)

    print(f"[bench] run_id={run_id}")
    print(f"[bench] output_dir={run_dir.relative_to(repo_root)}")

    total = 0
    failures = 0
    for target in enabled:
        for trial in range(1, target.trials + 1):
            total += 1
            print(f"[bench] {target.id} trial {trial}/{target.trials}")
            record = run_one_target(target, trial, run_dir, repo_root)
            raw_file = raw_dir / f"{target.id}.trial{trial}.json"
            write_json(raw_file, record)
            if record["status"] != "ok":
                failures += 1
                print(f"[bench] {target.id} trial {trial} -> {record['status']}")

    print(f"[bench] completed {total} trials, {failures} failed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
