//! AES-128 CTR (Counter) mode implementation.
//!
//! CTR mode turns the AES block cipher into a stream cipher by encrypting
//! successive counter blocks and XOR-ing the resulting keystream with the
//! data. Because only the forward AES cipher is used, decryption is
//! identical to encryption.

use crate::{encrypt_block, key_expansion};
use alloc::vec::Vec;

/// Encrypts (or decrypts) arbitrary-length data using AES-128 CTR mode.
///
/// `iv` is the 16-byte initial counter block (ICB). After each 16-byte
/// block the counter is incremented as a 128-bit big-endian integer,
/// matching the NIST SP 800-38A specification.
///
/// Unlike ECB/CBC, CTR mode does **not** require block-aligned input;
/// the final partial block is simply XOR-ed with the truncated keystream.
pub fn encrypt_ctr(data: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Vec<u8> {
    let schedule = key_expansion(*key);
    let mut counter_block = *iv;
    let mut output = Vec::with_capacity(data.len());

    for chunk in data.chunks(16) {
        let keystream = encrypt_block(counter_block, &schedule);
        for (i, &byte) in chunk.iter().enumerate() {
            output.push(byte ^ keystream[i]);
        }
        increment_counter(&mut counter_block);
    }

    output
}

/// Increments a 16-byte counter block as a 128-bit big-endian integer.
fn increment_counter(counter: &mut [u8; 16]) {
    for i in (0..16).rev() {
        counter[i] = counter[i].wrapping_add(1);
        if counter[i] != 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NIST SP 800-38A Appendix F.5 -- AES-128 CTR test vector.
    #[test]
    fn nist_aes128_ctr_encrypt() {
        let key: [u8; 16] = [
            0x2B, 0x7E, 0x15, 0x16, 0x28, 0xAE, 0xD2, 0xA6, 0xAB, 0xF7, 0x15, 0x88, 0x09, 0xCF,
            0x4F, 0x3C,
        ];
        let iv: [u8; 16] = [
            0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD,
            0xFE, 0xFF,
        ];
        let plaintext: [u8; 64] = [
            // Block 1
            0x6B, 0xC1, 0xBE, 0xE2, 0x2E, 0x40, 0x9F, 0x96, 0xE9, 0x3D, 0x7E, 0x11, 0x73, 0x93,
            0x17, 0x2A, // Block 2
            0xAE, 0x2D, 0x8A, 0x57, 0x1E, 0x03, 0xAC, 0x9C, 0x9E, 0xB7, 0x6F, 0xAC, 0x45, 0xAF,
            0x8E, 0x51, // Block 3
            0x30, 0xC8, 0x1C, 0x46, 0xA3, 0x5C, 0xE4, 0x11, 0xE5, 0xFB, 0xC1, 0x19, 0x1A, 0x0A,
            0x52, 0xEF, // Block 4
            0xF6, 0x9F, 0x24, 0x45, 0xDF, 0x4F, 0x9B, 0x17, 0xAD, 0x2B, 0x41, 0x7B, 0xE6, 0x6C,
            0x37, 0x10,
        ];
        // Expected ciphertext verified against OpenSSL:
        //   openssl enc -aes-128-ctr -K 2B7E...3C -iv F0F1...FF -nopad
        let expected_ciphertext: [u8; 64] = [
            // Block 1
            0x87, 0x4D, 0x61, 0x91, 0xB6, 0x20, 0xE3, 0x26, 0x1B, 0xEF, 0x68, 0x64, 0x99, 0x0D,
            0xB6, 0xCE, // Block 2
            0x98, 0x06, 0xF6, 0x6B, 0x79, 0x70, 0xFD, 0xFF, 0x86, 0x17, 0x18, 0x7B, 0xB9, 0xFF,
            0xFD, 0xFF, // Block 3
            0x5A, 0xE4, 0xDF, 0x3E, 0xDB, 0xD5, 0xD3, 0x5E, 0x5B, 0x4F, 0x09, 0x02, 0x0D, 0xB0,
            0x3E, 0xAB, // Block 4
            0x1E, 0x03, 0x1D, 0xDA, 0x2F, 0xBE, 0x03, 0xD1, 0x79, 0x21, 0x70, 0xA0, 0xF3, 0x00,
            0x9C, 0xEE,
        ];

        let ciphertext = encrypt_ctr(&plaintext, &key, &iv);
        assert_eq!(ciphertext, expected_ciphertext);

        // CTR decryption is the same operation applied to ciphertext.
        let recovered = encrypt_ctr(&ciphertext, &key, &iv);
        assert_eq!(recovered, plaintext);
    }

    /// CTR mode handles partial final blocks correctly.
    #[test]
    fn partial_block_roundtrip() {
        let key = [0x00u8; 16];
        let iv = [0x00u8; 16];
        let plaintext = [0xAA; 25]; // 1 full block + 9 bytes

        let ciphertext = encrypt_ctr(&plaintext, &key, &iv);
        assert_eq!(ciphertext.len(), 25);

        let recovered = encrypt_ctr(&ciphertext, &key, &iv);
        assert_eq!(recovered, plaintext);
    }

    /// Single-block encryption matches the first block of the NIST vector.
    #[test]
    fn single_block_matches_nist() {
        let key: [u8; 16] = [
            0x2B, 0x7E, 0x15, 0x16, 0x28, 0xAE, 0xD2, 0xA6, 0xAB, 0xF7, 0x15, 0x88, 0x09, 0xCF,
            0x4F, 0x3C,
        ];
        let iv: [u8; 16] = [
            0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD,
            0xFE, 0xFF,
        ];
        let plaintext: [u8; 16] = [
            0x6B, 0xC1, 0xBE, 0xE2, 0x2E, 0x40, 0x9F, 0x96, 0xE9, 0x3D, 0x7E, 0x11, 0x73, 0x93,
            0x17, 0x2A,
        ];
        let expected: [u8; 16] = [
            0x87, 0x4D, 0x61, 0x91, 0xB6, 0x20, 0xE3, 0x26, 0x1B, 0xEF, 0x68, 0x64, 0x99, 0x0D,
            0xB6, 0xCE,
        ];

        let ciphertext = encrypt_ctr(&plaintext, &key, &iv);
        assert_eq!(ciphertext, expected);
    }
}
