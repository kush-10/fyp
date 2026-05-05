# FYP: Benchmarking Symmetric Cryptography in RISC Zero

This repository contains the benchmarking and analysis code used for a final-year
project focused on symmetric cryptography in a zkVM (RISC Zero).

The core workflow is:

1. run cryptographic targets through the same host/guest proving pipeline;
2. collect JSON artifacts per trial;
3. aggregate and plot proving-time and cycle metrics;
4. compare implementation strategies, including authenticated AES variants.

## Repository Layout

Top-level benchmark workspaces:

- `aes-r0`: AES baseline in zkVM.
- `aes-r0-optimised`: AES workspace with S-box backend experimentation.
- `aes-ctr`: AES-CTR benchmark target (confidentiality path).
- `aes-ctr-hmac`: AES-CTR + HMAC-SHA256-192 benchmark target
  (Encrypt-then-MAC path).
- `lowmc-r0`: LowMC baseline.
- `lowmc-r0-optimised`: LowMC optimization workspace.
- `salsa-r0`: Salsa20 benchmark target.
- `operation-bnchmrk-r0`: operation-level directional benchmark target.
- `bench-harness`: Python runner, aggregation, plotting, and schema notes.
- `docs`: dissertation report source (`.tex`) and generated report artifacts.

## Prerequisites

- Rust and Cargo (toolchains are managed per workspace where needed).
- Python 3.
- Python packages for harness tooling:

```bash
python3 -m pip install -r bench-harness/requirements.txt
```

## Root Make Workflow

Run all commands from the repository root.

### Show available targets and current project

```bash
make list
```

### Run one project in dev mode

```bash
make risc0-dev PROJECT=<project>
```

Example:

```bash
make risc0-dev PROJECT=aes-ctr
```

### Run one project in release mode

```bash
make risc0-prod PROJECT=<project>
```

Example:

```bash
make risc0-prod PROJECT=aes-ctr-hmac
```

### Run benchmark campaigns

```bash
make bench
make bench-auth-compare
```

- `make bench` runs the main encryption campaign (`config.toml`), then the
  operations campaign (`config.operations.toml`), and generates both aggregate
  files and plots.
- `make bench-auth-compare` runs the focused comparison campaign defined in
  `bench-harness/config.compare-auth.toml`.

### Cleanup targets

```bash
make clean
make clean-docs
```

- `make clean` runs `cargo clean` across all benchmark workspaces.
- `make clean-docs` cleans generated dissertation build artifacts in `docs/`.

## Project Switch (Important)

Single-target execution now uses one shared root target plus a `PROJECT` switch,
rather than separate per-project make targets.

The authenticated AES flow is also split structurally into two workspaces:

- `aes-ctr`: CTR-only encryption path.
- `aes-ctr-hmac`: CTR + HMAC authenticated path.

This means CTR and CTR+HMAC are selected by project/workspace choice in root
commands and benchmark config targets, not by a single runtime mode toggle.

## Benchmark Harness

The harness is config-driven and expects host binaries to emit JSON (`--json`).

### Config files

- `bench-harness/config.toml`: main encryption suite (includes AES, LowMC,
  Salsa, and AES CTR/CTR+HMAC block-sweep targets).
- `bench-harness/config.operations.toml`: operation-only suite.
- `bench-harness/config.compare-auth.toml`: focused auth comparison campaign.

Each target supports:

- `id`: unique benchmark target id.
- `enabled`: include/exclude target without deleting config.
- `workdir`: workspace directory where command runs.
- `command`: full command array to execute.
- `trials`: per-target trial count (optional; defaults supported).
- `timeout_sec`: per-target timeout (optional; defaults supported).

### Useful direct harness commands

List enabled targets:

```bash
python3 bench-harness/runner.py --config bench-harness/config.toml --list
python3 bench-harness/runner.py --config bench-harness/config.operations.toml --list
python3 bench-harness/runner.py --config bench-harness/config.compare-auth.toml --list
```

Run a campaign directly:

```bash
python3 bench-harness/runner.py --config bench-harness/config.toml
```

Aggregate a run:

```bash
python3 bench-harness/aggregate.py --output-root artifacts/benchmarks
```

Plot from latest aggregate:

```bash
python3 bench-harness/plot.py --output-root artifacts/benchmarks
```

For operations and auth-compare runs, use the corresponding output roots:

- `artifacts/benchmarks-ops`
- `artifacts/benchmarks-auth-compare`

### Output layout

Each harness run writes to a timestamped directory:

- `artifacts/benchmarks/<timestamp>/`
- `artifacts/benchmarks-ops/<timestamp>/`
- `artifacts/benchmarks-auth-compare/<timestamp>/`

Typical run contents:

- `run_manifest.json`: full run metadata (config, targets, machine/runtime info).
- `raw/*.json`: one record per target/trial.
- `logs/*.stdout.log` and `logs/*.stderr.log`: captured process output.
- `aggregated.json`: computed medians, means, p95, stddev, and success metrics.
- `plots/*.png` and `plots/*.svg`: generated charts.

### Benchmark IDs and naming

Target ids are descriptive and block-size aware for CTR campaigns, for example:

- `aes-ctr-1blk`, `aes-ctr-4blk`, `aes-ctr-16blk`, `aes-ctr-64blk`
- `aes-ctr-hmac-1blk`, `aes-ctr-hmac-4blk`, `aes-ctr-hmac-16blk`, `aes-ctr-hmac-64blk`

Operation benchmark outputs are flattened by operation name in aggregation, such
as `operation-bnchmrk-r0:and`.

For full output contract details, see `bench-harness/schema.md`.

## Dissertation Build

Build report PDF:

```bash
make -C docs
```

Clean report artifacts:

```bash
make clean-docs
```
