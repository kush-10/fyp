# AES Baseline RISC Zero Benchmark

This workspace benchmarks an AES-192 ECB implementation inside a RISC Zero
guest. The host runs the guest, verifies the receipt, and prints timing and
cycle information.

## Run

From the repository root:

```bash
make risc0-dev PROJECT=aes-r0
make risc0-prod PROJECT=aes-r0
```

From this directory:

```bash
make risc0-dev
make risc0-prod
```

## Layout

- `lib`: AES-192 implementation used by the guest.
- `host`: RISC Zero host runner.
- `methods`: RISC Zero method build wrapper.
- `methods/guest`: guest program executed inside the zkVM.
