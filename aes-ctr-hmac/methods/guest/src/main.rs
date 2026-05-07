#![no_std]
#![no_main]

extern crate alloc;

use aesencryption::{
    encrypt_then_mac, verify_then_decrypt, AES_KEY_LEN, HMAC_SHA256_192_TAG_LEN,
    HMAC_SHA256_KEY_LEN,
};
use alloc::format;
use alloc::vec::Vec;
use risc0_zkvm::guest::{entry, env};
use serde::{Deserialize, Serialize};

entry!(main);

/// Guest input for an AES-CTR+HMAC encryption check.
#[derive(Debug, Serialize, Deserialize)]
pub struct AesCtrSpec {
    pub plaintext: Vec<u8>,
    pub enc_key: [u8; AES_KEY_LEN],
    pub mac_key: [u8; HMAC_SHA256_KEY_LEN],
    pub iv: [u8; 16],
    pub aad: Vec<u8>,
    pub expected_ciphertext: Vec<u8>,
    pub expected_tag: [u8; HMAC_SHA256_192_TAG_LEN],
}

/// Guest output containing the computed ciphertext and auth tag.
#[derive(Debug, Serialize, Deserialize)]
pub struct AesCtrResult {
    pub ciphertext: Vec<u8>,
    pub tag: [u8; HMAC_SHA256_192_TAG_LEN],
}

/// Runs AES-CTR encryption and HMAC-SHA256 authentication in the guest.
pub fn main() {
    log_stage("reading input spec");
    let spec: AesCtrSpec = env::read();

    let num_blocks = spec.plaintext.len() / 16;
    log_stage(&format!("encrypting {num_blocks} blocks in ctr+hmac mode"));

    let (ciphertext, tag) = encrypt_then_mac(
        &spec.plaintext,
        &spec.enc_key,
        &spec.mac_key,
        &spec.iv,
        &spec.aad,
    );

    let recovered = verify_then_decrypt(
        &ciphertext,
        &spec.enc_key,
        &spec.mac_key,
        &spec.iv,
        &spec.aad,
        &tag,
    )
    .expect("authentication failed in guest");

    assert!(
        ciphertext == spec.expected_ciphertext,
        "ciphertext mismatch: expected {:?}, got {:?}",
        spec.expected_ciphertext,
        ciphertext
    );

    assert!(tag == spec.expected_tag, "tag mismatch");
    assert!(
        recovered == spec.plaintext,
        "verify_then_decrypt did not recover original plaintext"
    );

    log_stage("committing ciphertext and tag");
    env::commit(&AesCtrResult { ciphertext, tag });
}

/// Emits a cycle-count-based timestamp from inside the guest.
fn log_stage(stage: &str) {
    let cycle = env::cycle_count();
    env::log(&format!("[guest][aes-ctr-hmac][cycle={cycle}] {stage}"));
}
