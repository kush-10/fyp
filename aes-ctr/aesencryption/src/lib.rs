#![cfg_attr(not(test), no_std)]

//! AES-128 implementation with ECB and CTR mode support.
//!
//! This crate is derived from the `aesencryption` crate used by the
//! `aes-r0-optimised` benchmark and extended with a CTR (counter) mode
//! implementation suitable for stream encryption inside the RISC Zero zkVM.

extern crate alloc;

use alloc::{string::String, vec::Vec};

pub mod ctr;
mod mixcolumns;
mod shiftrows;
mod subbytes;

use mixcolumns::{inv_mix_columns, mix_columns};
use shiftrows::{inv_shift_rows, shift_rows};
use subbytes::{inv_sub_bytes, sub_bytes};

type Word = [u8; 4];
type State = [Word; 4];
type RoundKey = State;
type KeySchedule = [RoundKey; 11];

/// AES input/encoding validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AesError {
    InvalidHexLength,
    InvalidKeyLength,
    InvalidBlockLength,
    InvalidHexCharacter(char),
}

// ── Public CTR convenience re-export ────────────────────────────────────

pub use ctr::encrypt_ctr;

// ── Public ECB API (unchanged from aes-r0-optimised) ────────────────────

/// Encrypts block-aligned hex plaintext with AES-128 ECB.
pub fn encrypt_hex(plaintext_hex: &str, key_hex: &str) -> Result<String, AesError> {
    let key = parse_key(key_hex)?;
    let schedule = key_expansion(key);
    let blocks = hex_to_blocks(plaintext_hex)?;

    let mut out = String::with_capacity(plaintext_hex.len());
    for block in blocks {
        let encrypted = encrypt_block(block, &schedule);
        out.push_str(&bytes_to_hex(&encrypted));
    }
    Ok(out)
}

/// Decrypts block-aligned hex ciphertext with AES-128 ECB.
pub fn decrypt_hex(ciphertext_hex: &str, key_hex: &str) -> Result<String, AesError> {
    let key = parse_key(key_hex)?;
    let schedule = key_expansion(key);
    let blocks = hex_to_blocks(ciphertext_hex)?;

    let mut out = String::with_capacity(ciphertext_hex.len());
    for block in blocks {
        let decrypted = decrypt_block(block, &schedule);
        out.push_str(&bytes_to_hex(&decrypted));
    }
    Ok(out)
}

/// Encrypts a block-aligned byte payload with AES-128 ECB.
pub fn encrypt_bytes(plaintext: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, AesError> {
    if plaintext.len() % 16 != 0 {
        return Err(AesError::InvalidBlockLength);
    }

    let schedule = key_expansion(*key);
    let mut out = Vec::with_capacity(plaintext.len());

    for chunk in plaintext.chunks_exact(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        out.extend_from_slice(&encrypt_block(block, &schedule));
    }

    Ok(out)
}

/// Decrypts a block-aligned byte payload with AES-128 ECB.
pub fn decrypt_bytes(ciphertext: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, AesError> {
    if ciphertext.len() % 16 != 0 {
        return Err(AesError::InvalidBlockLength);
    }

    let schedule = key_expansion(*key);
    let mut out = Vec::with_capacity(ciphertext.len());

    for chunk in ciphertext.chunks_exact(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        out.extend_from_slice(&decrypt_block(block, &schedule));
    }

    Ok(out)
}

// ── Internal block cipher ───────────────────────────────────────────────

/// Encrypts a single 16-byte block with the given key schedule.
///
/// Made `pub(crate)` so the CTR module can call it directly.
pub(crate) fn encrypt_block(block: [u8; 16], schedule: &KeySchedule) -> [u8; 16] {
    let mut state = add_round_key(block_to_state(block), &schedule[0]);

    for round in 1..10 {
        state = sub_bytes(state);
        state = shift_rows(state);
        state = mix_columns(state);
        state = add_round_key(state, &schedule[round]);
    }

    state = sub_bytes(state);
    state = shift_rows(state);
    state = add_round_key(state, &schedule[10]);

    state_to_block(state)
}

fn decrypt_block(block: [u8; 16], schedule: &KeySchedule) -> [u8; 16] {
    let mut state = add_round_key(block_to_state(block), &schedule[10]);

    for round in (1..10).rev() {
        state = inv_shift_rows(state);
        state = inv_sub_bytes(state);
        state = add_round_key(state, &schedule[round]);
        state = inv_mix_columns(state);
    }

    state = inv_shift_rows(state);
    state = inv_sub_bytes(state);
    state = add_round_key(state, &schedule[0]);

    state_to_block(state)
}

// ── Key expansion ───────────────────────────────────────────────────────

pub(crate) fn key_expansion(key: [u8; 16]) -> KeySchedule {
    const RCON: [Word; 10] = [
        [0x01, 0x00, 0x00, 0x00],
        [0x02, 0x00, 0x00, 0x00],
        [0x04, 0x00, 0x00, 0x00],
        [0x08, 0x00, 0x00, 0x00],
        [0x10, 0x00, 0x00, 0x00],
        [0x20, 0x00, 0x00, 0x00],
        [0x40, 0x00, 0x00, 0x00],
        [0x80, 0x00, 0x00, 0x00],
        [0x1b, 0x00, 0x00, 0x00],
        [0x36, 0x00, 0x00, 0x00],
    ];

    let mut w = [[0u8; 4]; 44];
    for (i, chunk) in key.chunks_exact(4).enumerate() {
        w[i].copy_from_slice(chunk);
    }

    for i in 4..44 {
        let mut temp = w[i - 1];
        if i % 4 == 0 {
            temp = xor_words(sub_word(rot_word(temp)), RCON[(i / 4) - 1]);
        }
        w[i] = xor_words(w[i - 4], temp);
    }

    let mut schedule = [[[0u8; 4]; 4]; 11];
    for (round, chunk) in w.chunks_exact(4).enumerate() {
        for (col, word) in chunk.iter().enumerate() {
            schedule[round][col] = *word;
        }
    }
    schedule
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn rot_word(word: Word) -> Word {
    [word[1], word[2], word[3], word[0]]
}

fn sub_word(word: Word) -> Word {
    let mut out = word;
    for byte in out.iter_mut() {
        *byte = subbytes::sub_word_byte(*byte);
    }
    out
}

fn xor_words(a: Word, b: Word) -> Word {
    [a[0] ^ b[0], a[1] ^ b[1], a[2] ^ b[2], a[3] ^ b[3]]
}

fn block_to_state(block: [u8; 16]) -> State {
    let mut state = [[0u8; 4]; 4];
    for (col, chunk) in block.chunks_exact(4).enumerate() {
        state[col].copy_from_slice(chunk);
    }
    state
}

fn state_to_block(state: State) -> [u8; 16] {
    let mut block = [0u8; 16];
    for (i, col) in state.iter().enumerate() {
        block[(4 * i)..(4 * i + 4)].copy_from_slice(col);
    }
    block
}

fn add_round_key(mut state: State, key: &RoundKey) -> State {
    for (col, round_col) in state.iter_mut().zip(key.iter()) {
        for (b, k) in col.iter_mut().zip(round_col.iter()) {
            *b ^= k;
        }
    }
    state
}

fn parse_key(key_hex: &str) -> Result<[u8; 16], AesError> {
    let bytes = hex_to_bytes(key_hex)?;
    if bytes.len() != 16 {
        return Err(AesError::InvalidKeyLength);
    }
    let mut key = [0u8; 16];
    key.copy_from_slice(&bytes);
    Ok(key)
}

fn hex_to_blocks(input: &str) -> Result<Vec<[u8; 16]>, AesError> {
    let bytes = hex_to_bytes(input)?;
    if bytes.len() % 16 != 0 {
        return Err(AesError::InvalidBlockLength);
    }
    Ok(bytes
        .chunks_exact(16)
        .map(|chunk| {
            let mut block = [0u8; 16];
            block.copy_from_slice(chunk);
            block
        })
        .collect())
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, AesError> {
    let bytes = hex.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err(AesError::InvalidHexLength);
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        out.push((hex_digit(pair[0])? << 4) | hex_digit(pair[1])?);
    }
    Ok(out)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(LUT[(byte >> 4) as usize] as char);
        out.push(LUT[(byte & 0x0F) as usize] as char);
    }
    out
}

fn hex_digit(byte: u8) -> Result<u8, AesError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        other => Err(AesError::InvalidHexCharacter(other as char)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_and_decrypt_matches_nist_vector() {
        let key = "2B7E151628AED2A6ABF7158809CF4F3C";
        let plaintext = "6BC1BEE22E409F96E93D7E117393172AAE2D8A571E03AC9C9EB76FAC45AF8E5130C81C46A35CE411E5FBC1191A0A52EFF69F2445DF4F9B17AD2B417BE66C3710";
        let expected_cipher = "3AD77BB40D7A3660A89ECAF32466EF97F5D3D58503B9699DE785895A96FDBAAF43B1CD7F598ECE23881B00E3ED0306887B0C785E27E8AD3F8223207104725DD4";

        let ciphertext = encrypt_hex(plaintext, key).unwrap();
        assert_eq!(ciphertext, expected_cipher);

        let decrypted = decrypt_hex(&ciphertext, key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_bytes_and_decrypt_bytes_roundtrip() {
        let key: [u8; 16] = [
            0x2B, 0x7E, 0x15, 0x16, 0x28, 0xAE, 0xD2, 0xA6, 0xAB, 0xF7, 0x15, 0x88, 0x09, 0xCF,
            0x4F, 0x3C,
        ];
        let plaintext: [u8; 16] = [
            0x6B, 0xC1, 0xBE, 0xE2, 0x2E, 0x40, 0x9F, 0x96, 0xE9, 0x3D, 0x7E, 0x11, 0x73, 0x93,
            0x17, 0x2A,
        ];
        let expected_ciphertext: [u8; 16] = [
            0x3A, 0xD7, 0x7B, 0xB4, 0x0D, 0x7A, 0x36, 0x60, 0xA8, 0x9E, 0xCA, 0xF3, 0x24, 0x66,
            0xEF, 0x97,
        ];

        let ciphertext = encrypt_bytes(&plaintext, &key).unwrap();
        assert_eq!(ciphertext, expected_ciphertext);

        let recovered = decrypt_bytes(&ciphertext, &key).unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn rejects_invalid_input_lengths() {
        let key = "2B7E151628AED2A6ABF7158809CF4F3C";
        assert_eq!(encrypt_hex("0A", key), Err(AesError::InvalidBlockLength));
        assert_eq!(encrypt_hex("0A0", key), Err(AesError::InvalidHexLength));
        assert_eq!(encrypt_hex("0011", "AB"), Err(AesError::InvalidKeyLength));
        assert_eq!(
            encrypt_bytes(&[0u8; 3], &[0u8; 16]),
            Err(AesError::InvalidBlockLength)
        );
    }
}
