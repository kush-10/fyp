# FYP: Benchmarking Symmetric Cryptography in RISC Zero

This repository contains the code, benchmark harness, and dissertation sources
for a final-year project measuring symmetric cryptography inside the RISC Zero
zkVM.

The main workflow is:

1. run each cryptographic target through a shared host/guest proving pipeline;
2. collect per-trial JSON benchmark artifacts;
3. aggregate and plot proving-time, cycle, and proof-size metrics;
4. compare baseline, optimized, stream-mode, authenticated, and operation-level
   workloads.

## Start Here

Run commands from this repository root unless a component README says otherwise.

```bash
make list
```

Common examples:

```bash
make risc0-dev PROJECT=aes-ctr
make risc0-prod PROJECT=aes-ctr-hmac
make report-benchmark TRIALS=5
```

Install Python dependencies before running the Python benchmark harness directly:

```bash
python3 -m pip install -r bench-harness/requirements.txt
```

## Repository Layout

- `aes-r0`: AES-192 ECB baseline benchmark target.
- `aes-r0-optimised`: AES-192 ECB benchmark target used for S-box backend work.
- `aes-ctr`: AES-192 CTR benchmark target.
- `aes-ctr-hmac`: AES-192 CTR plus HMAC-SHA256-192 benchmark target.
- `lowmc-r0`: LowMC baseline benchmark target.
- `lowmc-r0-optimised`: LowMC optimization benchmark target.
- `salsa-r0`: Salsa20 benchmark target.
- `operation-bnchmrk-r0`: primitive operation benchmark target.
- `bench-harness`: Python runner, aggregation, plotting, and schema notes.
- `docs`: dissertation report source and generated PDF artifacts.
- `artifacts`: generated benchmark outputs and curated report results.
- `readme`: detailed command, benchmark, project, and result notes.

## More Detail

- `readme/commands.md`: root Makefile usage and direct harness commands.
- `readme/projects.md`: short notes for each benchmark workspace.
- `readme/benchmarks.md`: campaign configs, output layout, and naming.
- `readme/results.md`: curated local benchmark result locations.

Each benchmark workspace also has a local `README.md` with the shortest command
path for that component.
