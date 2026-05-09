# AES-CTR+HMAC RISC Zero Benchmark

This workspace benchmarks AES-192 CTR with a truncated HMAC-SHA256-192 tag
inside a RISC Zero guest. It is the authenticated-encryption comparison target.

## Run

From the repository root:

```bash
make risc0-dev PROJECT=aes-ctr-hmac
make risc0-prod PROJECT=aes-ctr-hmac
```

From this directory:

```bash
make risc0-dev
make risc0-prod
```

## Layout

- `aesencryption`: AES-192 CTR and HMAC-SHA256-192 implementation.
- `host`: RISC Zero host runner and JSON benchmark output.
- `methods`: RISC Zero method build wrapper.
- `methods/guest`: guest program that commits ciphertext and tag.

## Security Note

The benchmark uses fixed test material for repeatability. Production use needs
fresh IVs/nonces, separate encryption and MAC keys, and an external identity
layer where sender identity matters.
