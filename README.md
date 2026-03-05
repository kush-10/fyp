# This is my final year project.

## Root Make Commands

Run these from the repository root:

```bash
make aes-dev
make aes-prod
make aes-rustcrypto-dev
make aes-rustcrypto-prod
make aes-ctr-dev
make aes-ctr-prod
make lowmc-dev
make lowmc-prod
make op-dev
make op-prod
make salsa-dev
make salsa-prod
make aes-native-dev
make aes-rustcrypto-native-dev
make aes-ctr-native-dev
make lowmc-native-dev
make op-native-dev
make salsa-native-dev
make all-build-dev
make all-build-prod
make all-native-dev
make clean
```

## Benchmark Harness

The benchmark harness is config-driven and runs zk benchmarks only.

### Install Python dependencies

```bash
python3 -m pip install -r bench-harness/requirements.txt
```

### Configure targets

Edit `bench-harness/config.toml`.

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
```

### Output layout

Each run is written to `artifacts/benchmarks/<timestamp>/`:

- `run_manifest.json`
- `raw/*.json` per target+trial record
- `logs/*.stdout.log` and `logs/*.stderr.log`
- `aggregated.json` (after aggregate step)
- `plots/*.png` and `plots/*.svg` (after plot step)
