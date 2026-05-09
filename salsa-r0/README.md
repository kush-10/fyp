# Salsa20 RISC Zero Benchmark

This workspace benchmarks a manual Salsa20/20 implementation inside a RISC Zero
guest.

## Run

From the repository root:

```bash
make risc0-dev PROJECT=salsa-r0
make risc0-prod PROJECT=salsa-r0
```

From this directory:

```bash
make risc0-dev
make risc0-prod
```

Run without RISC Zero proving for native timing only:

```bash
NO_RISC0=1 cargo run -p host
```

## Layout

- `host`: RISC Zero host runner.
- `methods`: RISC Zero method build wrapper.
- `methods/guest`: guest program and Salsa20 workload.
