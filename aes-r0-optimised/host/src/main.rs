use aesencryption::{decrypt_bytes, encrypt_bytes};
use anyhow::{anyhow, Result};
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Host input that is serialized and sent to the guest.
#[derive(Debug, Serialize, Deserialize)]
struct AesTestSpec {
    plaintext: Vec<u8>,
    key: [u8; 16],
    expected_ciphertext: Vec<u8>,
}

/// Guest journal payload decoded by the host.
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
    log_stage("starting host");

    // NIST AES-128 test vector used by the guest library.
    let key = [
        0x2B, 0x7E, 0x15, 0x16, 0x28, 0xAE, 0xD2, 0xA6, 0xAB, 0xF7, 0x15, 0x88, 0x09, 0xCF, 0x4F,
        0x3C,
    ];
    let plaintext = vec![
        0x6B, 0xC1, 0xBE, 0xE2, 0x2E, 0x40, 0x9F, 0x96, 0xE9, 0x3D, 0x7E, 0x11, 0x73, 0x93, 0x17,
        0x2A, 0xAE, 0x2D, 0x8A, 0x57, 0x1E, 0x03, 0xAC, 0x9C, 0x9E, 0xB7, 0x6F, 0xAC, 0x45, 0xAF,
        0x8E, 0x51, 0x30, 0xC8, 0x1C, 0x46, 0xA3, 0x5C, 0xE4, 0x11, 0xE5, 0xFB, 0xC1, 0x19, 0x1A,
        0x0A, 0x52, 0xEF, 0xF6, 0x9F, 0x24, 0x45, 0xDF, 0x4F, 0x9B, 0x17, 0xAD, 0x2B, 0x41, 0x7B,
        0xE6, 0x6C, 0x37, 0x10,
    ];
    let expected_ciphertext = vec![
        0x3A, 0xD7, 0x7B, 0xB4, 0x0D, 0x7A, 0x36, 0x60, 0xA8, 0x9E, 0xCA, 0xF3, 0x24, 0x66, 0xEF,
        0x97, 0xF5, 0xD3, 0xD5, 0x85, 0x03, 0xB9, 0x69, 0x9D, 0xE7, 0x85, 0x89, 0x5A, 0x96, 0xFD,
        0xBA, 0xAF, 0x43, 0xB1, 0xCD, 0x7F, 0x59, 0x8E, 0xCE, 0x23, 0x88, 0x1B, 0x00, 0xE3, 0xED,
        0x03, 0x06, 0x88, 0x7B, 0x0C, 0x78, 0x5E, 0x27, 0xE8, 0xAD, 0x3F, 0x82, 0x23, 0x20, 0x71,
        0x04, 0x72, 0x5D, 0xD4,
    ];

    let spec = AesTestSpec {
        plaintext,
        key,
        expected_ciphertext,
    };

    if no_risc0_mode() {
        log_stage("running native benchmark path");
        let native_start = Instant::now();
        let ciphertext = encrypt_bytes(&spec.plaintext, &spec.key)
            .map_err(|err| anyhow!("native AES encryption failed: {err:?}"))?;
        let decrypted = decrypt_bytes(&ciphertext, &spec.key)
            .map_err(|err| anyhow!("native AES decryption failed: {err:?}"))?;
        let native_duration = native_start.elapsed();
        assert!(
            ciphertext == spec.expected_ciphertext,
            "native ciphertext mismatch"
        );
        assert!(
            decrypted == spec.plaintext,
            "native decrypt(encrypt(plaintext)) did not recover plaintext"
        );

        if json_mode {
            let out = CliBenchmarkResult {
                benchmark_id: "aes-r0-optimised",
                algorithm: "aes-128",
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
            println!("NO_RISC0=1: running native AES path without proving/verification.");
            println!("AES ciphertext (native bytes): {:?}", ciphertext);
            println!(
                "Native execution time: {:.3} seconds",
                native_duration.as_secs_f64()
            );
        }
        return Ok(());
    }

    log_stage("building zk executor env");
    let env = ExecutorEnv::builder().write(&spec)?.build()?;

    log_stage("starting proof generation");
    let prover = default_prover();
    let prove_start = Instant::now();
    let prove_info = prover.prove(env, METHOD_ELF)?;
    let prove_duration = prove_start.elapsed();
    let receipt = prove_info.receipt;

    log_stage("starting proof verification");
    let verify_start = Instant::now();
    receipt.verify(METHOD_ID)?;
    let verify_duration = verify_start.elapsed();
    log_stage("proof verification completed");

    let result: AesTestResult = receipt.journal.decode()?;
    if json_mode {
        let out = CliBenchmarkResult {
            benchmark_id: "aes-r0-optimised",
            algorithm: "aes-128",
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
            "AES ciphertext committed by the guest (bytes): {:?}",
            result.ciphertext
        );
        println!("Proof verified successfully for AES encryption/decryption roundtrip.");
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

fn args_contains(flag: &str) -> bool {
    std::env::args().any(|arg| arg == flag)
}

fn no_risc0_mode() -> bool {
    matches!(
        std::env::var("NO_RISC0").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
    )
}

/// Emits a wall-clock timestamped host log to stderr.
fn log_stage(stage: &str) {
    let ts = unix_timestamp();
    eprintln!(
        "[host][aes-r0-optimised][{}.{:03}] {stage}",
        ts.as_secs(),
        ts.subsec_millis()
    );
}

/// Returns current UNIX timestamp, falling back to zero on clock errors.
fn unix_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
}
