#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use risc0_zkvm::guest::{entry, env};
use salsa_core::salsa20_encrypt_manual;
use serde::{Deserialize, Serialize};

entry!(main);

#[derive(Debug, Serialize, Deserialize)]
pub struct SalsaTestSpec {
    pub plaintext: Vec<u8>,
    pub key: [u8; 32],
    pub nonce: [u8; 8],
    pub expected_ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalsaTestResult {
    pub ciphertext: Vec<u8>,
}

pub fn main() {
    let spec: SalsaTestSpec = env::read();

    let ciphertext = salsa20_encrypt_manual(&spec.plaintext, &spec.key, &spec.nonce);

    assert!(
        ciphertext == spec.expected_ciphertext,
        "ciphertext mismatch"
    );

    env::commit(&SalsaTestResult { ciphertext });
}
