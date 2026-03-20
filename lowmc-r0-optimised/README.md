# LowMC on RISC Zero

This project ports the reference LowMC implementation from https://github.com/LowMC/lowmc to Rust and runs it inside a RISC Zero guest.

It uses the parameter set from the reference implementation:

- Block size: 256
- Key size: 80
- Number of S-boxes: 49
- Rounds: 12

## Run

From this directory:

```bash
cargo run -p host
```

Or use the root `Makefile` shortcuts:

```bash
make risc0-dev PROJECT=lowmc-r0-optimised
make risc0-prod PROJECT=lowmc-r0-optimised
```

The host executes the zkVM guest, verifies the receipt, and prints performance output in the same style as `aes-r0`:

- Proof generation time
- Proof verification time
- Segment and cycle statistics

Input vectors are passed as fixed-size byte arrays (`[u8; 32]` plaintext/ciphertext and `[u8; 10]` key).
