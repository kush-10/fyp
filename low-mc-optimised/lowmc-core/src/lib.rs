#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

pub const NUM_SBOXES: usize = 49;
pub const BLOCK_SIZE: usize = 256;
pub const KEY_SIZE: usize = 80;
pub const ROUNDS: usize = 12;

const MATRIX_WORDS: usize = ROUNDS * BLOCK_SIZE;

#[cfg(test)]
const NON_LINEAR_BITS: usize = 3 * NUM_SBOXES;

const SBOX_LANE0_MASK: Block = [
    0x9249_2492_4924_9249,
    0x4924_9249_2492_4924,
    0x0000_0000_0001_2492,
    0x0000_0000_0000_0000,
];

const SBOX_LANE1_MASK: Block = [
    0x2492_4924_9249_2492,
    0x9249_2492_4924_9249,
    0x0000_0000_0002_4924,
    0x0000_0000_0000_0000,
];

const SBOX_LANE2_MASK: Block = [
    0x4924_9249_2492_4924,
    0x2492_4924_9249_2492,
    0x0000_0000_0004_9249,
    0x0000_0000_0000_0000,
];

const NON_LINEAR_MASK: Block = [
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0x0000_0000_0007_FFFF,
    0x0000_0000_0000_0000,
];

const LINEAR_TAIL_MASK: Block = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0000,
    0xFFFF_FFFF_FFF8_0000,
    0xFFFF_FFFF_FFFF_FFFF,
];

pub type Block = [u64; 4];
pub type KeyBlock = [u64; 2];
pub type BlockBytes = [u8; 32];
pub type KeyBytes = [u8; 10];

pub struct LowMc {
    lin_matrix_columns: Vec<Block>,
    inv_lin_matrix_columns: Vec<Block>,
    round_constants: Vec<Block>,
    round_keys: Vec<Block>,
}

impl LowMc {
    pub fn new(key: KeyBlock) -> Self {
        let mut generator = GrainSsg::new();
        let mut lin_matrix_columns = Vec::with_capacity(MATRIX_WORDS);
        let mut inv_lin_matrix_columns = Vec::with_capacity(MATRIX_WORDS);

        for _ in 0..ROUNDS {
            let matrix_rows = instantiate_block_matrix(&mut generator);
            let inverse_rows = invert_block_matrix(&matrix_rows);
            append_transposed_block_matrix(&matrix_rows, &mut lin_matrix_columns);
            append_transposed_block_matrix(&inverse_rows, &mut inv_lin_matrix_columns);
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
            lin_matrix_columns,
            inv_lin_matrix_columns,
            round_constants,
            round_keys,
        }
    }

    pub fn from_precomputed(
        lin_matrix_columns: Vec<Block>,
        inv_lin_matrix_columns: Vec<Block>,
        round_constants: Vec<Block>,
        round_keys: Vec<Block>,
    ) -> Self {
        assert!(
            lin_matrix_columns.len() == MATRIX_WORDS,
            "invalid linear matrix data"
        );
        assert!(
            inv_lin_matrix_columns.len() == MATRIX_WORDS,
            "invalid inverse linear matrix data"
        );
        assert!(
            round_constants.len() == ROUNDS,
            "invalid round constant count"
        );
        assert!(round_keys.len() == ROUNDS + 1, "invalid round key count");

        Self {
            lin_matrix_columns,
            inv_lin_matrix_columns,
            round_constants,
            round_keys,
        }
    }

    pub fn precomputed_data(&self) -> (Vec<Block>, Vec<Block>, Vec<Block>, Vec<Block>) {
        (
            self.lin_matrix_columns.clone(),
            self.inv_lin_matrix_columns.clone(),
            self.round_constants.clone(),
            self.round_keys.clone(),
        )
    }

    pub fn encrypt(&self, message: &Block) -> Block {
        self.encrypt_rounds(message, ROUNDS)
    }

    pub fn encrypt_one_round(&self, message: &Block) -> Block {
        self.encrypt_rounds(message, 1)
    }

    pub fn encrypt_rounds(&self, message: &Block, rounds: usize) -> Block {
        assert!(
            rounds <= ROUNDS,
            "requested round count exceeds LowMC parameter set"
        );

        let mut c = xor_block(message, &self.round_keys[0]);
        for r in 0..rounds {
            c = substitution(&c);
            c = multiply_block_matrix_columns(
                Self::matrix_columns_for_round(&self.lin_matrix_columns, r),
                &c,
            );
            c = xor_block(&c, &self.round_constants[r]);
            c = xor_block(&c, &self.round_keys[r + 1]);
        }
        c
    }

    pub fn decrypt(&self, message: &Block) -> Block {
        let mut c = *message;
        for r in (0..ROUNDS).rev() {
            c = xor_block(&c, &self.round_keys[r + 1]);
            c = xor_block(&c, &self.round_constants[r]);
            c = multiply_block_matrix_columns(
                Self::matrix_columns_for_round(&self.inv_lin_matrix_columns, r),
                &c,
            );
            c = inv_substitution(&c);
        }
        xor_block(&c, &self.round_keys[0])
    }

    pub fn round_key_count(&self) -> usize {
        self.round_keys.len()
    }

    #[inline(always)]
    fn matrix_columns_for_round(matrix_columns: &[Block], round: usize) -> &[Block] {
        let start = round * BLOCK_SIZE;
        &matrix_columns[start..(start + BLOCK_SIZE)]
    }
}

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
    let x0 = block_and(message, &SBOX_LANE0_MASK);
    let x1 = block_shift_right(&block_and(message, &SBOX_LANE1_MASK), 1);
    let x2 = block_shift_right(&block_and(message, &SBOX_LANE2_MASK), 2);

    let x1x2 = block_and(&x1, &x2);
    let x0x2 = block_and(&x0, &x2);
    let x0x1 = block_and(&x0, &x1);

    let y0 = xor_block(&xor_block(&x0, &x1), &xor_block(&x2, &x1x2));
    let y1 = xor_block(&xor_block(&x1, &x2), &x0x2);
    let y2 = xor_block(&x2, &x0x1);

    let nonlinear = block_and(
        &block_or(
            &y0,
            &block_or(&block_shift_left(&y1, 1), &block_shift_left(&y2, 2)),
        ),
        &NON_LINEAR_MASK,
    );

    block_or(&block_and(message, &LINEAR_TAIL_MASK), &nonlinear)
}

#[inline(always)]
pub fn substitution_layer_bitslice(message: &Block) -> Block {
    substitution(message)
}

fn inv_substitution(message: &Block) -> Block {
    let x0 = block_and(message, &SBOX_LANE0_MASK);
    let x1 = block_shift_right(&block_and(message, &SBOX_LANE1_MASK), 1);
    let x2 = block_shift_right(&block_and(message, &SBOX_LANE2_MASK), 2);

    let x1x2 = block_and(&x1, &x2);
    let x0x2 = block_and(&x0, &x2);
    let x0x1 = block_and(&x0, &x1);

    let y0 = xor_block(&xor_block(&x0, &x1), &xor_block(&x2, &x1x2));
    let y1 = xor_block(&x1, &x0x2);
    let y2 = xor_block(&xor_block(&x1, &x2), &x0x1);

    let nonlinear = block_and(
        &block_or(
            &y0,
            &block_or(&block_shift_left(&y1, 1), &block_shift_left(&y2, 2)),
        ),
        &NON_LINEAR_MASK,
    );

    block_or(&block_and(message, &LINEAR_TAIL_MASK), &nonlinear)
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

fn append_transposed_block_matrix(matrix_rows: &[Block], matrix_columns: &mut Vec<Block>) {
    let base = matrix_columns.len();
    matrix_columns.resize(base + BLOCK_SIZE, [0u64; 4]);

    for (row_idx, row) in matrix_rows.iter().enumerate() {
        let row_word = row_idx >> 6;
        let row_bit = row_idx & 63;
        let row_mask = 1u64 << row_bit;

        for (word_idx, word) in row.iter().copied().enumerate() {
            let mut bits = word;
            while bits != 0 {
                let bit = bits.trailing_zeros() as usize;
                let col_idx = base + (word_idx << 6) + bit;
                matrix_columns[col_idx][row_word] |= row_mask;
                bits &= bits - 1;
            }
        }
    }
}

#[inline(always)]
fn multiply_block_matrix_columns(matrix_columns: &[Block], message: &Block) -> Block {
    debug_assert!(matrix_columns.len() == BLOCK_SIZE);
    let mut out = [0u64; 4];

    for (word_idx, word) in message.iter().copied().enumerate() {
        let mut bits = word;
        while bits != 0 {
            let bit = bits.trailing_zeros() as usize;
            let column = &matrix_columns[(word_idx << 6) + bit];
            out[0] ^= column[0];
            out[1] ^= column[1];
            out[2] ^= column[2];
            out[3] ^= column[3];
            bits &= bits - 1;
        }
    }

    out
}

fn multiply_key_matrix(matrix: &[KeyBlock], key: &KeyBlock) -> Block {
    let k0 = key[0];
    let k1 = key[1];
    let mut out = [0u64; 4];

    for (row_idx, row) in matrix.iter().enumerate() {
        let dot = (row[0] & k0) ^ (row[1] & k1);
        let parity = parity_u64(dot);
        out[row_idx >> 6] |= parity << (row_idx & 63);
    }

    out
}

#[inline(always)]
fn parity_u64(mut x: u64) -> u64 {
    x ^= x >> 32;
    x ^= x >> 16;
    x ^= x >> 8;
    x ^= x >> 4;
    x ^= x >> 2;
    x ^= x >> 1;
    x & 1
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

#[inline(always)]
fn xor_block(lhs: &Block, rhs: &Block) -> Block {
    [
        lhs[0] ^ rhs[0],
        lhs[1] ^ rhs[1],
        lhs[2] ^ rhs[2],
        lhs[3] ^ rhs[3],
    ]
}

#[inline(always)]
fn block_and(lhs: &Block, rhs: &Block) -> Block {
    [
        lhs[0] & rhs[0],
        lhs[1] & rhs[1],
        lhs[2] & rhs[2],
        lhs[3] & rhs[3],
    ]
}

#[inline(always)]
fn block_or(lhs: &Block, rhs: &Block) -> Block {
    [
        lhs[0] | rhs[0],
        lhs[1] | rhs[1],
        lhs[2] | rhs[2],
        lhs[3] | rhs[3],
    ]
}

#[inline(always)]
fn block_shift_right(block: &Block, amount: u32) -> Block {
    debug_assert!(amount < 64);
    if amount == 0 {
        return *block;
    }

    [
        (block[0] >> amount) | (block[1] << (64 - amount)),
        (block[1] >> amount) | (block[2] << (64 - amount)),
        (block[2] >> amount) | (block[3] << (64 - amount)),
        block[3] >> amount,
    ]
}

#[inline(always)]
fn block_shift_left(block: &Block, amount: u32) -> Block {
    debug_assert!(amount < 64);
    if amount == 0 {
        return *block;
    }

    [
        block[0] << amount,
        (block[1] << amount) | (block[0] >> (64 - amount)),
        (block[2] << amount) | (block[1] >> (64 - amount)),
        (block[3] << amount) | (block[2] >> (64 - amount)),
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

    const SBOX: [u8; 8] = [0x00, 0x01, 0x03, 0x06, 0x07, 0x04, 0x05, 0x02];
    const INV_SBOX: [u8; 8] = [0x00, 0x01, 0x07, 0x02, 0x05, 0x06, 0x03, 0x04];

    #[test]
    fn vectorized_sbox_matches_reference_tables() {
        let mut state = [0u64; 4];

        for sbox_idx in 0..NUM_SBOXES {
            let input = (sbox_idx % 8) as u8;
            let base = 3 * sbox_idx;
            set_block_bit(&mut state, base, (input & 1) as u64);
            set_block_bit(&mut state, base + 1, ((input >> 1) & 1) as u64);
            set_block_bit(&mut state, base + 2, ((input >> 2) & 1) as u64);
        }

        // Make sure linear tail bits are preserved by the substitution layer.
        set_block_bit(&mut state, NON_LINEAR_BITS + 5, 1);
        set_block_bit(&mut state, BLOCK_SIZE - 1, 1);

        let substituted = substitution(&state);
        let recovered = inv_substitution(&substituted);

        assert_eq!(recovered, state);

        for sbox_idx in 0..NUM_SBOXES {
            let input = (sbox_idx % 8) as u8;
            let base = 3 * sbox_idx;

            let forward = (get_block_bit(&substituted, base)
                | (get_block_bit(&substituted, base + 1) << 1)
                | (get_block_bit(&substituted, base + 2) << 2)) as u8;
            assert_eq!(forward, SBOX[input as usize]);

            let inverse = (get_block_bit(&recovered, base)
                | (get_block_bit(&recovered, base + 1) << 1)
                | (get_block_bit(&recovered, base + 2) << 2)) as u8;
            assert_eq!(inverse, input);

            let inv_ref = INV_SBOX[SBOX[input as usize] as usize];
            assert_eq!(inv_ref, input);
        }
    }

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

    #[test]
    fn precomputed_columns_roundtrip_matches_encrypt_decrypt() {
        let mut key_bytes = [0u8; 10];
        key_bytes[9] = 0x01;
        let key = key_from_bytes(&key_bytes);

        let mut plaintext_bytes = [0u8; 32];
        plaintext_bytes[30] = 0xFF;
        plaintext_bytes[31] = 0xD5;
        let plaintext = block_from_bytes(&plaintext_bytes);

        let cipher = LowMc::new(key);
        let expected_ciphertext = cipher.encrypt(&plaintext);

        let (lin_matrix_columns, inv_lin_matrix_columns, round_constants, round_keys) =
            cipher.precomputed_data();

        let reconstructed = LowMc::from_precomputed(
            lin_matrix_columns,
            inv_lin_matrix_columns,
            round_constants,
            round_keys,
        );

        let ciphertext = reconstructed.encrypt(&plaintext);
        let recovered = reconstructed.decrypt(&ciphertext);

        assert_eq!(ciphertext, expected_ciphertext);
        assert_eq!(recovered, plaintext);
    }
}
