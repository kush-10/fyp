#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use lowmc_core::{
    block_from_bytes, block_to_bytes, key_from_bytes, Block, BlockBytes, KeyBlock, KeyBytes, LowMc,
};
use risc0_zkvm::guest::{entry, env};
use serde::{Deserialize, Serialize};

entry!(main);

#[derive(Debug, Serialize, Deserialize)]
pub struct LowMcTestSpec {
    pub plaintext: BlockBytes,
    pub key: KeyBytes,
    pub expected_cipher: BlockBytes,
    pub lin_matrices: Vec<Vec<Block>>,
    pub inv_lin_matrices: Vec<Vec<Block>>,
    pub round_constants: Vec<Block>,
    pub key_matrices: Vec<Vec<KeyBlock>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LowMcTestResult {
    pub ciphertext: BlockBytes,
}

pub fn main() {
    let spec: LowMcTestSpec = env::read();

    let key = key_from_bytes(&spec.key);
    let plaintext = block_from_bytes(&spec.plaintext);

    let cipher = LowMc::from_precomputed(
        key,
        spec.lin_matrices,
        spec.inv_lin_matrices,
        spec.round_constants,
        spec.key_matrices,
    );
    let ciphertext = cipher.encrypt(&plaintext);
    let decrypted = cipher.decrypt(&ciphertext);

    assert!(
        decrypted == plaintext,
        "decrypt(encrypt(plaintext)) did not recover plaintext"
    );

    let ciphertext_bytes = block_to_bytes(&ciphertext);
    assert!(
        ciphertext_bytes == spec.expected_cipher,
        "ciphertext mismatch"
    );

    env::commit(&LowMcTestResult {
        ciphertext: ciphertext_bytes,
    });
}
