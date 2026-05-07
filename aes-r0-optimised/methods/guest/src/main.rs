#![no_std]
#![no_main]

extern crate alloc;

use aesencryption::{encrypt_bytes, AES_KEY_LEN};
use alloc::format;
use alloc::vec::Vec;
use risc0_zkvm::guest::{entry, env};
use serde::{Deserialize, Serialize};

entry!(main);

/// Guest input for a single AES encryption check.
#[derive(Debug, Serialize, Deserialize)]
pub struct AesTestSpec {
    pub plaintext: Vec<u8>,
    pub key: [u8; AES_KEY_LEN],
    pub expected_ciphertext: Vec<u8>,
}

/// Guest output containing the computed ciphertext.
#[derive(Debug, Serialize, Deserialize)]
pub struct AesTestResult {
    pub ciphertext: Vec<u8>,
}

/// Runs AES encryption in the guest.
pub fn main() {
    log_stage("reading input spec");
    let spec: AesTestSpec = env::read();

    log_stage("encrypting plaintext");
    let ciphertext = encrypt_payload(&spec.plaintext, &spec.key);

    assert!(
        ciphertext == spec.expected_ciphertext,
        "ciphertext mismatch: expected {:?}, got {:?}",
        spec.expected_ciphertext,
        ciphertext
    );

    log_stage("committing ciphertext");
    env::commit(&AesTestResult { ciphertext });
}

/// Encrypts a block-aligned AES payload.
fn encrypt_payload(plaintext: &[u8], key: &[u8; AES_KEY_LEN]) -> Vec<u8> {
    encrypt_bytes(plaintext, key).expect("AES encryption failed")
}

/// Emits a cycle-count-based timestamp from inside the guest.
fn log_stage(stage: &str) {
    let cycle = env::cycle_count();
    env::log(&format!("[guest][aes-r0-optimised][cycle={cycle}] {stage}"));
}
