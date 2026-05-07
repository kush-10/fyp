#![cfg_attr(not(test), no_std)]

//! Small AES-192 implementation used by the AES benchmark targets.
//!
//! The crate exposes byte and hex APIs and keeps implementation details in
//! dedicated modules for SubBytes, ShiftRows, and MixColumns.

extern crate alloc;

use alloc::{string::String, vec::Vec};

mod mixcolumns;
mod shiftrows;
mod subbytes;

use mixcolumns::{inv_mix_columns, mix_columns};
use shiftrows::{inv_shift_rows, shift_rows};
use subbytes::{inv_sub_bytes, sub_bytes};

type Word = [u8; 4];
type State = [Word; 4];
type RoundKey = State;

pub const AES_KEY_LEN: usize = 24;
pub const AES_BLOCK_LEN: usize = 16;

const AES_KEY_WORDS: usize = AES_KEY_LEN / 4;
const AES_ROUNDS: usize = 12;
const AES_ROUND_KEYS: usize = AES_ROUNDS + 1;
const AES_EXPANDED_WORDS: usize = 4 * AES_ROUND_KEYS;

type KeySchedule = [RoundKey; AES_ROUND_KEYS];

/// AES input/encoding validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AesError {
    InvalidHexLength,
    InvalidKeyLength,
    InvalidBlockLength,
    InvalidHexCharacter(char),
}

/// Encrypts block-aligned hex plaintext with AES-192 ECB.
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

/// Decrypts block-aligned hex ciphertext with AES-192 ECB.
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

/// Encrypts a block-aligned byte payload with AES-192 ECB.
pub fn encrypt_bytes(plaintext: &[u8], key: &[u8; AES_KEY_LEN]) -> Result<Vec<u8>, AesError> {
    if plaintext.len() % AES_BLOCK_LEN != 0 {
        return Err(AesError::InvalidBlockLength);
    }

    let schedule = key_expansion(*key);
    let mut out = Vec::with_capacity(plaintext.len());

    for chunk in plaintext.chunks_exact(AES_BLOCK_LEN) {
        let mut block = [0u8; AES_BLOCK_LEN];
        block.copy_from_slice(chunk);
        out.extend_from_slice(&encrypt_block(block, &schedule));
    }

    Ok(out)
}

/// Decrypts a block-aligned byte payload with AES-192 ECB.
pub fn decrypt_bytes(ciphertext: &[u8], key: &[u8; AES_KEY_LEN]) -> Result<Vec<u8>, AesError> {
    if ciphertext.len() % AES_BLOCK_LEN != 0 {
        return Err(AesError::InvalidBlockLength);
    }

    let schedule = key_expansion(*key);
    let mut out = Vec::with_capacity(ciphertext.len());

    for chunk in ciphertext.chunks_exact(AES_BLOCK_LEN) {
        let mut block = [0u8; AES_BLOCK_LEN];
        block.copy_from_slice(chunk);
        out.extend_from_slice(&decrypt_block(block, &schedule));
    }

    Ok(out)
}

fn encrypt_block(block: [u8; 16], schedule: &KeySchedule) -> [u8; 16] {
    let mut state = add_round_key(block_to_state(block), &schedule[0]);

    for round in 1..AES_ROUNDS {
        state = sub_bytes(state);
        state = shift_rows(state);
        state = mix_columns(state);
        state = add_round_key(state, &schedule[round]);
    }

    state = sub_bytes(state);
    state = shift_rows(state);
    state = add_round_key(state, &schedule[AES_ROUNDS]);

    state_to_block(state)
}

fn decrypt_block(block: [u8; 16], schedule: &KeySchedule) -> [u8; 16] {
    let mut state = add_round_key(block_to_state(block), &schedule[AES_ROUNDS]);

    for round in (1..AES_ROUNDS).rev() {
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

fn key_expansion(key: [u8; AES_KEY_LEN]) -> KeySchedule {
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

    let mut w = [[0u8; 4]; AES_EXPANDED_WORDS];
    for (i, chunk) in key.chunks_exact(4).enumerate() {
        w[i].copy_from_slice(chunk);
    }

    for i in AES_KEY_WORDS..AES_EXPANDED_WORDS {
        let mut temp = w[i - 1];
        if i % AES_KEY_WORDS == 0 {
            temp = xor_words(sub_word(rot_word(temp)), RCON[(i / AES_KEY_WORDS) - 1]);
        }
        w[i] = xor_words(w[i - AES_KEY_WORDS], temp);
    }

    let mut schedule = [[[0u8; 4]; 4]; AES_ROUND_KEYS];
    for (round, chunk) in w.chunks_exact(4).enumerate() {
        for (col, word) in chunk.iter().enumerate() {
            schedule[round][col] = *word;
        }
    }
    schedule
}

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

fn parse_key(key_hex: &str) -> Result<[u8; AES_KEY_LEN], AesError> {
    let bytes = hex_to_bytes(key_hex)?;
    if bytes.len() != AES_KEY_LEN {
        return Err(AesError::InvalidKeyLength);
    }
    let mut key = [0u8; AES_KEY_LEN];
    key.copy_from_slice(&bytes);
    Ok(key)
}

fn hex_to_blocks(input: &str) -> Result<Vec<[u8; 16]>, AesError> {
    let bytes = hex_to_bytes(input)?;
    if bytes.len() % AES_BLOCK_LEN != 0 {
        return Err(AesError::InvalidBlockLength);
    }
    Ok(bytes
        .chunks_exact(AES_BLOCK_LEN)
        .map(|chunk| {
            let mut block = [0u8; AES_BLOCK_LEN];
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
        let key = "8E73B0F7DA0E6452C810F32B809079E562F8EAD2522C6B7B";
        let plaintext = "6BC1BEE22E409F96E93D7E117393172AAE2D8A571E03AC9C9EB76FAC45AF8E5130C81C46A35CE411E5FBC1191A0A52EFF69F2445DF4F9B17AD2B417BE66C3710";
        let expected_cipher = "BD334F1D6E45F25FF712A214571FA5CC974104846D0AD3AD7734ECB3ECEE4EEFEF7AFD2270E2E60ADCE0BA2FACE6444E9A4B41BA738D6C72FB16691603C18E0E";

        let ciphertext = encrypt_hex(plaintext, key).unwrap();
        assert_eq!(ciphertext, expected_cipher);

        let decrypted = decrypt_hex(&ciphertext, key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_bytes_and_decrypt_bytes_roundtrip() {
        let key: [u8; AES_KEY_LEN] = [
            0x8E, 0x73, 0xB0, 0xF7, 0xDA, 0x0E, 0x64, 0x52, 0xC8, 0x10, 0xF3, 0x2B, 0x80, 0x90,
            0x79, 0xE5, 0x62, 0xF8, 0xEA, 0xD2, 0x52, 0x2C, 0x6B, 0x7B,
        ];
        let plaintext: [u8; 16] = [
            0x6B, 0xC1, 0xBE, 0xE2, 0x2E, 0x40, 0x9F, 0x96, 0xE9, 0x3D, 0x7E, 0x11, 0x73, 0x93,
            0x17, 0x2A,
        ];
        let expected_ciphertext: [u8; 16] = [
            0xBD, 0x33, 0x4F, 0x1D, 0x6E, 0x45, 0xF2, 0x5F, 0xF7, 0x12, 0xA2, 0x14, 0x57, 0x1F,
            0xA5, 0xCC,
        ];

        let ciphertext = encrypt_bytes(&plaintext, &key).unwrap();
        assert_eq!(ciphertext, expected_ciphertext);

        let recovered = decrypt_bytes(&ciphertext, &key).unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn rejects_invalid_input_lengths() {
        let key = "8E73B0F7DA0E6452C810F32B809079E562F8EAD2522C6B7B";
        assert_eq!(encrypt_hex("0A", key), Err(AesError::InvalidBlockLength));
        assert_eq!(encrypt_hex("0A0", key), Err(AesError::InvalidHexLength));
        assert_eq!(encrypt_hex("0011", "AB"), Err(AesError::InvalidKeyLength));
        assert_eq!(
            encrypt_bytes(&[0u8; 3], &[0u8; AES_KEY_LEN]),
            Err(AesError::InvalidBlockLength)
        );
    }
}
