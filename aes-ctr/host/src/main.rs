use aesencryption::{decrypt_ctr, encrypt_ctr, AES_KEY_LEN};
use anyhow::Result;
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// -- Shared types (host <-> guest) ---------------------------------------

/// Host input serialized and sent to the guest.
#[derive(Debug, Serialize, Deserialize)]
struct AesCtrSpec {
    plaintext: Vec<u8>,
    enc_key: [u8; AES_KEY_LEN],
    iv: [u8; 16],
    expected_ciphertext: Vec<u8>,
}

/// Guest journal payload decoded by the host.
#[derive(Debug, Serialize, Deserialize)]
struct AesCtrResult {
    ciphertext: Vec<u8>,
}

// -- CLI benchmark output -------------------------------------------------

#[derive(Debug, Serialize)]
struct CliBenchmarkResult {
    benchmark_id: String,
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
    num_blocks: usize,
    proof_bytes: Option<usize>,
    full_receipt_bytes: Option<usize>,
}

// -- Test material --------------------------------------------------------

const NIST_ENC_KEY: [u8; AES_KEY_LEN] = [
    0x8E, 0x73, 0xB0, 0xF7, 0xDA, 0x0E, 0x64, 0x52, 0xC8, 0x10, 0xF3, 0x2B, 0x80, 0x90, 0x79, 0xE5,
    0x62, 0xF8, 0xEA, 0xD2, 0x52, 0x2C, 0x6B, 0x7B,
];

const NIST_IV: [u8; 16] = [
    0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF,
];

/// First block of the NIST SP 800-38A plaintext, replicated to fill N blocks.
const NIST_BLOCK: [u8; 16] = [
    0x6B, 0xC1, 0xBE, 0xE2, 0x2E, 0x40, 0x9F, 0x96, 0xE9, 0x3D, 0x7E, 0x11, 0x73, 0x93, 0x17, 0x2A,
];

fn main() -> Result<()> {
    let json_mode = args_contains("--json");
    let num_blocks = parse_blocks_arg().unwrap_or(4);

    log_stage(&format!("starting host (blocks={num_blocks})"));

    let plaintext: Vec<u8> = NIST_BLOCK
        .iter()
        .copied()
        .cycle()
        .take(16 * num_blocks)
        .collect();
    let expected_ciphertext = encrypt_ctr(&plaintext, &NIST_ENC_KEY, &NIST_IV);

    let spec = AesCtrSpec {
        plaintext,
        enc_key: NIST_ENC_KEY,
        iv: NIST_IV,
        expected_ciphertext,
    };

    if no_risc0_mode() {
        log_stage("running native benchmark path");
        let native_start = Instant::now();
        let ciphertext = encrypt_ctr(&spec.plaintext, &spec.enc_key, &spec.iv);
        let recovered = decrypt_ctr(&ciphertext, &spec.enc_key, &spec.iv);
        let native_duration = native_start.elapsed();

        assert!(
            ciphertext == spec.expected_ciphertext,
            "native ciphertext mismatch"
        );
        assert!(
            recovered == spec.plaintext,
            "native decrypt_ctr did not recover plaintext"
        );

        if json_mode {
            let out = CliBenchmarkResult {
                benchmark_id: format!("aes-ctr-{}blk", num_blocks),
                algorithm: "aes-192-ctr",
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
                    num_blocks,
                    proof_bytes: None,
                    full_receipt_bytes: None,
                },
            };
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("NO_RISC0=1: running native AES-CTR path without proving/verification.");
            println!(
                "Blocks: {num_blocks}, payload: {} bytes",
                spec.plaintext.len()
            );
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
    let proof_bytes = receipt.seal_size();
    let full_receipt_bytes =
        risc0_zkvm::serde::to_vec(&receipt)?.len() * core::mem::size_of::<u32>();

    log_stage("starting proof verification");
    let verify_start = Instant::now();
    receipt.verify(METHOD_ID)?;
    let verify_duration = verify_start.elapsed();
    log_stage("proof verification completed");

    let result: AesCtrResult = receipt.journal.decode()?;
    assert!(
        result.ciphertext == spec.expected_ciphertext,
        "guest ciphertext mismatch"
    );
    let recovered = decrypt_ctr(&result.ciphertext, &spec.enc_key, &spec.iv);
    assert!(
        recovered == spec.plaintext,
        "host decrypt_ctr did not recover plaintext"
    );

    if json_mode {
        let out = CliBenchmarkResult {
            benchmark_id: format!("aes-ctr-{}blk", num_blocks),
            algorithm: "aes-192-ctr",
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
                num_blocks,
                proof_bytes: Some(proof_bytes),
                full_receipt_bytes: Some(full_receipt_bytes),
            },
        };
        println!("{}", serde_json::to_string(&out)?);
    } else {
        println!(
            "AES-CTR output committed by the guest ({} blocks, {} bytes)",
            num_blocks,
            result.ciphertext.len(),
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
        println!(
            "Proof size: {} bytes (seal), full receipt size: {} bytes",
            proof_bytes, full_receipt_bytes
        );
    }

    Ok(())
}

/// Parses `--blocks <N>` from CLI arguments.
fn parse_blocks_arg() -> Option<usize> {
    let args: Vec<String> = std::env::args().collect();
    for (i, arg) in args.iter().enumerate() {
        if arg == "--blocks" {
            return args.get(i + 1).and_then(|v| v.parse().ok());
        }
    }
    None
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
        "[host][aes-ctr][{}.{:03}] {stage}",
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
