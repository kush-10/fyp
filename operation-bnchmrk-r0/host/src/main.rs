use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::{fmt, time::Instant};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(u8)]
enum Operation {
    And,
    Or,
    Xor,
    Xnor,
    Toffoli,
    Add,
    Sub,
    Mul,
    RotateLeft,
    RotateRight,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Operation::And => "And",
            Operation::Or => "Or",
            Operation::Xor => "Xor",
            Operation::Xnor => "XNOR",
            Operation::Toffoli => "Toffoli",
            Operation::Add => "Add",
            Operation::Sub => "Sub",
            Operation::Mul => "Mul",
            Operation::RotateLeft => "Rotate Left",
            Operation::RotateRight => "Rotate Right",
        };
        f.write_str(name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BenchmarkRequest {
    op: Operation,
    iterations: u32,
    seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BenchmarkResult {
    op: Operation,
    iterations: u32,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    const ITERATIONS: u32 = 10000;
    const BASE_SEED: u64 = 0xC0DE_CAFE_D15E_A5E5;

    let operations: &[Operation] = &[
        Operation::And,
        Operation::Or,
        Operation::Xor,
        Operation::Xnor,
        Operation::Toffoli,
        Operation::Add,
        Operation::Sub,
        Operation::Mul,
        Operation::RotateLeft,
        Operation::RotateRight,
    ];

    let prover = default_prover();

    // Table columns:
    // - Total/User Cycle: zkVM cycles (total includes paging/reserved, user is guest code only).
    // - User/Iter: user cycles divided by iteration count for rough per-op cost.
    // - Seconds/Seconds/Iter: wall-clock proof time for the run and per-iteration average.
    // - Operation: which workload was exercised.
    print_header();

    for (idx, op) in operations.iter().copied().enumerate() {
        let request = BenchmarkRequest {
            op,
            iterations: ITERATIONS,
            seed: BASE_SEED.wrapping_add(idx as u64 * 0x9E37_79B9),
        };

        let env = ExecutorEnv::builder()
            .write(&request)
            .unwrap()
            .build()
            .unwrap();

        let start = Instant::now();
        let prove_info = prover.prove(env, METHOD_ELF).unwrap();
        let elapsed = start.elapsed();
        let receipt = prove_info.receipt;
        let stats = prove_info.stats;

        let _result: BenchmarkResult = receipt.journal.decode().unwrap();
        print_row(
            stats.total_cycles,
            stats.user_cycles,
            elapsed.as_secs_f64(),
            ITERATIONS,
            op,
        );

        receipt.verify(METHOD_ID).unwrap();
    }
}

fn print_header() {
    println!(
        "{:>12} | {:>11} | {:>12} | {:>10} | {:>13} | {:<12}",
        "Total Cycle", "User Cycle", "User/Iter", "Seconds", "Seconds/Iter", "Operation"
    );
    println!("{}", "-".repeat(85));
}

fn print_row(total_cycles: u64, user_cycles: u64, seconds: f64, iterations: u32, op: Operation) {
    println!(
        "{:>12} | {:>11} | {:>12.2} | {:>10.4} | {:>13.6} | {:<12}",
        total_cycles,
        user_cycles,
        user_cycles as f64 / iterations as f64,
        seconds,
        seconds / iterations as f64,
        op
    );
}
