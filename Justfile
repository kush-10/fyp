default: list

list:
	@printf "Project commands:\n"
	@printf "  just aes-dev             Run aes-r0 in dev profile\n"
	@printf "  just aes-prod            Run aes-r0 in release profile\n"
	@printf "  just aes-native-dev      Run aes-r0 without RISC0 proving (dev)\n"
	@printf "  just aes-native-prod     Run aes-r0 without RISC0 proving (release)\n"
	@printf "  just aes-rustcrypto-dev             Run aes-rustcrypto-r0 in dev profile\n"
	@printf "  just aes-rustcrypto-prod            Run aes-rustcrypto-r0 in release profile\n"
	@printf "  just aes-rustcrypto-native-dev      Run aes-rustcrypto-r0 without RISC0 proving (dev)\n"
	@printf "  just aes-rustcrypto-native-prod     Run aes-rustcrypto-r0 without RISC0 proving (release)\n"
	@printf "  just aes-ctr-dev         Run aes-ctr-r0 in dev profile\n"
	@printf "  just aes-ctr-prod        Run aes-ctr-r0 in release profile\n"
	@printf "  just aes-ctr-native-dev  Run aes-ctr-r0 without RISC0 proving (dev)\n"
	@printf "  just aes-ctr-native-prod Run aes-ctr-r0 without RISC0 proving (release)\n"
	@printf "  just lowmc-dev           Run lowmc-r0 in dev profile\n"
	@printf "  just lowmc-prod          Run lowmc-r0 in release profile\n"
	@printf "  just lowmc-native-dev    Run lowmc-r0 without RISC0 proving (dev)\n"
	@printf "  just lowmc-native-prod   Run lowmc-r0 without RISC0 proving (release)\n"
	@printf "  just op-dev              Run operation-bnchmrk-r0 in dev profile\n"
	@printf "  just op-prod             Run operation-bnchmrk-r0 in release profile\n"
	@printf "  just op-native-dev       Run operation-bnchmrk-r0 without RISC0 proving (dev)\n"
	@printf "  just op-native-prod      Run operation-bnchmrk-r0 without RISC0 proving (release)\n"
	@printf "  just salsa-dev           Run salsa-r0 (manual impl) in dev profile\n"
	@printf "  just salsa-prod          Run salsa-r0 (manual impl) in release profile\n"
	@printf "  just salsa-native-dev    Run salsa-r0 (manual impl) without RISC0 proving (dev)\n"
	@printf "  just salsa-native-prod   Run salsa-r0 (manual impl) without RISC0 proving (release)\n"
	@printf "  just all-build-dev       Build all projects in dev\n"
	@printf "  just all-build-prod      Build all projects in release\n"
	@printf "  just all-native-dev      Run all projects without RISC0 proving (dev)\n"
	@printf "  just all-native-prod     Run all projects without RISC0 proving (release)\n"
	@printf "  just bench-list          List enabled benchmark targets\n"
	@printf "  just bench-run           Run benchmark harness and write raw JSON\n"
	@printf "  just bench-aggregate     Aggregate latest benchmark run to JSON\n"
	@printf "  just bench-plot          Render PNG and SVG plots for latest run\n"
	@printf "  just bench-all           Run + aggregate + plot\n"
	@printf "  just bench-clean         Remove benchmark artifacts\n"
	@printf "  just clean               cargo clean in all projects\n"

aes-build-dev:
	just --justfile aes-r0/Justfile --working-directory aes-r0 build-dev

aes-build-prod:
	just --justfile aes-r0/Justfile --working-directory aes-r0 build-prod

aes-dev:
	just --justfile aes-r0/Justfile --working-directory aes-r0 run-dev

aes-prod:
	just --justfile aes-r0/Justfile --working-directory aes-r0 run-prod

aes-native-dev:
	just --justfile aes-r0/Justfile --working-directory aes-r0 run-native-dev

aes-native-prod:
	just --justfile aes-r0/Justfile --working-directory aes-r0 run-native-prod

aes-clean:
	just --justfile aes-r0/Justfile --working-directory aes-r0 clean

aes-rustcrypto-build-dev:
	just --justfile aes-rustcrypto-r0/Justfile --working-directory aes-rustcrypto-r0 build-dev

aes-rustcrypto-build-prod:
	just --justfile aes-rustcrypto-r0/Justfile --working-directory aes-rustcrypto-r0 build-prod

aes-rustcrypto-dev:
	just --justfile aes-rustcrypto-r0/Justfile --working-directory aes-rustcrypto-r0 run-dev

aes-rustcrypto-prod:
	just --justfile aes-rustcrypto-r0/Justfile --working-directory aes-rustcrypto-r0 run-prod

aes-rustcrypto-native-dev:
	just --justfile aes-rustcrypto-r0/Justfile --working-directory aes-rustcrypto-r0 run-native-dev

aes-rustcrypto-native-prod:
	just --justfile aes-rustcrypto-r0/Justfile --working-directory aes-rustcrypto-r0 run-native-prod

aes-rustcrypto-clean:
	just --justfile aes-rustcrypto-r0/Justfile --working-directory aes-rustcrypto-r0 clean

aes-ctr-build-dev:
	just --justfile aes-ctr-r0/Justfile --working-directory aes-ctr-r0 build-dev

aes-ctr-build-prod:
	just --justfile aes-ctr-r0/Justfile --working-directory aes-ctr-r0 build-prod

aes-ctr-dev:
	just --justfile aes-ctr-r0/Justfile --working-directory aes-ctr-r0 run-dev

aes-ctr-prod:
	just --justfile aes-ctr-r0/Justfile --working-directory aes-ctr-r0 run-prod

aes-ctr-native-dev:
	just --justfile aes-ctr-r0/Justfile --working-directory aes-ctr-r0 run-native-dev

aes-ctr-native-prod:
	just --justfile aes-ctr-r0/Justfile --working-directory aes-ctr-r0 run-native-prod

aes-ctr-clean:
	just --justfile aes-ctr-r0/Justfile --working-directory aes-ctr-r0 clean

lowmc-build-dev:
	just --justfile lowmc-r0/Justfile --working-directory lowmc-r0 build-dev

lowmc-build-prod:
	just --justfile lowmc-r0/Justfile --working-directory lowmc-r0 build-prod

lowmc-dev:
	just --justfile lowmc-r0/Justfile --working-directory lowmc-r0 run-dev

lowmc-prod:
	just --justfile lowmc-r0/Justfile --working-directory lowmc-r0 run-prod

lowmc-native-dev:
	just --justfile lowmc-r0/Justfile --working-directory lowmc-r0 run-native-dev

lowmc-native-prod:
	just --justfile lowmc-r0/Justfile --working-directory lowmc-r0 run-native-prod

lowmc-clean:
	just --justfile lowmc-r0/Justfile --working-directory lowmc-r0 clean

op-build-dev:
	just --justfile operation-bnchmrk-r0/Justfile --working-directory operation-bnchmrk-r0 build-dev

op-build-prod:
	just --justfile operation-bnchmrk-r0/Justfile --working-directory operation-bnchmrk-r0 build-prod

op-dev:
	just --justfile operation-bnchmrk-r0/Justfile --working-directory operation-bnchmrk-r0 run-dev

op-prod:
	just --justfile operation-bnchmrk-r0/Justfile --working-directory operation-bnchmrk-r0 run-prod

op-native-dev:
	just --justfile operation-bnchmrk-r0/Justfile --working-directory operation-bnchmrk-r0 run-native-dev

op-native-prod:
	just --justfile operation-bnchmrk-r0/Justfile --working-directory operation-bnchmrk-r0 run-native-prod

op-clean:
	just --justfile operation-bnchmrk-r0/Justfile --working-directory operation-bnchmrk-r0 clean

salsa-build-dev:
	just --justfile salsa-r0/Justfile --working-directory salsa-r0 build-dev

salsa-build-prod:
	just --justfile salsa-r0/Justfile --working-directory salsa-r0 build-prod

salsa-dev:
	just --justfile salsa-r0/Justfile --working-directory salsa-r0 run-dev

salsa-prod:
	just --justfile salsa-r0/Justfile --working-directory salsa-r0 run-prod

salsa-native-dev:
	just --justfile salsa-r0/Justfile --working-directory salsa-r0 run-native-dev

salsa-native-prod:
	just --justfile salsa-r0/Justfile --working-directory salsa-r0 run-native-prod

salsa-clean:
	just --justfile salsa-r0/Justfile --working-directory salsa-r0 clean

all-build-dev: aes-build-dev aes-rustcrypto-build-dev aes-ctr-build-dev lowmc-build-dev op-build-dev salsa-build-dev

all-build-prod: aes-build-prod aes-rustcrypto-build-prod aes-ctr-build-prod lowmc-build-prod op-build-prod salsa-build-prod

all-dev: aes-dev aes-rustcrypto-dev aes-ctr-dev lowmc-dev op-dev salsa-dev

all-prod: aes-prod aes-rustcrypto-prod aes-ctr-prod lowmc-prod op-prod salsa-prod

all-native-dev: aes-native-dev aes-rustcrypto-native-dev aes-ctr-native-dev lowmc-native-dev op-native-dev salsa-native-dev

all-native-prod: aes-native-prod aes-rustcrypto-native-prod aes-ctr-native-prod lowmc-native-prod op-native-prod salsa-native-prod

bench-list:
	python3 bench-harness/runner.py --list

bench-run:
	python3 bench-harness/runner.py

bench-aggregate:
	python3 bench-harness/aggregate.py

bench-plot:
	python3 bench-harness/plot.py

bench-all: bench-run bench-aggregate bench-plot

bench-clean:
	rm -rf artifacts/benchmarks

clean: aes-clean aes-rustcrypto-clean aes-ctr-clean lowmc-clean op-clean salsa-clean
