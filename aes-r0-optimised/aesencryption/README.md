# AES-192 Library

Small `no_std` AES-192 implementation used by the RISC Zero AES benchmark
targets. It provides block-aligned byte and hex helpers for ECB-mode benchmark
workloads.

## Features

- `no_std` outside tests, using `alloc` only.
- AES-192 key expansion and round transformations.
- `sbox-table` feature enabled by default.
- `sbox-logic` feature for logic-based S-box experiments.
- Unit tests for AES behavior.

## Usage

```toml
[dependencies]
aesencryption = { path = "aesencryption" }
```

```rust
use aesencryption::{decrypt_hex, encrypt_hex};

let plaintext = "00112233445566778899AABBCCDDEEFF";
let key = "000102030405060708090A0B0C0D0E0F1011121314151617";

let ciphertext = encrypt_hex(plaintext, key).expect("encrypt");
let recovered = decrypt_hex(&ciphertext, key).expect("decrypt");
assert_eq!(recovered, plaintext);
```

## Notes

ECB mode is used here for isolated block-cipher benchmarking. It is not an
authenticated encryption construction and should not be used as an application
security design.
