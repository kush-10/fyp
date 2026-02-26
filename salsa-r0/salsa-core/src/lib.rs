#![no_std]

extern crate alloc;

use alloc::vec::Vec;

const SIGMA: [u8; 16] = *b"expand 32-byte k";

pub fn salsa20_encrypt_manual(plaintext: &[u8], key: &[u8; 32], nonce: &[u8; 8]) -> Vec<u8> {
    let mut counter = 0u64;
    let mut output = Vec::with_capacity(plaintext.len());

    for chunk in plaintext.chunks(64) {
        let block = salsa20_block(key, nonce, counter);
        for (i, byte) in chunk.iter().enumerate() {
            output.push(*byte ^ block[i]);
        }
        counter = counter.wrapping_add(1);
    }

    output
}

fn salsa20_block(key: &[u8; 32], nonce: &[u8; 8], counter: u64) -> [u8; 64] {
    let mut state = [0u32; 16];

    state[0] = read_u32_le(&SIGMA, 0);
    state[5] = read_u32_le(&SIGMA, 4);
    state[10] = read_u32_le(&SIGMA, 8);
    state[15] = read_u32_le(&SIGMA, 12);

    state[1] = read_u32_le(key, 0);
    state[2] = read_u32_le(key, 4);
    state[3] = read_u32_le(key, 8);
    state[4] = read_u32_le(key, 12);
    state[11] = read_u32_le(key, 16);
    state[12] = read_u32_le(key, 20);
    state[13] = read_u32_le(key, 24);
    state[14] = read_u32_le(key, 28);

    state[6] = read_u32_le(nonce, 0);
    state[7] = read_u32_le(nonce, 4);
    state[8] = counter as u32;
    state[9] = (counter >> 32) as u32;

    let mut working = state;

    for _ in 0..10 {
        quarter_round(&mut working, 0, 4, 8, 12);
        quarter_round(&mut working, 5, 9, 13, 1);
        quarter_round(&mut working, 10, 14, 2, 6);
        quarter_round(&mut working, 15, 3, 7, 11);

        quarter_round(&mut working, 0, 1, 2, 3);
        quarter_round(&mut working, 5, 6, 7, 4);
        quarter_round(&mut working, 10, 11, 8, 9);
        quarter_round(&mut working, 15, 12, 13, 14);
    }

    for i in 0..16 {
        working[i] = working[i].wrapping_add(state[i]);
    }

    let mut out = [0u8; 64];
    for (i, word) in working.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_le_bytes());
    }

    out
}

fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[b] ^= state[a].wrapping_add(state[d]).rotate_left(7);
    state[c] ^= state[b].wrapping_add(state[a]).rotate_left(9);
    state[d] ^= state[c].wrapping_add(state[b]).rotate_left(13);
    state[a] ^= state[d].wrapping_add(state[c]).rotate_left(18);
}

fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::salsa20_encrypt_manual;

    const EXPECTED_KEY1_IV0: [u8; 64] = [
        0xE3, 0xBE, 0x8F, 0xDD, 0x8B, 0xEC, 0xA2, 0xE3, 0xEA, 0x8E, 0xF9, 0x47, 0x5B, 0x29, 0xA6,
        0xE7, 0x00, 0x39, 0x51, 0xE1, 0x09, 0x7A, 0x5C, 0x38, 0xD2, 0x3B, 0x7A, 0x5F, 0xAD, 0x9F,
        0x68, 0x44, 0xB2, 0x2C, 0x97, 0x55, 0x9E, 0x27, 0x23, 0xC7, 0xCB, 0xBD, 0x3F, 0xE4, 0xFC,
        0x8D, 0x9A, 0x07, 0x44, 0x65, 0x2A, 0x83, 0xE7, 0x2A, 0x9C, 0x46, 0x18, 0x76, 0xAF, 0x4D,
        0x7E, 0xF1, 0xA1, 0x17,
    ];

    #[test]
    fn known_vector_key1_iv0() {
        let mut key = [0u8; 32];
        key[0] = 0x80;
        let nonce = [0u8; 8];
        let plaintext = [0u8; 64];

        let ciphertext = salsa20_encrypt_manual(&plaintext, &key, &nonce);
        assert_eq!(ciphertext.as_slice(), EXPECTED_KEY1_IV0.as_slice());
    }
}
