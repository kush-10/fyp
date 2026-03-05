.DEFAULT_GOAL := list

TARGETS := build build-dev build-prod run run-dev run-prod run-native-dev run-native-prod bench bench-dev bench-prod bench-native-dev bench-native-prod test clean

.PHONY: default list $(TARGETS)

default: list

list: ; @printf "Targets:\n"; for target in $(TARGETS); do printf "  make %s\n" "$$target"; done

build: build-dev
build-dev: ; cargo build --workspace
build-prod: ; cargo build --workspace --release

run: run-dev
run-dev: ; cargo run -p host
run-prod: ; cargo run -p host --release
run-native-dev: ; NO_RISC0=1 cargo run -p host
run-native-prod: ; NO_RISC0=1 cargo run -p host --release

bench: bench-dev
bench-dev: run-dev
bench-prod: run-prod
bench-native-dev: run-native-dev
bench-native-prod: run-native-prod

test: ; cargo test --workspace
clean: ; cargo clean
