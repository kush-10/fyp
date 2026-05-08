.DEFAULT_GOAL := list

PROJECTS := aes-r0 aes-r0-optimised aes-ctr aes-ctr-hmac lowmc-r0 lowmc-r0-optimised salsa-r0 operation-bnchmrk-r0
PROJECT ?= lowmc-r0
TARGETS := risc0-dev risc0-prod bench clean clean-docs
TARGETS += bench-auth-compare
TARGETS += report-benchmark
TRIALS ?= 5
LOCK_FLAGS ?= --require-clean
PYTHON ?= python3

.PHONY: list $(TARGETS)

list:
	@printf "Targets:\n"
	@printf "  make risc0-dev  PROJECT=<project>   # dev run with pprof output\n"
	@printf "  make risc0-prod PROJECT=<project>   # release run\n"
	@printf "  make bench                          # benchmark everything\n"
	@printf "  make bench-auth-compare             # compare LowMC/AES/CTR/CTR+HMAC\n"
	@printf "  make report-benchmark               # commit-locked N-trial report campaign\n"
	@printf "  make clean                          # cargo clean across projects\n"
	@printf "  make clean-docs                     # clean docs build artifacts\n"
	@printf "\nAvailable projects:\n"
	@for project in $(PROJECTS); do printf "  %s\n" "$$project"; done
	@printf "\nCurrent PROJECT=%s\n" "$(PROJECT)"

risc0-dev: ; @$(MAKE) -C "$(PROJECT)" risc0-dev
risc0-prod: ; @$(MAKE) -C "$(PROJECT)" risc0-prod

bench:
	@$(PYTHON) bench-harness/runner.py && \
	$(PYTHON) bench-harness/aggregate.py && \
	$(PYTHON) bench-harness/plot.py && \
	$(PYTHON) bench-harness/runner.py --config bench-harness/config.operations.toml && \
	$(PYTHON) bench-harness/aggregate.py --output-root artifacts/benchmarks-ops && \
	$(PYTHON) bench-harness/plot.py --output-root artifacts/benchmarks-ops

bench-auth-compare:
	@$(PYTHON) bench-harness/runner.py --config bench-harness/config.compare-auth.toml && \
	$(PYTHON) bench-harness/aggregate.py --output-root artifacts/benchmarks-auth-compare && \
	$(PYTHON) bench-harness/plot.py --output-root artifacts/benchmarks-auth-compare

report-benchmark:
	@$(PYTHON) bench-harness/runner.py --config bench-harness/config.report-main.toml --trials "$(TRIALS)" --interleave $(LOCK_FLAGS) && \
	$(PYTHON) bench-harness/aggregate.py --output-root artifacts/benchmarks/report-benchmarks/main && \
	$(PYTHON) bench-harness/plot.py --output-root artifacts/benchmarks/report-benchmarks/main && \
	$(PYTHON) bench-harness/runner.py --config bench-harness/config.report-ops.toml --trials "$(TRIALS)" --interleave $(LOCK_FLAGS) && \
	$(PYTHON) bench-harness/aggregate.py --output-root artifacts/benchmarks/report-benchmarks/ops && \
	$(PYTHON) bench-harness/plot.py --output-root artifacts/benchmarks/report-benchmarks/ops && \
	$(PYTHON) bench-harness/report_plots.py && \
	$(PYTHON) bench-harness/report_benchmark.py

clean:
	@for project in $(PROJECTS); do \
		printf "Cleaning %s...\n" "$$project"; \
		cargo clean --manifest-path "$$project/Cargo.toml"; \
	done

clean-docs: ; @$(MAKE) -C docs clean
