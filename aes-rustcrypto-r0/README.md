# AES-128 (RustCrypto) on RISC Zero

This project runs AES-128 block encryption using the RustCrypto `aes` crate inside a RISC Zero guest and verifies the result with a receipt on the host.

It uses the same NIST AES-128 test vector as `aes-r0` so both implementations are directly comparable.

## Run

From this directory:

```bash
cargo run -p host
```

Or use the root `Justfile` shortcuts:

```bash
just aes-rustcrypto-dev
just aes-rustcrypto-prod
just aes-rustcrypto-native-dev
```

The host executes the zkVM guest, verifies the receipt, and prints benchmarking output in the same format as the other benchmarks:

- Proof generation time
- Proof verification time
- Segment and cycle statistics
