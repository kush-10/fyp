# AES-128 in Rust

Small, dependency-free AES-128 (ECB) implementation that works in `no_std` + `alloc` environments. It processes hex-encoded plaintext/ciphertext and 128-bit hex keys.

### Reference

This implementation is based on:

- [NIST FIPS 197](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.197-upd1.pdf) - Advanced Encryption Standard (AES)

## Features

- `no_std` (uses `alloc` only); no git or external deps
- AES-128 key expansion and round transformations (SubBytes, ShiftRows, MixColumns, AddRoundKey)
- Hex in/out helpers for block-aligned inputs (ECB mode)
- Unit test covering the NIST AES-128 test vector

## Usage

```toml
[dependencies]
aesencryption = { path = "." } # or copy src/ into your guest crate
```

```rust
use aesencryption::{decrypt_hex, encrypt_hex};

fn main() {
    // 3 AES blocks of plaintext (hex-encoded), 128-bit key
    let plaintext = "6BC1BEE22E409F96E93D7E117393172AAE2D8A571E03AC9C9EB76FAC45AF8E5130C81C46A35CE411E5FBC1191A0A52EFF69F2445DF4F9B17AD2B417BE66C3710";
    let key = "2B7E151628AED2A6ABF7158809CF4F3C";

    let ciphertext = encrypt_hex(plaintext, key).expect("encrypt");
    let recovered = decrypt_hex(&ciphertext, key).expect("decrypt");

    assert_eq!(recovered, plaintext);
}
```

## Notes

- Mode: ECB only; inputs must be block-aligned (multiples of 32 hex chars / 16 bytes). Add your own padding/IV/mode if needed.
- Errors: `encrypt_hex`/`decrypt_hex` return `AesError` for malformed hex, wrong key length, or non-block-aligned input.

## Testing

```sh
cargo test
```
