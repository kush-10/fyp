# This is my final year project.

## Root Just Commands

Install `just` first (macOS):

```bash
brew install just
```

Run these from the repository root:

```bash
just aes-dev
just aes-prod
just aes-rustcrypto-dev
just aes-rustcrypto-prod
just aes-ctr-dev
just aes-ctr-prod
just lowmc-dev
just lowmc-prod
just op-dev
just op-prod
just salsa-dev
just salsa-prod
just aes-native-dev
just aes-rustcrypto-native-dev
just aes-ctr-native-dev
just lowmc-native-dev
just op-native-dev
just salsa-native-dev
just all-build-dev
just all-build-prod
just all-native-dev
just clean
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
just bench-list
just bench-run
just bench-aggregate
just bench-plot
just bench-all
just bench-clean
```

### Output layout

Each run is written to `artifacts/benchmarks/<timestamp>/`:

- `run_manifest.json`
- `raw/*.json` per target+trial record
- `logs/*.stdout.log` and `logs/*.stderr.log`
- `aggregated.json` (after aggregate step)
- `plots/*.png` and `plots/*.svg` (after plot step)
