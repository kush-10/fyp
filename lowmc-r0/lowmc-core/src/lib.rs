#![no_std]

//! Baseline LowMC implementation used by the `lowmc-r0` benchmark target.
//!
//! This crate intentionally keeps the reference-style structure simple so it can
//! act as the comparison point against `lowmc-r0-optimised`.

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

/// Number of active 3-bit S-boxes in the LowMC parameter set.
pub const NUM_SBOXES: usize = 49;
/// Block size in bits.
pub const BLOCK_SIZE: usize = 256;
/// Key size in bits.
pub const KEY_SIZE: usize = 80;
/// Number of encryption rounds.
pub const ROUNDS: usize = 12;

const NON_LINEAR_BITS: usize = 3 * NUM_SBOXES;

const SBOX: [u8; 8] = [0x00, 0x01, 0x03, 0x06, 0x07, 0x04, 0x05, 0x02];
const INV_SBOX: [u8; 8] = [0x00, 0x01, 0x07, 0x02, 0x05, 0x06, 0x03, 0x04];

/// 256-bit block represented as four little-endian words.
pub type Block = [u64; 4];
/// 80-bit key represented as two words.
pub type KeyBlock = [u64; 2];
/// Network-style byte representation of a block.
pub type BlockBytes = [u8; 32];
/// Network-style byte representation of a key.
pub type KeyBytes = [u8; 10];

pub struct LowMc {
    lin_matrices: Vec<Vec<Block>>,
    inv_lin_matrices: Vec<Vec<Block>>,
    round_constants: Vec<Block>,
    round_keys: Vec<Block>,
}

impl LowMc {
    /// Builds a new LowMC instance from a key, generating all matrices and
    /// round constants from the Grain self-shrinking generator.
    pub fn new(key: KeyBlock) -> Self {
        let mut generator = GrainSsg::new();
        let mut lin_matrices = Vec::with_capacity(ROUNDS);
        let mut inv_lin_matrices = Vec::with_capacity(ROUNDS);

        for _ in 0..ROUNDS {
            let matrix = instantiate_block_matrix(&mut generator);
            let inverse = invert_block_matrix(&matrix);
            lin_matrices.push(matrix);
            inv_lin_matrices.push(inverse);
        }

        let mut round_constants = Vec::with_capacity(ROUNDS);
        for _ in 0..ROUNDS {
            round_constants.push(rand_block(&mut generator));
        }

        let mut key_matrices = Vec::with_capacity(ROUNDS + 1);
        for _ in 0..=ROUNDS {
            key_matrices.push(instantiate_key_matrix(&mut generator));
        }

        let mut round_keys = Vec::with_capacity(ROUNDS + 1);
        for matrix in &key_matrices {
            round_keys.push(multiply_key_matrix(matrix, &key));
        }

        Self {
            lin_matrices,
            inv_lin_matrices,
            round_constants,
            round_keys,
        }
    }

    /// Encrypts a single 256-bit LowMC block.
    pub fn encrypt(&self, message: &Block) -> Block {
        let mut c = xor_block(message, &self.round_keys[0]);
        for r in 0..ROUNDS {
            c = substitution(&c);
            c = multiply_block_matrix(&self.lin_matrices[r], &c);
            c = xor_block(&c, &self.round_constants[r]);
            c = xor_block(&c, &self.round_keys[r + 1]);
        }
        c
    }

    /// Decrypts a single 256-bit LowMC block.
    pub fn decrypt(&self, message: &Block) -> Block {
        let mut c = *message;
        for r in (0..ROUNDS).rev() {
            c = xor_block(&c, &self.round_keys[r + 1]);
            c = xor_block(&c, &self.round_constants[r]);
            c = multiply_block_matrix(&self.inv_lin_matrices[r], &c);
            c = inv_substitution(&c);
        }
        xor_block(&c, &self.round_keys[0])
    }

    /// Returns the number of round keys, including whitening key.
    pub fn round_key_count(&self) -> usize {
        self.round_keys.len()
    }
}

/// Converts bytes into the internal block bit ordering.
pub fn block_from_bytes(bytes: &BlockBytes) -> Block {
    let mut out = [0u64; 4];
    for bit in 0..BLOCK_SIZE {
        let src_byte = bit / 8;
        let src_offset = 7 - (bit % 8);
        let value = ((bytes[src_byte] >> src_offset) & 1) as u64;
        let dst_bit = BLOCK_SIZE - 1 - bit;
        set_block_bit(&mut out, dst_bit, value);
    }
    out
}

/// Converts an internal block into external byte ordering.
pub fn block_to_bytes(block: &Block) -> BlockBytes {
    let mut out = [0u8; 32];
    for bit in 0..BLOCK_SIZE {
        let src_bit = BLOCK_SIZE - 1 - bit;
        let value = get_block_bit(block, src_bit) as u8;
        let dst_byte = bit / 8;
        let dst_offset = 7 - (bit % 8);
        out[dst_byte] |= value << dst_offset;
    }
    out
}

/// Converts bytes into the internal key bit ordering.
pub fn key_from_bytes(bytes: &KeyBytes) -> KeyBlock {
    let mut out = [0u64; 2];
    for bit in 0..KEY_SIZE {
        let src_byte = bit / 8;
        let src_offset = 7 - (bit % 8);
        let value = ((bytes[src_byte] >> src_offset) & 1) as u64;
        let dst_bit = KEY_SIZE - 1 - bit;
        set_key_bit(&mut out, dst_bit, value);
    }
    out
}

/// Converts an internal key into external byte ordering.
pub fn key_to_bytes(key: &KeyBlock) -> KeyBytes {
    let mut out = [0u8; 10];
    for bit in 0..KEY_SIZE {
        let src_bit = KEY_SIZE - 1 - bit;
        let value = get_key_bit(key, src_bit) as u8;
        let dst_byte = bit / 8;
        let dst_offset = 7 - (bit % 8);
        out[dst_byte] |= value << dst_offset;
    }
    out
}

fn substitution(message: &Block) -> Block {
    let mut out = *message;
    for sbox_idx in 0..(NON_LINEAR_BITS / 3) {
        let base = 3 * sbox_idx;
        let in_value = get_triple(message, base);
        set_triple(&mut out, base, SBOX[in_value as usize]);
    }
    out
}

fn inv_substitution(message: &Block) -> Block {
    let mut out = *message;
    for sbox_idx in 0..(NON_LINEAR_BITS / 3) {
        let base = 3 * sbox_idx;
        let in_value = get_triple(message, base);
        set_triple(&mut out, base, INV_SBOX[in_value as usize]);
    }
    out
}

fn get_triple(message: &Block, bit_offset: usize) -> u8 {
    (get_block_bit(message, bit_offset)
        | (get_block_bit(message, bit_offset + 1) << 1)
        | (get_block_bit(message, bit_offset + 2) << 2)) as u8
}

fn set_triple(message: &mut Block, bit_offset: usize, value: u8) {
    set_block_bit(message, bit_offset, (value & 1) as u64);
    set_block_bit(message, bit_offset + 1, ((value >> 1) & 1) as u64);
    set_block_bit(message, bit_offset + 2, ((value >> 2) & 1) as u64);
}

fn instantiate_block_matrix(generator: &mut GrainSsg) -> Vec<Block> {
    loop {
        let mut matrix = Vec::with_capacity(BLOCK_SIZE);
        for _ in 0..BLOCK_SIZE {
            matrix.push(rand_block(generator));
        }
        if rank_block_matrix(&matrix) == BLOCK_SIZE {
            return matrix;
        }
    }
}

fn instantiate_key_matrix(generator: &mut GrainSsg) -> Vec<KeyBlock> {
    loop {
        let mut matrix = Vec::with_capacity(BLOCK_SIZE);
        for _ in 0..BLOCK_SIZE {
            matrix.push(rand_key_block(generator));
        }
        if rank_key_matrix(&matrix) >= KEY_SIZE {
            return matrix;
        }
    }
}

fn rank_block_matrix(matrix: &[Block]) -> usize {
    let mut mat = matrix.to_vec();
    let mut row = 0usize;
    for col in 0..BLOCK_SIZE {
        let mut pivot = row;
        while pivot < mat.len() && get_block_bit(&mat[pivot], col) == 0 {
            pivot += 1;
        }
        if pivot == mat.len() {
            continue;
        }

        mat.swap(row, pivot);
        let pivot_row = mat[row];

        for r in (row + 1)..mat.len() {
            if get_block_bit(&mat[r], col) == 1 {
                xor_block_in_place(&mut mat[r], &pivot_row);
            }
        }

        row += 1;
        if row == BLOCK_SIZE {
            break;
        }
    }
    row
}

fn rank_key_matrix(matrix: &[KeyBlock]) -> usize {
    let mut mat = matrix.to_vec();
    let mut row = 0usize;
    for col in 0..KEY_SIZE {
        let mut pivot = row;
        while pivot < mat.len() && get_key_bit(&mat[pivot], col) == 0 {
            pivot += 1;
        }
        if pivot == mat.len() {
            continue;
        }

        mat.swap(row, pivot);
        let pivot_row = mat[row];

        for r in (row + 1)..mat.len() {
            if get_key_bit(&mat[r], col) == 1 {
                xor_key_in_place(&mut mat[r], &pivot_row);
            }
        }

        row += 1;
        if row == KEY_SIZE {
            break;
        }
    }
    row
}

fn invert_block_matrix(matrix: &[Block]) -> Vec<Block> {
    let mut mat = matrix.to_vec();
    let mut inv = vec![[0u64; 4]; BLOCK_SIZE];
    for (idx, row) in inv.iter_mut().enumerate() {
        set_block_bit(row, idx, 1);
    }

    let mut row = 0usize;
    for col in 0..BLOCK_SIZE {
        let mut pivot = row;
        while pivot < BLOCK_SIZE && get_block_bit(&mat[pivot], col) == 0 {
            pivot += 1;
        }
        if pivot == BLOCK_SIZE {
            continue;
        }

        if pivot != row {
            mat.swap(row, pivot);
            inv.swap(row, pivot);
        }

        let pivot_mat_row = mat[row];
        let pivot_inv_row = inv[row];

        for r in (row + 1)..BLOCK_SIZE {
            if get_block_bit(&mat[r], col) == 1 {
                xor_block_in_place(&mut mat[r], &pivot_mat_row);
                xor_block_in_place(&mut inv[r], &pivot_inv_row);
            }
        }
        row += 1;
    }

    for col in (0..BLOCK_SIZE).rev() {
        let pivot_mat_row = mat[col];
        let pivot_inv_row = inv[col];
        for r in 0..col {
            if get_block_bit(&mat[r], col) == 1 {
                xor_block_in_place(&mut mat[r], &pivot_mat_row);
                xor_block_in_place(&mut inv[r], &pivot_inv_row);
            }
        }
    }

    inv
}

fn multiply_block_matrix(matrix: &[Block], message: &Block) -> Block {
    let mut out = [0u64; 4];
    for (row_idx, row) in matrix.iter().enumerate() {
        let parity = ((row[0] & message[0]).count_ones()
            + (row[1] & message[1]).count_ones()
            + (row[2] & message[2]).count_ones()
            + (row[3] & message[3]).count_ones())
            & 1;
        set_block_bit(&mut out, row_idx, parity as u64);
    }
    out
}

fn multiply_key_matrix(matrix: &[KeyBlock], key: &KeyBlock) -> Block {
    let mut out = [0u64; 4];
    for (row_idx, row) in matrix.iter().enumerate() {
        let parity = ((row[0] & key[0]).count_ones() + (row[1] & key[1]).count_ones()) & 1;
        set_block_bit(&mut out, row_idx, parity as u64);
    }
    out
}

fn rand_block(generator: &mut GrainSsg) -> Block {
    let mut out = [0u64; 4];
    for bit in 0..BLOCK_SIZE {
        set_block_bit(&mut out, bit, generator.next_bit() as u64);
    }
    out
}

fn rand_key_block(generator: &mut GrainSsg) -> KeyBlock {
    let mut out = [0u64; 2];
    for bit in 0..KEY_SIZE {
        set_key_bit(&mut out, bit, generator.next_bit() as u64);
    }
    out
}

fn get_block_bit(block: &Block, bit: usize) -> u64 {
    (block[bit / 64] >> (bit % 64)) & 1
}

fn set_block_bit(block: &mut Block, bit: usize, value: u64) {
    let word = bit / 64;
    let offset = bit % 64;
    let mask = 1u64 << offset;
    if value == 0 {
        block[word] &= !mask;
    } else {
        block[word] |= mask;
    }
}

fn get_key_bit(block: &KeyBlock, bit: usize) -> u64 {
    (block[bit / 64] >> (bit % 64)) & 1
}

fn set_key_bit(block: &mut KeyBlock, bit: usize, value: u64) {
    let word = bit / 64;
    let offset = bit % 64;
    let mask = 1u64 << offset;
    if value == 0 {
        block[word] &= !mask;
    } else {
        block[word] |= mask;
    }
}

fn xor_block(lhs: &Block, rhs: &Block) -> Block {
    [
        lhs[0] ^ rhs[0],
        lhs[1] ^ rhs[1],
        lhs[2] ^ rhs[2],
        lhs[3] ^ rhs[3],
    ]
}

fn xor_block_in_place(lhs: &mut Block, rhs: &Block) {
    lhs[0] ^= rhs[0];
    lhs[1] ^= rhs[1];
    lhs[2] ^= rhs[2];
    lhs[3] ^= rhs[3];
}

fn xor_key_in_place(lhs: &mut KeyBlock, rhs: &KeyBlock) {
    lhs[0] ^= rhs[0];
    lhs[1] ^= rhs[1];
}

struct GrainSsg {
    state: u128,
    initialized: bool,
}

impl GrainSsg {
    fn new() -> Self {
        Self {
            state: 0,
            initialized: false,
        }
    }

    fn next_bit(&mut self) -> bool {
        if !self.initialized {
            self.state = (1u128 << 80) - 1;
            self.initialized = true;
            for _ in 0..160 {
                let _ = self.advance_state();
            }
        }

        loop {
            let choice = self.advance_state();
            let data = self.advance_state();
            if choice {
                return data;
            }
        }
    }

    fn advance_state(&mut self) -> bool {
        let bit = self.get_state_bit(0)
            ^ self.get_state_bit(13)
            ^ self.get_state_bit(23)
            ^ self.get_state_bit(38)
            ^ self.get_state_bit(51)
            ^ self.get_state_bit(62);
        self.state >>= 1;
        if bit {
            self.state |= 1u128 << 79;
        }
        bit
    }

    fn get_state_bit(&self, idx: usize) -> bool {
        ((self.state >> idx) & 1) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let mut key_bytes = [0u8; 10];
        key_bytes[9] = 0x01;
        let key = key_from_bytes(&key_bytes);

        let mut plaintext_bytes = [0u8; 32];
        plaintext_bytes[30] = 0xFF;
        plaintext_bytes[31] = 0xD5;
        let plaintext = block_from_bytes(&plaintext_bytes);

        let cipher = LowMc::new(key);
        let ciphertext = cipher.encrypt(&plaintext);
        let recovered = cipher.decrypt(&ciphertext);

        assert_eq!(recovered, plaintext);
        assert_eq!(cipher.round_key_count(), ROUNDS + 1);
        assert_eq!(block_to_bytes(&plaintext), plaintext_bytes);
        assert_eq!(key_to_bytes(&key), key_bytes);
        assert_eq!(NON_LINEAR_BITS, 147);
    }

    #[test]
    fn matches_reference_cpp_ciphertext() {
        let mut key_bytes = [0u8; 10];
        key_bytes[9] = 0x01;
        let key = key_from_bytes(&key_bytes);

        let mut plaintext_bytes = [0u8; 32];
        plaintext_bytes[30] = 0xFF;
        plaintext_bytes[31] = 0xD5;
        let plaintext = block_from_bytes(&plaintext_bytes);

        let expected_ciphertext: [u8; 32] = [
            0xAA, 0x2E, 0x3E, 0x6B, 0xB4, 0xAC, 0x71, 0x14, 0xB4, 0xC0, 0x2E, 0xD1, 0x3A, 0x37,
            0x0C, 0x04, 0x7C, 0x8D, 0x76, 0x42, 0x5C, 0x4C, 0xA4, 0x21, 0xDA, 0xE0, 0x2A, 0x51,
            0xF3, 0x2C, 0x07, 0x2C,
        ];

        let cipher = LowMc::new(key);
        let ciphertext = cipher.encrypt(&plaintext);

        assert_eq!(block_to_bytes(&ciphertext), expected_ciphertext);
    }
}
