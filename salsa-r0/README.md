# salsa-r0

RISC Zero benchmark target for Salsa20 with a manual implementation:

- `manual`: from-spec Salsa20/20 implementation

Run in dev mode:

```bash
cargo run -p host
```

Run without proving (native timing only):

```bash
NO_RISC0=1 cargo run -p host
```
