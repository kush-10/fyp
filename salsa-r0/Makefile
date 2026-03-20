.DEFAULT_GOAL := list

TARGETS := risc0-dev risc0-prod
PPROF_DIR ?= ../artifacts/pprof/$(notdir $(CURDIR))

.PHONY: list $(TARGETS)

list:
	@printf "Targets:\n"
	@printf "  make risc0-dev\n"
	@printf "  make risc0-prod\n"

risc0-dev:
	@ts="$$(date -u +"%Y%m%d-%H%M%SZ")"; \
	out_file="$(PPROF_DIR)/dev-$$ts.pb"; \
	mkdir -p "$(PPROF_DIR)"; \
	printf "RISC0_PPROF_OUT=%s\n" "$$out_file"; \
	RISC0_PPROF_OUT="$$out_file" RISC0_PPROF_ENABLE_INLINE_FUNCTIONS=1 RUST_LOG=info RISC0_DEV_MODE=1 RISC0_INFO=1 cargo run -p host

risc0-prod: ; cargo run -p host --release
