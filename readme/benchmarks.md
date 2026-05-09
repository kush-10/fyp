# Benchmarks

The benchmark harness is config-driven. Targets are defined in TOML files and
the host binaries emit JSON records with `--json`.

## Config Files

- `bench-harness/config.toml`: main encryption campaign.
- `bench-harness/config.operations.toml`: primitive operation campaign.
- `bench-harness/config.compare-auth.toml`: focused LowMC/AES/CTR/CTR+HMAC
  comparison.
- `bench-harness/config.report-main.toml`: commit-locked report main campaign.
- `bench-harness/config.report-ops.toml`: commit-locked report operation
  campaign.
- `bench-harness/config.report-ctr-1to16.toml`: report CTR block-scaling
  campaign.

Each target entry can define:

- `id`: benchmark target id.
- `enabled`: include or exclude the target.
- `workdir`: workspace directory where the command runs.
- `command`: command array to execute.
- `trials`: per-target trial count.
- `timeout_sec`: per-target timeout.

## Output Layout

Each run writes to a timestamped directory below an output root, for example:

```text
artifacts/benchmarks/<timestamp>/
artifacts/benchmarks-ops/<timestamp>/
artifacts/benchmarks-auth-compare/<timestamp>/
```

Typical contents:

- `run_manifest.json`: config, targets, runtime, git, and tool metadata.
- `raw/*.json`: one benchmark record per target/trial.
- `logs/*.stdout.log` and `logs/*.stderr.log`: captured process output.
- `aggregated.json`: medians, means, p95, standard deviation, confidence
  intervals, IQR/CV, and success metrics.
- `statistical_analysis.json`: comparison tests and effect sizes when generated
  by the report workflow.
- `plots/*.png` and `plots/*.svg`: generated charts.

## Naming

CTR benchmark ids are block-size aware:

```text
aes-ctr-1blk
aes-ctr-4blk
aes-ctr-16blk
aes-ctr-64blk
aes-ctr-hmac-1blk
aes-ctr-hmac-4blk
aes-ctr-hmac-16blk
aes-ctr-hmac-64blk
```

Operation benchmark outputs are flattened by operation name in aggregation, for
example `operation-bnchmrk-r0:and`.

For the complete JSON output contract, see `bench-harness/schema.md`.
