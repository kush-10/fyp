# AES-CTR RISC Zero Benchmark

This workspace benchmarks AES-192 CTR encryption inside a RISC Zero guest. It is
the confidentiality-only comparison target for the authenticated AES work.

## Run

From the repository root:

```bash
make risc0-dev PROJECT=aes-ctr
make risc0-prod PROJECT=aes-ctr
```

From this directory:

```bash
make risc0-dev
make risc0-prod
```

## Layout

- `aesencryption`: AES-192 ECB primitives plus CTR mode.
- `host`: RISC Zero host runner and JSON benchmark output.
- `methods`: RISC Zero method build wrapper.
- `methods/guest`: guest program that commits ciphertext.
