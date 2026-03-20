#![no_std]
#![no_main]

extern crate alloc;

use aesencryption::{decrypt_bytes, encrypt_bytes};
use alloc::format;
use alloc::vec::Vec;
use risc0_zkvm::guest::{entry, env};
use serde::{Deserialize, Serialize};

entry!(main);

/// Guest input for a single AES roundtrip.
#[derive(Debug, Serialize, Deserialize)]
pub struct AesTestSpec {
    pub plaintext: Vec<u8>,
    pub key: [u8; 16],
    pub expected_ciphertext: Vec<u8>,
}

/// Guest output containing the computed ciphertext.
#[derive(Debug, Serialize, Deserialize)]
pub struct AesTestResult {
    pub ciphertext: Vec<u8>,
}

/// Runs AES encryption and decryption in the guest.
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

    log_stage("decrypting ciphertext");
    let decrypted = decrypt_payload(&ciphertext, &spec.key);
    assert!(
        decrypted == spec.plaintext,
        "decrypt(encrypt(plaintext)) did not recover plaintext"
    );

    log_stage("committing ciphertext");
    env::commit(&AesTestResult { ciphertext });
}

/// Encrypts a block-aligned AES payload.
fn encrypt_payload(plaintext: &[u8], key: &[u8; 16]) -> Vec<u8> {
    encrypt_bytes(plaintext, key).expect("AES encryption failed")
}

/// Decrypts a block-aligned AES payload.
fn decrypt_payload(ciphertext: &[u8], key: &[u8; 16]) -> Vec<u8> {
    decrypt_bytes(ciphertext, key).expect("AES decryption failed")
}

/// Emits a cycle-count-based timestamp from inside the guest.
fn log_stage(stage: &str) {
    let cycle = env::cycle_count();
    env::log(&format!("[guest][aes-r0][cycle={cycle}] {stage}"));
}
