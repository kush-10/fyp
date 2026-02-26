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
    pub initial_counter_block: [u8; 16],
    pub expected_ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AesTestResult {
    pub ciphertext: Vec<u8>,
}

pub fn main() {
    let spec: AesTestSpec = env::read();

    let ciphertext = aes_ctr_encrypt_bytes(&spec.plaintext, &spec.key, spec.initial_counter_block)
        .expect("AES-CTR encryption failed");

    assert!(
        ciphertext == spec.expected_ciphertext,
        "ciphertext mismatch: expected {:?}, got {:?}",
        spec.expected_ciphertext,
        ciphertext
    );

    env::commit(&AesTestResult { ciphertext });
}

fn aes_ctr_encrypt_bytes(
    plaintext: &[u8],
    key: &[u8; 16],
    initial_counter_block: [u8; 16],
) -> Result<Vec<u8>, aesencryption::AesError> {
    let mut counter = initial_counter_block;
    let mut out = Vec::with_capacity(plaintext.len());

    for chunk in plaintext.chunks(16) {
        let keystream = encrypt_bytes(&counter, key)?;
        for (i, byte) in chunk.iter().enumerate() {
            out.push(*byte ^ keystream[i]);
        }
        increment_counter_be(&mut counter);
    }

    Ok(out)
}

fn increment_counter_be(counter: &mut [u8; 16]) {
    for byte in counter.iter_mut().rev() {
        let (next, carry) = byte.overflowing_add(1);
        *byte = next;
        if !carry {
            break;
        }
    }
}
