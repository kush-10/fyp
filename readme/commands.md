# Commands

Run these commands from the repository root unless noted.

## Root Makefile

Show available root targets and project names:

```bash
make list
```

Run one project in RISC Zero dev mode:

```bash
make risc0-dev PROJECT=<project>
```

Run one project in release/proving mode:

```bash
make risc0-prod PROJECT=<project>
```

Examples:

```bash
make risc0-dev PROJECT=aes-ctr
make risc0-prod PROJECT=aes-ctr-hmac
```

Available projects:

```text
aes-r0
aes-r0-optimised
aes-ctr
aes-ctr-hmac
lowmc-r0
lowmc-r0-optimised
salsa-r0
operation-bnchmrk-r0
```

## Benchmark Campaigns

Run the main encryption campaign and the operation campaign:

```bash
make bench
```

Run the focused authenticated-encryption comparison campaign:

```bash
make bench-auth-compare
```

Run the dissertation/report benchmark campaign:

```bash
make report-benchmark
```

Useful overrides:

```bash
make report-benchmark TRIALS=3
make report-benchmark LOCK_FLAGS=
PYTHON=.venv/bin/python make report-benchmark
```

`LOCK_FLAGS=--require-clean` is the default for report runs. Set
`LOCK_FLAGS=` only when you intentionally want to run against a dirty tree.

## Direct Harness Commands

List enabled targets:

```bash
python3 bench-harness/runner.py --config bench-harness/config.toml --list
python3 bench-harness/runner.py --config bench-harness/config.operations.toml --list
python3 bench-harness/runner.py --config bench-harness/config.compare-auth.toml --list
```

Run, aggregate, and plot the default campaign directly:

```bash
python3 bench-harness/runner.py --config bench-harness/config.toml
python3 bench-harness/aggregate.py --output-root artifacts/benchmarks
python3 bench-harness/plot.py --output-root artifacts/benchmarks
```

Use the matching output root for operations and auth-comparison campaigns:

```text
artifacts/benchmarks-ops
artifacts/benchmarks-auth-compare
```

## Cleanup

Clean Rust build outputs across benchmark workspaces:

```bash
make clean
```

Clean generated dissertation build files:

```bash
make clean-docs
```
