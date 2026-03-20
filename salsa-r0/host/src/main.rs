use anyhow::{anyhow, Result};
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use salsa_core::salsa20_encrypt_manual;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Host input that is serialized and sent to the guest.
#[derive(Debug, Serialize, Deserialize)]
struct SalsaTestSpec {
    plaintext: Vec<u8>,
    key: [u8; 32],
    nonce: [u8; 8],
    expected_ciphertext: Vec<u8>,
}

/// Guest journal payload decoded by the host.
#[derive(Debug, Serialize, Deserialize)]
struct SalsaTestResult {
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
    key_bytes: usize,
    nonce_bytes: usize,
}

const EXPECTED_KEY1_IV0: [u8; 64] = [
    0xE3, 0xBE, 0x8F, 0xDD, 0x8B, 0xEC, 0xA2, 0xE3, 0xEA, 0x8E, 0xF9, 0x47, 0x5B, 0x29, 0xA6, 0xE7,
    0x00, 0x39, 0x51, 0xE1, 0x09, 0x7A, 0x5C, 0x38, 0xD2, 0x3B, 0x7A, 0x5F, 0xAD, 0x9F, 0x68, 0x44,
    0xB2, 0x2C, 0x97, 0x55, 0x9E, 0x27, 0x23, 0xC7, 0xCB, 0xBD, 0x3F, 0xE4, 0xFC, 0x8D, 0x9A, 0x07,
    0x44, 0x65, 0x2A, 0x83, 0xE7, 0x2A, 0x9C, 0x46, 0x18, 0x76, 0xAF, 0x4D, 0x7E, 0xF1, 0xA1, 0x17,
];

fn main() -> Result<()> {
    let json_mode = args_contains("--json");
    log_stage("starting host");

    let mut key = [0u8; 32];
    key[0] = 0x80;
    let nonce = [0u8; 8];
    let plaintext = vec![0u8; 1024];

    let expected_ciphertext = encrypt_payload(&plaintext, &key, &nonce);
    assert_eq!(
        &expected_ciphertext[..EXPECTED_KEY1_IV0.len()],
        EXPECTED_KEY1_IV0.as_slice(),
        "reference vector mismatch for manual Salsa20",
    );

    let spec = SalsaTestSpec {
        plaintext,
        key,
        nonce,
        expected_ciphertext,
    };

    if no_risc0_mode() {
        log_stage("running native benchmark path");
        let native_start = Instant::now();
        let ciphertext = encrypt_payload(&spec.plaintext, &spec.key, &spec.nonce);
        let native_duration = native_start.elapsed();

        if ciphertext != spec.expected_ciphertext {
            return Err(anyhow!("native ciphertext mismatch"));
        }

        let decrypted = decrypt_payload(&ciphertext, &spec.key, &spec.nonce);
        if decrypted != spec.plaintext {
            return Err(anyhow!(
                "native decrypt(encrypt(plaintext)) did not recover plaintext"
            ));
        }

        if json_mode {
            let out = CliBenchmarkResult {
                benchmark_id: "salsa-r0",
                algorithm: "salsa20",
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
                    key_bytes: spec.key.len(),
                    nonce_bytes: spec.nonce.len(),
                },
            };
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("NO_RISC0=1: running native Salsa20 path without proving/verification.");
            println!(
                "Native execution time: {:.6} seconds",
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

    let result: SalsaTestResult = receipt.journal.decode()?;
    if result.ciphertext != spec.expected_ciphertext {
        return Err(anyhow!("guest ciphertext mismatch"));
    }

    let decrypted = decrypt_payload(&result.ciphertext, &spec.key, &spec.nonce);
    if decrypted != spec.plaintext {
        return Err(anyhow!(
            "guest decrypt(encrypt(plaintext)) did not recover plaintext"
        ));
    }

    if json_mode {
        let out = CliBenchmarkResult {
            benchmark_id: "salsa-r0",
            algorithm: "salsa20",
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
                key_bytes: spec.key.len(),
                nonce_bytes: spec.nonce.len(),
            },
        };
        println!("{}", serde_json::to_string(&out)?);
    } else {
        println!("Proof verified successfully for Salsa20 encryption.");
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

/// Encrypts a Salsa20 payload.
fn encrypt_payload(plaintext: &[u8], key: &[u8; 32], nonce: &[u8; 8]) -> Vec<u8> {
    salsa20_encrypt_manual(plaintext, key, nonce)
}

/// Decrypts a Salsa20 payload (same operation as encryption for stream ciphers).
fn decrypt_payload(ciphertext: &[u8], key: &[u8; 32], nonce: &[u8; 8]) -> Vec<u8> {
    salsa20_encrypt_manual(ciphertext, key, nonce)
}

/// Emits a wall-clock timestamped host log to stderr.
fn log_stage(stage: &str) {
    let ts = unix_timestamp();
    eprintln!(
        "[host][salsa-r0][{}.{:03}] {stage}",
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
