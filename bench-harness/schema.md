# Benchmark JSON Schema (Practical Contract)

This file documents the JSON contract emitted by host binaries (`--json`) and
consumed by `bench-harness/runner.py`, `bench-harness/aggregate.py`, and
`bench-harness/plot.py`.

Use this as the compatibility reference when adding new benchmark targets.

## Contract Scope

The contract has three layers:

1. host process output (`stdout`) in either single-result or multi-result form;
2. per-trial raw harness records in `raw/*.json`;
3. aggregated campaign output in `aggregated.json`.

The harness is intentionally tolerant of optional fields, but field names and
overall object structure should remain stable to keep aggregation and plotting
reproducible.

## Single-Result Host Payload

This is the common shape for one benchmark metric record from one host run.

```json
{
  "benchmark_id": "aes-ctr-hmac-4blk",
  "algorithm": "aes-128-ctr+hmac-sha256-192",
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
    "payload_bytes": 64,
    "num_blocks": 4,
    "proof_bytes": 1024,
    "full_receipt_bytes": 4096
  }
}
```

### Single-result field notes

- `benchmark_id`: concrete benchmark instance identifier (for example
  `aes-ctr-4blk`, `aes-ctr-hmac-4blk`, `lowmc-r0-optimised`).
- `algorithm`: display/grouping label used in plots and reports.
- `mode`: runtime mode such as `zk` or `native`.
- `status`: benchmark status from the host payload perspective (typically `ok`).
- `timings`: measured times in seconds.
- `cycles`: prover cycle metrics when available.
- `params`: target-specific metadata; schema is extensible per benchmark.

## Multi-Result Host Payload

This shape is used by the operation benchmark, where one run emits several
operation-level metric records.

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

### Flattening behavior

`runner.py` flattens each `results[]` entry into a metric object with:

- `benchmark_id = <target_id>:<operation_slug>` (for example
  `operation-bnchmrk-r0:and`);
- `algorithm = <base_algorithm>:<operation_name>`;
- inherited `mode` and `params` from the top-level payload;
- per-operation `status`, `timings`, and `cycles`.

## Raw Trial Record (`raw/*.json`)

Each target trial produces one raw record, regardless of success.

Important fields:

- `timestamp_utc`: trial start timestamp.
- `target_id`: target id from config.
- `trial`: 1-based trial index.
- `command`: executed command array.
- `workdir`: workspace path relative to repo root.
- `timeout_sec`: effective timeout for this trial (`null` means no timeout).
- `status`: one of `ok`, `timeout`, `error`, `parse_error`.
- `return_code`: process exit code when available.
- `metrics`: flattened metric list (empty on non-`ok` trial states).
- `stdout_path` and `stderr_path`: captured logs for audit/debug.

### Trial status semantics

- `ok`: process succeeded and stdout contained parseable JSON.
- `timeout`: process exceeded configured timeout.
- `error`: process exited non-zero.
- `parse_error`: process exited zero but stdout was not parseable as expected.

## Aggregated Campaign Output (`aggregated.json`)

The aggregator groups raw metrics by `benchmark_id`, then computes summary stats
for each timing and cycle series.

Top-level fields:

- `generated_at_utc`: aggregation timestamp.
- `run_dir`: run directory relative to repo root.
- `benchmark_count`: number of aggregated benchmark ids.
- `benchmarks`: list of per-benchmark summary objects.

Per-benchmark fields:

- identity/context: `benchmark_id`, `algorithm`, `mode`, `params`;
- attempt counters: `success_count`, `timeout_count`, `error_count`,
  `parse_error_count`, `attempted`, `success_rate`;
- `timings` and `cycles`: nested stat blocks for each metric family.

Stat block shape for each numeric series:

- `count`, `mean`, `median`, `p95`, `stddev`, `min`, `max`.

If no values are available for a series, the stat block is an empty object.

## Structural Split Note: `aes-ctr` vs `aes-ctr-hmac`

The authenticated AES path now uses two explicit benchmark target families:

- CTR-only targets (workspace `aes-ctr`), such as `aes-ctr-4blk`.
- CTR+HMAC targets (workspace `aes-ctr-hmac`), such as `aes-ctr-hmac-4blk`.

This is a structural target split in config and output ids, rather than a single
runtime mode switch inside one benchmark id family. Keep this naming stable,
because plotting and comparison order logic depends on these ids.

## Compatibility Checklist for New Targets

When adding a benchmark target, verify:

1. host emits valid JSON as the last parseable line on stdout;
2. payload includes `benchmark_id`, `algorithm`, `mode`, `status`, `timings`,
   `cycles`, `params` (or a `results[]` wrapper for multi-result targets);
3. target id in config is unique and matches expected artifact naming;
4. at least one successful trial appears in `raw/*.json`;
5. `aggregated.json` includes expected benchmark ids and non-empty timing/cycle
   stats for successful runs.
