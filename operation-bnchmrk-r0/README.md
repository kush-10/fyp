# Operation RISC Zero Benchmark

This workspace benchmarks primitive operation workloads inside a RISC Zero
guest. It provides directional cost comparisons for operations used by the
cryptographic targets.

Benchmarked operations include AND, OR, XOR, XNOR, Toffoli-style boolean work,
addition, subtraction, multiplication, rotate-left, and rotate-right.

## Run

From the repository root:

```bash
make risc0-dev PROJECT=operation-bnchmrk-r0
make risc0-prod PROJECT=operation-bnchmrk-r0
```

From this directory:

```bash
make risc0-dev
make risc0-prod
```

The benchmark harness uses the release command with JSON output:

```bash
cargo run -p host --release -- --json
```

## Layout

- `host`: runs each operation workload and emits tabular or JSON results.
- `methods`: RISC Zero method build wrapper.
- `methods/guest`: guest program that executes the operation loops.
