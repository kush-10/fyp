use anyhow::Result;
use lowmc_core::{
    block_from_bytes, block_to_bytes, key_from_bytes, Block, BlockBytes, KeyBlock, KeyBytes, LowMc,
};
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct LowMcTestSpec {
    plaintext: BlockBytes,
    key: KeyBytes,
    expected_cipher: BlockBytes,
    lin_matrices: Vec<Vec<Block>>,
    inv_lin_matrices: Vec<Vec<Block>>,
    round_constants: Vec<Block>,
    key_matrices: Vec<Vec<KeyBlock>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LowMcTestResult {
    ciphertext: BlockBytes,
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
    block_bytes: usize,
    key_bytes: usize,
}

fn main() -> Result<()> {
    let json_mode = args_contains("--json");

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let mut key_bytes = [0u8; 10];
    key_bytes[9] = 0x01;

    let mut plaintext_bytes = [0u8; 32];
    plaintext_bytes[30] = 0xFF;
    plaintext_bytes[31] = 0xD5;

    let key = key_from_bytes(&key_bytes);
    let plaintext = block_from_bytes(&plaintext_bytes);

    let lowmc = LowMc::new(key);
    let reference_cipher = lowmc.encrypt(&plaintext);
    let expected_cipher = block_to_bytes(&reference_cipher);
    let (lin_matrices, inv_lin_matrices, round_constants, key_matrices) = lowmc.precomputed_data();

    let spec = LowMcTestSpec {
        plaintext: plaintext_bytes,
        key: key_bytes,
        expected_cipher,
        lin_matrices,
        inv_lin_matrices,
        round_constants,
        key_matrices,
    };

    if no_risc0_mode() {
        let native_start = Instant::now();
        let ciphertext = lowmc.encrypt(&plaintext);
        let decrypted = lowmc.decrypt(&ciphertext);
        let native_duration = native_start.elapsed();

        assert!(
            decrypted == plaintext,
            "native decrypt(encrypt(plaintext)) did not recover plaintext"
        );

        let ciphertext_bytes = block_to_bytes(&ciphertext);
        assert!(
            ciphertext_bytes == spec.expected_cipher,
            "native ciphertext mismatch"
        );

        if json_mode {
            let out = CliBenchmarkResult {
                benchmark_id: "lowmc-r0",
                algorithm: "lowmc",
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
                    block_bytes: plaintext_bytes.len(),
                    key_bytes: key_bytes.len(),
                },
            };
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("NO_RISC0=1: running native LowMC path without proving/verification.");
            println!(
                "LowMC ciphertext (native): {}",
                hex_bytes(&ciphertext_bytes)
            );
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

    let result: LowMcTestResult = receipt.journal.decode()?;

    if json_mode {
        let out = CliBenchmarkResult {
            benchmark_id: "lowmc-r0",
            algorithm: "lowmc",
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
                block_bytes: plaintext_bytes.len(),
                key_bytes: key_bytes.len(),
            },
        };
        println!("{}", serde_json::to_string(&out)?);
    } else {
        println!(
            "LowMC ciphertext committed by the guest: {}",
            hex_bytes(&result.ciphertext)
        );
        println!("Proof verified successfully for LowMC encryption/decryption roundtrip.");
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

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0F) as usize] as char);
    }
    out
}
