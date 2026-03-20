#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use lowmc_core::{block_from_bytes, block_to_bytes, key_from_bytes, BlockBytes, KeyBytes, LowMc};
use risc0_zkvm::guest::{entry, env};
use serde::{Deserialize, Serialize};

entry!(main);

/// Guest input for a single LowMC roundtrip.
#[derive(Debug, Serialize, Deserialize)]
pub struct LowMcTestSpec {
    pub plaintext: BlockBytes,
    pub key: KeyBytes,
    pub expected_cipher: BlockBytes,
}

/// Guest output containing the computed ciphertext.
#[derive(Debug, Serialize, Deserialize)]
pub struct LowMcTestResult {
    pub ciphertext: BlockBytes,
}

/// Runs LowMC encryption and decryption in the guest.
pub fn main() {
    log_stage("reading input spec");
    let spec: LowMcTestSpec = env::read();

    log_stage("building cipher from key");
    let key = key_from_bytes(&spec.key);
    let cipher = LowMc::new(key);

    log_stage("encrypting plaintext");
    let ciphertext = encrypt_bytes(&cipher, &spec.plaintext);
    assert!(ciphertext == spec.expected_cipher, "ciphertext mismatch");

    log_stage("decrypting ciphertext");
    let decrypted = decrypt_bytes(&cipher, &ciphertext);
    assert!(
        decrypted == spec.plaintext,
        "decrypt(encrypt(plaintext)) did not recover plaintext"
    );

    log_stage("committing ciphertext");
    env::commit(&LowMcTestResult { ciphertext });
}

/// Encrypts one LowMC block encoded as bytes.
fn encrypt_bytes(cipher: &LowMc, plaintext_bytes: &BlockBytes) -> BlockBytes {
    let plaintext = block_from_bytes(plaintext_bytes);
    let ciphertext = cipher.encrypt(&plaintext);
    block_to_bytes(&ciphertext)
}

/// Decrypts one LowMC block encoded as bytes.
fn decrypt_bytes(cipher: &LowMc, ciphertext_bytes: &BlockBytes) -> BlockBytes {
    let ciphertext = block_from_bytes(ciphertext_bytes);
    let plaintext = cipher.decrypt(&ciphertext);
    block_to_bytes(&plaintext)
}

/// Emits a cycle-count-based timestamp from inside the guest.
fn log_stage(stage: &str) {
    let cycle = env::cycle_count();
    env::log(&format!("[guest][lowmc-r0][cycle={cycle}] {stage}"));
}
