.DEFAULT_GOAL := list

TARGETS := build build-dev build-prod run run-dev run-prod run-native-dev run-native-prod encrypt encrypt-dev encrypt-prod encrypt-native-dev encrypt-native-prod test clean

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

encrypt: encrypt-dev
encrypt-dev: run-dev
encrypt-prod: run-prod
encrypt-native-dev: run-native-dev
encrypt-native-prod: run-native-prod

test: ; cargo test --workspace
clean: ; cargo clean
