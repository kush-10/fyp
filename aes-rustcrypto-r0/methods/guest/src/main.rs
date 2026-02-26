#![no_std]
#![no_main]

extern crate alloc;

use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128;
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

    let ciphertext = encrypt_blocks(&spec.plaintext, &spec.key).expect("AES encryption failed");

    assert!(
        ciphertext == spec.expected_ciphertext,
        "ciphertext mismatch: expected {:?}, got {:?}",
        spec.expected_ciphertext,
        ciphertext
    );

    env::commit(&AesTestResult { ciphertext });
}

fn encrypt_blocks(plaintext: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, &'static str> {
    if plaintext.len() % 16 != 0 {
        return Err("plaintext length must be a multiple of 16 bytes");
    }

    let cipher = Aes128::new_from_slice(key).map_err(|_| "invalid AES-128 key")?;
    let mut out = Vec::with_capacity(plaintext.len());

    for chunk in plaintext.chunks_exact(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        cipher.encrypt_block((&mut block).into());
        out.extend_from_slice(&block);
    }

    Ok(out)
}
