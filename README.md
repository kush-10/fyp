# This is my final year project.

## Root Make Commands

Run these from the repository root:

```bash
make aes-dev
make aes-prod
make aes-optimised-dev
make aes-optimised-prod
make lowmc-dev
make lowmc-prod
make lowmc-r0-optimised-dev
make lowmc-r0-optimised-prod
make op-dev
make op-prod
make salsa-dev
make salsa-prod
make aes-native-dev
make aes-optimised-native-dev
make lowmc-native-dev
make lowmc-r0-optimised-native-dev
make op-native-dev
make salsa-native-dev
make aes-pprof-dev
make aes-optimised-pprof-dev
make lowmc-pprof-dev
make lowmc-r0-optimised-pprof-dev
make op-pprof-dev
make salsa-pprof-dev
make all-build-dev
make all-build-prod
make all-native-dev
make lowmc-fn-breakdown
make clean
```

`make lowmc-fn-breakdown` runs a one-shot function-level profile for `lowmc-r0` and
`lowmc-r0-optimised`, then writes results to `artifacts/lowmc-function-breakdown/<timestamp>/`.

## Benchmark Harness

The benchmark harness is config-driven and runs zk benchmarks only.

- `bench-harness/config.toml` is the encryption suite (AES, AES optimised, Salsa, LowMC, LowMC optimised)
- `bench-harness/config.operations.toml` is the operations-only suite

### Install Python dependencies

```bash
python3 -m pip install -r bench-harness/requirements.txt
```

### Configure targets

Edit `bench-harness/config.toml` for encryption benchmarks, or
`bench-harness/config.operations.toml` for the operations suite.

- Set `enabled = true/false` per target
- Set `trials` per target (or use `[defaults].trials`)
- Set `timeout_sec` per target (or use `[defaults].timeout_sec`)
- Update `command` if a target needs custom args

### Run commands

```bash
make bench-list
make bench-run
make bench-aggregate
make bench-plot
make bench-all
make bench-clean
make bench-op-list
make bench-op-run
make bench-op-aggregate
make bench-op-plot
make bench-op-all
make bench-op-clean
```

### Output layout

Encryption suite runs are written to `artifacts/benchmarks/<timestamp>/` and
operations suite runs are written to `artifacts/benchmarks-ops/<timestamp>/`.

Each run directory contains:

- `run_manifest.json`
- `raw/*.json` per target+trial record
- `logs/*.stdout.log` and `logs/*.stderr.log`
- `aggregated.json` (after aggregate step)
- `plots/*.png` and `plots/*.svg` (after plot step)
