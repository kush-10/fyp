#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use risc0_zkvm::guest::{entry, env};
use salsa_core::salsa20_encrypt_manual;
use serde::{Deserialize, Serialize};

entry!(main);

/// Guest input for a single Salsa20 roundtrip.
#[derive(Debug, Serialize, Deserialize)]
pub struct SalsaTestSpec {
    pub plaintext: Vec<u8>,
    pub key: [u8; 32],
    pub nonce: [u8; 8],
    pub expected_ciphertext: Vec<u8>,
}

/// Guest output containing the computed ciphertext.
#[derive(Debug, Serialize, Deserialize)]
pub struct SalsaTestResult {
    pub ciphertext: Vec<u8>,
}

/// Runs Salsa20 encryption and decryption in the guest.
pub fn main() {
    log_stage("reading input spec");
    let spec: SalsaTestSpec = env::read();

    log_stage("encrypting plaintext");
    let ciphertext = encrypt_payload(&spec.plaintext, &spec.key, &spec.nonce);

    assert!(
        ciphertext == spec.expected_ciphertext,
        "ciphertext mismatch"
    );

    log_stage("decrypting ciphertext");
    let decrypted = decrypt_payload(&ciphertext, &spec.key, &spec.nonce);
    assert!(
        decrypted == spec.plaintext,
        "decrypt(encrypt(plaintext)) did not recover plaintext"
    );

    log_stage("committing ciphertext");
    env::commit(&SalsaTestResult { ciphertext });
}

/// Encrypts a Salsa20 payload.
fn encrypt_payload(plaintext: &[u8], key: &[u8; 32], nonce: &[u8; 8]) -> Vec<u8> {
    salsa20_encrypt_manual(plaintext, key, nonce)
}

/// Decrypts a Salsa20 payload (same operation as encryption for stream ciphers).
fn decrypt_payload(ciphertext: &[u8], key: &[u8; 32], nonce: &[u8; 8]) -> Vec<u8> {
    salsa20_encrypt_manual(ciphertext, key, nonce)
}

/// Emits a cycle-count-based timestamp from inside the guest.
fn log_stage(stage: &str) {
    let cycle = env::cycle_count();
    env::log(&format!("[guest][salsa-r0][cycle={cycle}] {stage}"));
}
