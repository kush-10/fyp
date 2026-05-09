# LowMC Baseline RISC Zero Benchmark

This workspace ports the reference LowMC implementation to Rust and runs it
inside a RISC Zero guest.

Parameter set:

- Block size: 256 bits.
- Key size: 80 bits.
- S-boxes: 49.
- Rounds: 12.

## Run

From the repository root:

```bash
make risc0-dev PROJECT=lowmc-r0
make risc0-prod PROJECT=lowmc-r0
```

From this directory:

```bash
make risc0-dev
make risc0-prod
```

## Layout

- `host`: RISC Zero host runner.
- `methods`: RISC Zero method build wrapper.
- `methods/guest`: guest program and LowMC workload.

Input vectors are fixed-size byte arrays for a 256-bit block and 80-bit key.
