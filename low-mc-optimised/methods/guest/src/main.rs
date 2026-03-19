#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use lowmc_core::{block_from_bytes, block_to_bytes, Block, BlockBytes, LowMc, ROUNDS};
use risc0_zkvm::guest::{entry, env};
use serde::{Deserialize, Serialize};

entry!(main);

#[derive(Debug, Serialize, Deserialize)]
pub struct LowMcTestSpec {
    pub rounds: u32,
    pub plaintext: BlockBytes,
    pub expected_cipher: BlockBytes,
    pub lin_matrix_columns: Vec<Block>,
    pub inv_lin_matrix_columns: Vec<Block>,
    pub round_constants: Vec<Block>,
    pub round_keys: Vec<Block>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LowMcTestResult {
    pub ciphertext: BlockBytes,
}

pub fn main() {
    let spec: LowMcTestSpec = env::read();
    let rounds = spec.rounds as usize;
    assert!(rounds > 0 && rounds <= ROUNDS, "invalid round count");

    let plaintext = block_from_bytes(&spec.plaintext);

    let cipher = LowMc::from_precomputed(
        spec.lin_matrix_columns,
        spec.inv_lin_matrix_columns,
        spec.round_constants,
        spec.round_keys,
    );
    let ciphertext = cipher.encrypt_rounds(&plaintext, rounds);

    if rounds == ROUNDS {
        let decrypted = cipher.decrypt(&ciphertext);
        assert!(
            decrypted == plaintext,
            "decrypt(encrypt(plaintext)) did not recover plaintext"
        );
    }

    let ciphertext_bytes = block_to_bytes(&ciphertext);
    assert!(
        ciphertext_bytes == spec.expected_cipher,
        "ciphertext mismatch"
    );

    env::commit(&LowMcTestResult {
        ciphertext: ciphertext_bytes,
    });
}
