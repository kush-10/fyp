# Projects

All benchmark workspaces follow the same basic RISC Zero shape: a host crate
drives execution and verification, while `methods/guest` contains the zkVM guest
program.

| Project | Purpose |
| --- | --- |
| `aes-r0` | AES-192 ECB baseline target. |
| `aes-r0-optimised` | AES-192 ECB target used for S-box backend experimentation. |
| `aes-ctr` | AES-192 CTR target for variable-block stream-encryption benchmarks. |
| `aes-ctr-hmac` | AES-192 CTR plus HMAC-SHA256-192 authenticated-encryption target. |
| `lowmc-r0` | LowMC baseline target using the 256-bit block, 80-bit key parameter set. |
| `lowmc-r0-optimised` | LowMC target used for optimization experiments. |
| `salsa-r0` | Salsa20/20 benchmark target using a manual implementation. |
| `operation-bnchmrk-r0` | Primitive operation workloads for directional zkVM cost comparisons. |

Run a single project from the root with:

```bash
make risc0-prod PROJECT=<project>
```

Run from inside an individual workspace with:

```bash
make risc0-prod
```

Most workspaces also support `make risc0-dev`, which enables RISC Zero dev mode
and writes pprof data below `artifacts/pprof`.
