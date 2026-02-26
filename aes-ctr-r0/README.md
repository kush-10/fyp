# AES-128-CTR on RISC Zero

This project runs AES-128 in CTR mode inside a RISC Zero guest and verifies the result with a receipt on the host.

It uses the NIST SP 800-38A F.5 AES-128-CTR vector:

- Key size: 128 bits
- Counter block size: 128 bits
- Counter increment: big-endian increment over the full 16-byte counter block

## Run

From this directory:

```bash
cargo run -p host
```

Or use the root `Justfile` shortcuts:

```bash
just aes-ctr-dev
just aes-ctr-prod
just aes-ctr-native-dev
```

The host executes the zkVM guest, verifies the receipt, and prints benchmarking output in the same style as `aes-r0`:

- Proof generation time
- Proof verification time
- Segment and cycle statistics
