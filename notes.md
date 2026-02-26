# Project Plan

I need to implement Salsa and ChaCha20 stream ciphers.
I need to optimize LowMC by reducing matrix multiplication and inversion overhead through precomputation.
I need to benchmark all implementations.
I need to create a test harness that outputs clean, graph-ready benchmark data in JSON, along with a Python script for plotting.

## LowMC
I cant get it to run it exceeds 240s runtime and still doesnt seem to run

## Salsa
All

## AES 
Block cipher - This needs to be done in the [aes repo](https://github.com/kush-10/aesencryption)

## Benchmarking 
Add a proper output/test harness

## Docs
Plan out structure. Deffo do the backround knowledge stuff at the least.

## Build/Run Profiles
- dev: Runs with Rust's default debug profile (`cargo run -p host`), so compile time is faster and debug checks are enabled; useful while iterating.
- prod: Runs with release optimizations (`cargo run -p host --release`), so compile takes longer but runtime is usually faster; use for performance measurements and final runs.
- native-dev: Same as dev profile but with proving disabled (`NO_RISC0=1 cargo run -p host`); useful to measure host/algorithm behavior without zk proving overhead.
- native-prod: Same as prod profile but with proving disabled (`NO_RISC0=1 cargo run -p host --release`); useful for optimized non-zk baseline comparisons.
