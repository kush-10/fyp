use anyhow::{anyhow, Result};
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Primitive operation workloads benchmarked by this host target.
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

/// Request payload serialized into the guest.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BenchmarkRequest {
    op: Operation,
    iterations: u32,
    seed: u64,
}

/// Guest journal payload decoded by the host.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BenchmarkResult {
    op: Operation,
    iterations: u32,
}

#[derive(Debug, Serialize)]
struct CliBenchmarkResult {
    benchmark_id: &'static str,
    algorithm: &'static str,
    mode: &'static str,
    status: &'static str,
    params: CliParams,
    results: Vec<CliOperationResult>,
}

#[derive(Debug, Serialize)]
struct CliParams {
    iterations: u32,
}

#[derive(Debug, Serialize)]
struct CliOperationResult {
    operation: String,
    status: &'static str,
    timings: CliTimings,
    cycles: CliCycles,
}

#[derive(Debug, Serialize)]
struct CliTimings {
    prove_seconds: f64,
    verify_seconds: f64,
    total_seconds: f64,
}

#[derive(Debug, Serialize)]
struct CliCycles {
    total_cycles: u64,
    user_cycles: u64,
    paging_cycles: u64,
    reserved_cycles: u64,
}

fn main() -> Result<()> {
    let (json_mode, iterations) = parse_cli_args()?;
    log_stage("starting host");

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    const DEFAULT_ITERATIONS: u32 = 100;
    const BASE_SEED: u64 = 0xC0DE_CAFE_D15E_A5E5;

    let iterations = iterations.unwrap_or(DEFAULT_ITERATIONS);

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

    // Table columns:
    // - Total/User Cycle: zkVM cycles (total includes paging/reserved, user is guest code only).
    // - User/Iter: user cycles divided by iteration count for rough per-op cost.
    // - Seconds/Seconds/Iter: wall-clock proof time for the run and per-iteration average.
    // - Operation: which workload was exercised.
    if !json_mode {
        print_header();
    }

    if no_risc0_mode() {
        for (idx, op) in operations.iter().copied().enumerate() {
            log_stage(&format!("running native operation {op}"));
            let request = BenchmarkRequest {
                op,
                iterations,
                seed: BASE_SEED.wrapping_add(idx as u64 * 0x9E37_79B9),
            };

            let start = Instant::now();
            let result = run_native_benchmark(request);
            let elapsed = start.elapsed();
            if !json_mode {
                print_native_row(elapsed.as_secs_f64(), result.iterations, result.op);
            }
        }

        if json_mode {
            let out = CliBenchmarkResult {
                benchmark_id: "operation-bnchmrk-r0",
                algorithm: "operations",
                mode: "native",
                status: "ok",
                params: CliParams { iterations },
                results: Vec::new(),
            };
            println!("{}", serde_json::to_string(&out)?);
        }

        return Ok(());
    }

    let prover = default_prover();
    let mut json_results = Vec::with_capacity(operations.len());

    for (idx, op) in operations.iter().copied().enumerate() {
        log_stage(&format!("starting zk operation {op}"));
        let request = BenchmarkRequest {
            op,
            iterations,
            seed: BASE_SEED.wrapping_add(idx as u64 * 0x9E37_79B9),
        };

        let env = ExecutorEnv::builder()
            .write(&request)
            .map_err(|e| anyhow!("failed to encode benchmark request: {e}"))?
            .build()
            .map_err(|e| anyhow!("failed to build executor env: {e}"))?;

        let prove_start = Instant::now();
        let prove_info = prover
            .prove(env, METHOD_ELF)
            .map_err(|e| anyhow!("failed proving {op}: {e}"))?;
        let prove_elapsed = prove_start.elapsed();
        let receipt = prove_info.receipt;
        let stats = prove_info.stats;

        let _result: BenchmarkResult = receipt
            .journal
            .decode()
            .map_err(|e| anyhow!("failed decoding journal for {op}: {e}"))?;

        let verify_start = Instant::now();
        receipt
            .verify(METHOD_ID)
            .map_err(|e| anyhow!("failed verifying receipt for {op}: {e}"))?;
        let verify_elapsed = verify_start.elapsed();
        log_stage(&format!("finished zk operation {op}"));

        if json_mode {
            json_results.push(CliOperationResult {
                operation: op.to_string(),
                status: "ok",
                timings: CliTimings {
                    prove_seconds: prove_elapsed.as_secs_f64(),
                    verify_seconds: verify_elapsed.as_secs_f64(),
                    total_seconds: prove_elapsed.as_secs_f64() + verify_elapsed.as_secs_f64(),
                },
                cycles: CliCycles {
                    total_cycles: stats.total_cycles,
                    user_cycles: stats.user_cycles,
                    paging_cycles: stats.paging_cycles,
                    reserved_cycles: stats.reserved_cycles,
                },
            });
        } else {
            print_row(
                stats.total_cycles,
                stats.user_cycles,
                prove_elapsed.as_secs_f64(),
                iterations,
                op,
            );
        }
    }

    if json_mode {
        let out = CliBenchmarkResult {
            benchmark_id: "operation-bnchmrk-r0",
            algorithm: "operations",
            mode: "zk",
            status: "ok",
            params: CliParams { iterations },
            results: json_results,
        };
        println!("{}", serde_json::to_string(&out)?);
    }

    Ok(())
}

fn no_risc0_mode() -> bool {
    matches!(
        std::env::var("NO_RISC0").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
    )
}

fn parse_cli_args() -> Result<(bool, Option<u32>)> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json_mode = args.iter().any(|a| a == "--json");

    let mut iterations = None;
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--iterations" {
            let value = args
                .get(i + 1)
                .ok_or_else(|| anyhow!("missing value for --iterations"))?;
            iterations = Some(
                value
                    .parse::<u32>()
                    .map_err(|e| anyhow!("invalid --iterations value '{value}': {e}"))?,
            );
            i += 1;
        }
        i += 1;
    }

    Ok((json_mode, iterations))
}

fn run_native_benchmark(request: BenchmarkRequest) -> BenchmarkResult {
    let mut a = request.seed;
    let mut b = request.seed.rotate_left(13);
    let mut c = request.seed.rotate_right(7);
    let mut acc: u64 = 0;

    match request.op {
        Operation::And => {
            for _ in 0..request.iterations {
                acc ^= a & b;
                a = a.rotate_left(1);
                b = b.rotate_right(3);
            }
        }
        Operation::Or => {
            for _ in 0..request.iterations {
                acc = acc.wrapping_add(a | b);
                a = a.wrapping_add(0x9E3779B97F4A7C15);
                b ^= a;
            }
        }
        Operation::Xor => {
            for _ in 0..request.iterations {
                acc ^= a ^ b;
                a = a.wrapping_add(1);
                b = b.rotate_left(5);
            }
        }
        Operation::Xnor => {
            for _ in 0..request.iterations {
                acc = acc.wrapping_add(!(a ^ b));
                a ^= b;
                b = b.wrapping_add(0x517CC1B727220A95);
            }
        }
        Operation::Toffoli => {
            for _ in 0..request.iterations {
                c ^= a & b;
                acc ^= c;
                a = a.rotate_left(3);
                b = b.rotate_right(3);
            }
        }
        Operation::Add => {
            for _ in 0..request.iterations {
                acc = acc.wrapping_add(a.wrapping_add(b));
                a = a.wrapping_add(3);
                b = b.wrapping_add(5);
            }
        }
        Operation::Sub => {
            for _ in 0..request.iterations {
                acc = acc.wrapping_add(a.wrapping_sub(b));
                a = a.wrapping_add(7);
                b = b.wrapping_add(11);
            }
        }
        Operation::Mul => {
            for _ in 0..request.iterations {
                let res = a.wrapping_mul(b | 1);
                acc ^= res;
                a ^= res.rotate_left(9);
                b = b.wrapping_add(0xD1B54A32D192ED03);
            }
        }
        Operation::RotateLeft => {
            for _ in 0..request.iterations {
                let rot = (b as u32) & 63;
                let res = a.rotate_left(rot);
                acc ^= res;
                a = res;
                b = b.rotate_right(1);
            }
        }
        Operation::RotateRight => {
            for _ in 0..request.iterations {
                let rot = (b as u32) & 63;
                let res = a.rotate_right(rot);
                acc = acc.wrapping_add(res);
                a = res;
                b = b.rotate_left(1);
            }
        }
    }

    core::hint::black_box(acc);
    core::hint::black_box(a ^ b ^ c);

    BenchmarkResult {
        op: request.op,
        iterations: request.iterations,
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

fn print_native_row(seconds: f64, iterations: u32, op: Operation) {
    println!(
        "{:>12} | {:>11} | {:>12} | {:>10.4} | {:>13.6} | {:<12}",
        "native",
        "native",
        "native",
        seconds,
        seconds / iterations as f64,
        op
    );
}

/// Emits a wall-clock timestamped host log to stderr.
fn log_stage(stage: &str) {
    let ts = unix_timestamp();
    eprintln!(
        "[host][operation-bnchmrk-r0][{}.{:03}] {stage}",
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
