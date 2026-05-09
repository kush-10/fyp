# AES Optimized RISC Zero Benchmark

This workspace benchmarks the AES-192 ECB implementation used for optimization
experiments, including S-box backend comparison through crate features.

## Run

From the repository root:

```bash
make risc0-dev PROJECT=aes-r0-optimised
make risc0-prod PROJECT=aes-r0-optimised
```

From this directory:

```bash
make risc0-dev
make risc0-prod
```

## Layout

- `aesencryption`: AES-192 implementation with selectable S-box backends.
- `host`: RISC Zero host runner.
- `methods`: RISC Zero method build wrapper.
- `methods/guest`: guest program executed inside the zkVM.
