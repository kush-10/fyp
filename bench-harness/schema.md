# Benchmark JSON Schema (Practical Contract)

This describes the JSON contract emitted by host binaries (`--json`) and consumed by `bench-harness/runner.py`.

## Single-target result shape

```json
{
  "benchmark_id": "aes-r0",
  "algorithm": "aes-128",
  "mode": "zk",
  "status": "ok",
  "timings": {
    "prove_seconds": 1.23,
    "verify_seconds": 0.04,
    "total_seconds": 1.27
  },
  "cycles": {
    "total_cycles": 123456,
    "user_cycles": 120000,
    "paging_cycles": 3000,
    "reserved_cycles": 456
  },
  "params": {
    "payload_bytes": 64
  }
}
```

## Multi-result shape (used by operation benchmark)

```json
{
  "benchmark_id": "operation-bnchmrk-r0",
  "algorithm": "operations",
  "mode": "zk",
  "status": "ok",
  "params": {
    "iterations": 100
  },
  "results": [
    {
      "operation": "And",
      "status": "ok",
      "timings": {
        "prove_seconds": 0.95,
        "verify_seconds": 0.03,
        "total_seconds": 0.98
      },
      "cycles": {
        "total_cycles": 45000,
        "user_cycles": 42000,
        "paging_cycles": 2500,
        "reserved_cycles": 500
      }
    }
  ]
}
```

The runner flattens this into separate benchmark IDs like `operation-bnchmrk-r0:and`.

## Raw run record shape (`raw/*.json`)

- `status`: `ok`, `timeout`, `error`, or `parse_error`
- `metrics`: flattened benchmark records (empty for failed runs)
- `stdout_path` / `stderr_path`: captured process logs

## Aggregated shape (`aggregated.json`)

For each benchmark:
- attempt counters: `success_count`, `timeout_count`, `error_count`, `parse_error_count`, `attempted`, `success_rate`
- stats blocks for timings and cycles: `count`, `mean`, `median`, `p95`, `stddev`, `min`, `max`
