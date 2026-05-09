# Results

Benchmark outputs are generated under `artifacts/`. Most timestamped runs are
working data. The curated report results are the ones intended for release and
review.

## Curated Report Results

Current curated report outputs in this checkout:

```text
artifacts/benchmarks/report-benchmarks/main/20260508-190507Z
artifacts/benchmarks/report-benchmarks/ops/20260508-205918Z
artifacts/benchmarks/report-benchmarks/ctr-1to16/20260426-164924Z
artifacts/benchmarks/report-benchmarks/plots
artifacts/benchmarks/report-benchmarks/analysis
```

The `main` and `ops` directories contain timestamped benchmark records,
aggregates, logs, manifests, and plots. The top-level `plots` directory contains
the report-ready figures and CSV inputs. The `analysis` directory contains the
latest statistical analysis JSON.

## Regenerating Report Results

Run the report benchmark workflow from the repository root:

```bash
make report-benchmark TRIALS=5
```

This workflow runs commit-locked benchmark campaigns, aggregates their results,
generates plots, and prints the report tables.
