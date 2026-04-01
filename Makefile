.DEFAULT_GOAL := list

PROJECTS := aes-r0 aes-r0-optimised aes-ctr lowmc-r0 lowmc-r0-optimised salsa-r0 operation-bnchmrk-r0
PROJECT ?= lowmc-r0
TARGETS := risc0-dev risc0-prod bench clean clean-docs

.PHONY: list $(TARGETS)

list:
	@printf "Targets:\n"
	@printf "  make risc0-dev  PROJECT=<project>   # dev run with pprof output\n"
	@printf "  make risc0-prod PROJECT=<project>   # release run\n"
	@printf "  make bench                          # benchmark everything\n"
	@printf "  make clean                          # cargo clean across projects\n"
	@printf "  make clean-docs                     # clean docs build artifacts\n"
	@printf "\nAvailable projects:\n"
	@for project in $(PROJECTS); do printf "  %s\n" "$$project"; done
	@printf "\nCurrent PROJECT=%s\n" "$(PROJECT)"

risc0-dev: ; @$(MAKE) -C "$(PROJECT)" risc0-dev
risc0-prod: ; @$(MAKE) -C "$(PROJECT)" risc0-prod

bench:
	@python3 bench-harness/runner.py && \
	python3 bench-harness/aggregate.py && \
	python3 bench-harness/plot.py && \
	python3 bench-harness/runner.py --config bench-harness/config.operations.toml && \
	python3 bench-harness/aggregate.py --output-root artifacts/benchmarks-ops && \
	python3 bench-harness/plot.py --output-root artifacts/benchmarks-ops

clean:
	@for project in $(PROJECTS); do \
		printf "Cleaning %s...\n" "$$project"; \
		cargo clean --manifest-path "$$project/Cargo.toml"; \
	done

clean-docs: ; @$(MAKE) -C docs clean
