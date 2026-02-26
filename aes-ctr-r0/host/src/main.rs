use aesencryption::{encrypt_bytes, AesError};
use anyhow::{anyhow, Result};
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::time::Instant;

// Input and output structs must match the guest definitions for serde encoding.
#[derive(Debug, Serialize, Deserialize)]
struct AesTestSpec {
    plaintext: Vec<u8>,
    key: [u8; 16],
    initial_counter_block: [u8; 16],
    expected_ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AesTestResult {
    ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize)]
struct CliBenchmarkResult {
    benchmark_id: &'static str,
    algorithm: &'static str,
    mode: &'static str,
    status: &'static str,
    timings: CliTimings,
    cycles: CliCycles,
    params: CliParams,
}

#[derive(Debug, Serialize)]
struct CliTimings {
    prove_seconds: Option<f64>,
    verify_seconds: Option<f64>,
    total_seconds: f64,
}

#[derive(Debug, Serialize)]
struct CliCycles {
    total_cycles: Option<u64>,
    user_cycles: Option<u64>,
    paging_cycles: Option<u64>,
    reserved_cycles: Option<u64>,
}

#[derive(Debug, Serialize)]
struct CliParams {
    payload_bytes: usize,
}

fn main() -> Result<()> {
    let json_mode = args_contains("--json");

    // NIST SP 800-38A F.5 AES-128-CTR test vector.
    let key = [
        0x2B, 0x7E, 0x15, 0x16, 0x28, 0xAE, 0xD2, 0xA6, 0xAB, 0xF7, 0x15, 0x88, 0x09, 0xCF, 0x4F,
        0x3C,
    ];
    let initial_counter_block = [
        0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE,
        0xFF,
    ];
    let plaintext = vec![
        0x6B, 0xC1, 0xBE, 0xE2, 0x2E, 0x40, 0x9F, 0x96, 0xE9, 0x3D, 0x7E, 0x11, 0x73, 0x93, 0x17,
        0x2A, 0xAE, 0x2D, 0x8A, 0x57, 0x1E, 0x03, 0xAC, 0x9C, 0x9E, 0xB7, 0x6F, 0xAC, 0x45, 0xAF,
        0x8E, 0x51, 0x30, 0xC8, 0x1C, 0x46, 0xA3, 0x5C, 0xE4, 0x11, 0xE5, 0xFB, 0xC1, 0x19, 0x1A,
        0x0A, 0x52, 0xEF, 0xF6, 0x9F, 0x24, 0x45, 0xDF, 0x4F, 0x9B, 0x17, 0xAD, 0x2B, 0x41, 0x7B,
        0xE6, 0x6C, 0x37, 0x10,
    ];
    let expected_ciphertext = vec![
        0x87, 0x4D, 0x61, 0x91, 0xB6, 0x20, 0xE3, 0x26, 0x1B, 0xEF, 0x68, 0x64, 0x99, 0x0D, 0xB6,
        0xCE, 0x98, 0x06, 0xF6, 0x6B, 0x79, 0x70, 0xFD, 0xFF, 0x86, 0x17, 0x18, 0x7B, 0xB9, 0xFF,
        0xFD, 0xFF, 0x5A, 0xE4, 0xDF, 0x3E, 0xDB, 0xD5, 0xD3, 0x5E, 0x5B, 0x4F, 0x09, 0x02, 0x0D,
        0xB0, 0x3E, 0xAB, 0x1E, 0x03, 0x1D, 0xDA, 0x2F, 0xBE, 0x03, 0xD1, 0x79, 0x21, 0x70, 0xA0,
        0xF3, 0x00, 0x9C, 0xEE,
    ];

    let spec = AesTestSpec {
        plaintext,
        key,
        initial_counter_block,
        expected_ciphertext,
    };

    if no_risc0_mode() {
        let native_start = Instant::now();
        let ciphertext =
            aes_ctr_encrypt_bytes(&spec.plaintext, &spec.key, spec.initial_counter_block)
                .map_err(|err| anyhow!("native AES-CTR encryption failed: {err:?}"))?;
        let native_duration = native_start.elapsed();
        assert!(
            ciphertext == spec.expected_ciphertext,
            "native ciphertext mismatch"
        );

        if json_mode {
            let out = CliBenchmarkResult {
                benchmark_id: "aes-ctr-r0",
                algorithm: "aes-ctr",
                mode: "native",
                status: "ok",
                timings: CliTimings {
                    prove_seconds: None,
                    verify_seconds: None,
                    total_seconds: native_duration.as_secs_f64(),
                },
                cycles: CliCycles {
                    total_cycles: None,
                    user_cycles: None,
                    paging_cycles: None,
                    reserved_cycles: None,
                },
                params: CliParams {
                    payload_bytes: spec.plaintext.len(),
                },
            };
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("NO_RISC0=1: running native AES-CTR path without proving/verification.");
            println!("AES-CTR ciphertext (native bytes): {:?}", ciphertext);
            println!(
                "Native execution time: {:.3} seconds",
                native_duration.as_secs_f64()
            );
        }
        return Ok(());
    }

    let env = ExecutorEnv::builder().write(&spec)?.build()?;

    let prover = default_prover();
    let prove_start = Instant::now();
    let prove_info = prover.prove(env, METHOD_ELF)?;
    let prove_duration = prove_start.elapsed();
    let receipt = prove_info.receipt;

    let verify_start = Instant::now();
    receipt.verify(METHOD_ID)?;
    let verify_duration = verify_start.elapsed();

    let result: AesTestResult = receipt.journal.decode()?;
    if json_mode {
        let out = CliBenchmarkResult {
            benchmark_id: "aes-ctr-r0",
            algorithm: "aes-ctr",
            mode: "zk",
            status: "ok",
            timings: CliTimings {
                prove_seconds: Some(prove_duration.as_secs_f64()),
                verify_seconds: Some(verify_duration.as_secs_f64()),
                total_seconds: prove_duration.as_secs_f64() + verify_duration.as_secs_f64(),
            },
            cycles: CliCycles {
                total_cycles: Some(prove_info.stats.total_cycles),
                user_cycles: Some(prove_info.stats.user_cycles),
                paging_cycles: Some(prove_info.stats.paging_cycles),
                reserved_cycles: Some(prove_info.stats.reserved_cycles),
            },
            params: CliParams {
                payload_bytes: spec.plaintext.len(),
            },
        };
        println!("{}", serde_json::to_string(&out)?);
    } else {
        println!(
            "AES-CTR ciphertext committed by the guest (bytes): {:?}",
            result.ciphertext
        );
        println!("Proof verified successfully for AES-CTR encryption.");
        println!(
            "Proof generation time: {:.3} seconds (segments: {}, total cycles: {}, user: {}, paging: {}, reserved: {})",
            prove_duration.as_secs_f64(),
            prove_info.stats.segments,
            prove_info.stats.total_cycles,
            prove_info.stats.user_cycles,
            prove_info.stats.paging_cycles,
            prove_info.stats.reserved_cycles,
        );
        println!(
            "Proof verification time: {:.3} seconds",
            verify_duration.as_secs_f64()
        );
    }

    Ok(())
}

fn no_risc0_mode() -> bool {
    matches!(
        std::env::var("NO_RISC0").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
    )
}

fn args_contains(flag: &str) -> bool {
    std::env::args().any(|arg| arg == flag)
}

fn aes_ctr_encrypt_bytes(
    plaintext: &[u8],
    key: &[u8; 16],
    initial_counter_block: [u8; 16],
) -> Result<Vec<u8>, AesError> {
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
