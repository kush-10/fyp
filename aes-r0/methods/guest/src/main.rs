#![no_std]
#![no_main]

extern crate alloc;

use aesencryption::encrypt_bytes;
use alloc::vec::Vec;
use risc0_zkvm::guest::{entry, env};
use serde::{Deserialize, Serialize};

entry!(main);

#[derive(Debug, Serialize, Deserialize)]
pub struct AesTestSpec {
    pub plaintext: Vec<u8>,
    pub key: [u8; 16],
    pub expected_ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AesTestResult {
    pub ciphertext: Vec<u8>,
}

pub fn main() {
    let spec: AesTestSpec = env::read();

    let ciphertext = encrypt_bytes(&spec.plaintext, &spec.key).expect("AES encryption failed");

    assert!(
        ciphertext == spec.expected_ciphertext,
        "ciphertext mismatch: expected {:?}, got {:?}",
        spec.expected_ciphertext,
        ciphertext
    );

    env::commit(&AesTestResult { ciphertext });
}
