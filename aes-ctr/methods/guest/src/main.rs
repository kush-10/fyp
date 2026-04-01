#![no_std]
#![no_main]

extern crate alloc;

use aesencryption::encrypt_ctr;
use alloc::format;
use alloc::vec::Vec;
use risc0_zkvm::guest::{entry, env};
use serde::{Deserialize, Serialize};

entry!(main);

/// Guest input for an AES-CTR encryption check.
#[derive(Debug, Serialize, Deserialize)]
pub struct AesCtrSpec {
    pub plaintext: Vec<u8>,
    pub key: [u8; 16],
    pub iv: [u8; 16],
    pub expected_ciphertext: Vec<u8>,
}

/// Guest output containing the computed ciphertext.
#[derive(Debug, Serialize, Deserialize)]
pub struct AesCtrResult {
    pub ciphertext: Vec<u8>,
}

/// Runs AES-CTR encryption in the guest.
pub fn main() {
    log_stage("reading input spec");
    let spec: AesCtrSpec = env::read();

    let num_blocks = spec.plaintext.len() / 16;
    log_stage(&format!("encrypting {num_blocks} blocks in CTR mode"));

    let ciphertext = encrypt_ctr(&spec.plaintext, &spec.key, &spec.iv);

    assert!(
        ciphertext == spec.expected_ciphertext,
        "ciphertext mismatch: expected {:?}, got {:?}",
        spec.expected_ciphertext,
        ciphertext
    );

    log_stage("committing ciphertext");
    env::commit(&AesCtrResult { ciphertext });
}

/// Emits a cycle-count-based timestamp from inside the guest.
fn log_stage(stage: &str) {
    let cycle = env::cycle_count();
    env::log(&format!("[guest][aes-ctr][cycle={cycle}] {stage}"));
}
