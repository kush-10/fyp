use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Operation {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkRequest {
    pub op: Operation,
    pub iterations: u32,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub op: Operation,
    pub iterations: u32,
}

fn main() {
    let request: BenchmarkRequest = env::read();
    let result = run_benchmark(request);
    env::commit(&result);
}

fn run_benchmark(request: BenchmarkRequest) -> BenchmarkResult {
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
                c ^= a & b; // target flips when both controls are set
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
                let res = a.wrapping_mul(b | 1); // keep multiplier odd to avoid zeroing bits
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

    // Keep the computation from being optimized out even though we don't return the values.
    core::hint::black_box(acc);
    core::hint::black_box(a ^ b ^ c);

    BenchmarkResult {
        op: request.op,
        iterations: request.iterations,
    }
}
