//! HMAC-SHA256 helpers for AES-CTR authentication.
//!
//! This module implements HMAC directly from the SHA-256 primitive, with
//! project-specific truncation to 192 bits (24 bytes).

use alloc::vec::Vec;
use sha2::{Digest, Sha256};

pub const HMAC_SHA256_BLOCK_LEN: usize = 64;
pub const HMAC_SHA256_OUTPUT_LEN: usize = 32;
pub const HMAC_SHA256_KEY_LEN: usize = 32;
pub const HMAC_SHA256_192_TAG_LEN: usize = 24;

const ETM_DOMAIN_SEP: &[u8] = b"aes-192-ctr+hmac-sha256-192:v1";

/// Computes HMAC-SHA256 for arbitrary key/data.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; HMAC_SHA256_OUTPUT_LEN] {
    let mut key_block = [0u8; HMAC_SHA256_BLOCK_LEN];

    if key.len() > HMAC_SHA256_BLOCK_LEN {
        let key_hash = Sha256::digest(key);
        key_block[..HMAC_SHA256_OUTPUT_LEN].copy_from_slice(&key_hash);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; HMAC_SHA256_BLOCK_LEN];
    let mut opad = [0x5Cu8; HMAC_SHA256_BLOCK_LEN];
    for i in 0..HMAC_SHA256_BLOCK_LEN {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(data);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    let outer_hash = outer.finalize();

    let mut out = [0u8; HMAC_SHA256_OUTPUT_LEN];
    out.copy_from_slice(&outer_hash);
    out
}

/// Computes HMAC-SHA256 tag truncated to 192 bits.
pub fn hmac_sha256_192(key: &[u8], data: &[u8]) -> [u8; HMAC_SHA256_192_TAG_LEN] {
    let full = hmac_sha256(key, data);
    let mut out = [0u8; HMAC_SHA256_192_TAG_LEN];
    out.copy_from_slice(&full[..HMAC_SHA256_192_TAG_LEN]);
    out
}

/// Computes Encrypt-then-MAC tag over framed associated data, IV, and ciphertext.
pub fn compute_auth_tag_192(
    mac_key: &[u8; HMAC_SHA256_KEY_LEN],
    aad: &[u8],
    iv: &[u8; 16],
    ciphertext: &[u8],
) -> [u8; HMAC_SHA256_192_TAG_LEN] {
    let mac_input = build_etm_mac_input(aad, iv, ciphertext);
    hmac_sha256_192(mac_key, &mac_input)
}

/// Verifies a 192-bit auth tag in constant time.
pub fn verify_auth_tag_192(
    mac_key: &[u8; HMAC_SHA256_KEY_LEN],
    aad: &[u8],
    iv: &[u8; 16],
    ciphertext: &[u8],
    tag: &[u8],
) -> bool {
    if tag.len() != HMAC_SHA256_192_TAG_LEN {
        return false;
    }

    let expected = compute_auth_tag_192(mac_key, aad, iv, ciphertext);
    constant_time_eq(&expected, tag)
}

fn build_etm_mac_input(aad: &[u8], iv: &[u8; 16], ciphertext: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(
        ETM_DOMAIN_SEP.len() + 8 + aad.len() + 8 + iv.len() + 8 + ciphertext.len(),
    );

    out.extend_from_slice(ETM_DOMAIN_SEP);
    append_u64_be(&mut out, aad.len() as u64);
    out.extend_from_slice(aad);
    append_u64_be(&mut out, iv.len() as u64);
    out.extend_from_slice(iv);
    append_u64_be(&mut out, ciphertext.len() as u64);
    out.extend_from_slice(ciphertext);

    out
}

fn append_u64_be(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0u8;
    for (&l, &r) in left.iter().zip(right.iter()) {
        diff |= l ^ r;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc4231_test_case_1_matches_full_hmac_sha256() {
        let key = [0x0Bu8; 20];
        let data = b"Hi There";
        let expected = [
            0xB0, 0x34, 0x4C, 0x61, 0xD8, 0xDB, 0x38, 0x53, 0x5C, 0xA8, 0xAF, 0xCE, 0xAF, 0x0B,
            0xF1, 0x2B, 0x88, 0x1D, 0xC2, 0x00, 0xC9, 0x83, 0x3D, 0xA7, 0x26, 0xE9, 0x37, 0x6C,
            0x2E, 0x32, 0xCF, 0xF7,
        ];

        assert_eq!(hmac_sha256(&key, data), expected);
    }

    #[test]
    fn truncated_hmac_sha256_192_matches_prefix() {
        let key = [0x0Bu8; 20];
        let data = b"Hi There";
        let expected = [
            0xB0, 0x34, 0x4C, 0x61, 0xD8, 0xDB, 0x38, 0x53, 0x5C, 0xA8, 0xAF, 0xCE, 0xAF, 0x0B,
            0xF1, 0x2B, 0x88, 0x1D, 0xC2, 0x00, 0xC9, 0x83, 0x3D, 0xA7,
        ];

        assert_eq!(hmac_sha256_192(&key, data), expected);
    }

    #[test]
    fn verify_auth_tag_detects_tampering() {
        let mac_key = [0x11u8; HMAC_SHA256_KEY_LEN];
        let iv = [0x22u8; 16];
        let aad = b"aad";
        let ciphertext = b"ciphertext";

        let tag = compute_auth_tag_192(&mac_key, aad, &iv, ciphertext);
        assert!(verify_auth_tag_192(&mac_key, aad, &iv, ciphertext, &tag));

        let mut tampered = tag;
        tampered[0] ^= 0x80;
        assert!(!verify_auth_tag_192(
            &mac_key, aad, &iv, ciphertext, &tampered
        ));
    }
}
