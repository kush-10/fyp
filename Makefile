.DEFAULT_GOAL := list

PROJECT_TARGETS := \
    aes-build-dev aes-build-prod aes-dev aes-prod aes-native-dev aes-native-prod aes-clean \
    aes-optimised-build-dev aes-optimised-build-prod aes-optimised-dev aes-optimised-prod aes-optimised-native-dev aes-optimised-native-prod aes-optimised-clean \
    lowmc-build-dev lowmc-build-prod lowmc-dev lowmc-prod lowmc-native-dev lowmc-native-prod lowmc-clean \
    lowmc-optimised-build-dev lowmc-optimised-build-prod lowmc-optimised-dev lowmc-optimised-prod lowmc-optimised-native-dev lowmc-optimised-native-prod lowmc-optimised-clean \
    op-build-dev op-build-prod op-dev op-prod op-native-dev op-native-prod op-clean \
    salsa-build-dev salsa-build-prod salsa-dev salsa-prod salsa-native-dev salsa-native-prod salsa-clean \
    all-build-dev all-build-prod all-dev all-prod all-native-dev all-native-prod \
    bench-list bench-run bench-aggregate bench-plot bench-all bench-clean lowmc-fn-breakdown clean

.PHONY: default list $(PROJECT_TARGETS)

default: list

list: ; @printf "Project commands:\n"; for target in $(PROJECT_TARGETS); do printf "  make %s\n" "$$target"; done

aes-build-dev: ; @$(MAKE) -C aes-r0 build-dev
aes-build-prod: ; @$(MAKE) -C aes-r0 build-prod
aes-dev: ; @$(MAKE) -C aes-r0 run-dev
aes-prod: ; @$(MAKE) -C aes-r0 run-prod
aes-native-dev: ; @$(MAKE) -C aes-r0 run-native-dev
aes-native-prod: ; @$(MAKE) -C aes-r0 run-native-prod
aes-clean: ; @$(MAKE) -C aes-r0 clean

aes-optimised-build-dev: ; @$(MAKE) -C aes-r0-optimised build-dev
aes-optimised-build-prod: ; @$(MAKE) -C aes-r0-optimised build-prod
aes-optimised-dev: ; @$(MAKE) -C aes-r0-optimised run-dev
aes-optimised-prod: ; @$(MAKE) -C aes-r0-optimised run-prod
aes-optimised-native-dev: ; @$(MAKE) -C aes-r0-optimised run-native-dev
aes-optimised-native-prod: ; @$(MAKE) -C aes-r0-optimised run-native-prod
aes-optimised-clean: ; @$(MAKE) -C aes-r0-optimised clean

lowmc-build-dev: ; @$(MAKE) -C lowmc-r0 build-dev
lowmc-build-prod: ; @$(MAKE) -C lowmc-r0 build-prod
lowmc-dev: ; @$(MAKE) -C lowmc-r0 run-dev
lowmc-prod: ; @$(MAKE) -C lowmc-r0 run-prod
lowmc-native-dev: ; @$(MAKE) -C lowmc-r0 run-native-dev
lowmc-native-prod: ; @$(MAKE) -C lowmc-r0 run-native-prod
lowmc-clean: ; @$(MAKE) -C lowmc-r0 clean

lowmc-optimised-build-dev: ; @$(MAKE) -C low-mc-optimised build-dev
lowmc-optimised-build-prod: ; @$(MAKE) -C low-mc-optimised build-prod
lowmc-optimised-dev: ; @$(MAKE) -C low-mc-optimised run-dev
lowmc-optimised-prod: ; @$(MAKE) -C low-mc-optimised run-prod
lowmc-optimised-native-dev: ; @$(MAKE) -C low-mc-optimised run-native-dev
lowmc-optimised-native-prod: ; @$(MAKE) -C low-mc-optimised run-native-prod
lowmc-optimised-clean: ; @$(MAKE) -C low-mc-optimised clean

op-build-dev: ; @$(MAKE) -C operation-bnchmrk-r0 build-dev
op-build-prod: ; @$(MAKE) -C operation-bnchmrk-r0 build-prod
op-dev: ; @$(MAKE) -C operation-bnchmrk-r0 run-dev
op-prod: ; @$(MAKE) -C operation-bnchmrk-r0 run-prod
op-native-dev: ; @$(MAKE) -C operation-bnchmrk-r0 run-native-dev
op-native-prod: ; @$(MAKE) -C operation-bnchmrk-r0 run-native-prod
op-clean: ; @$(MAKE) -C operation-bnchmrk-r0 clean

salsa-build-dev: ; @$(MAKE) -C salsa-r0 build-dev
salsa-build-prod: ; @$(MAKE) -C salsa-r0 build-prod
salsa-dev: ; @$(MAKE) -C salsa-r0 run-dev
salsa-prod: ; @$(MAKE) -C salsa-r0 run-prod
salsa-native-dev: ; @$(MAKE) -C salsa-r0 run-native-dev
salsa-native-prod: ; @$(MAKE) -C salsa-r0 run-native-prod
salsa-clean: ; @$(MAKE) -C salsa-r0 clean

all-build-dev: aes-build-dev aes-optimised-build-dev lowmc-build-dev lowmc-optimised-build-dev op-build-dev salsa-build-dev
all-build-prod: aes-build-prod aes-optimised-build-prod lowmc-build-prod lowmc-optimised-build-prod op-build-prod salsa-build-prod
all-dev: aes-dev aes-optimised-dev lowmc-dev lowmc-optimised-dev op-dev salsa-dev
all-prod: aes-prod aes-optimised-prod lowmc-prod lowmc-optimised-prod op-prod salsa-prod
all-native-dev: aes-native-dev aes-optimised-native-dev lowmc-native-dev lowmc-optimised-native-dev op-native-dev salsa-native-dev
all-native-prod: aes-native-prod aes-optimised-native-prod lowmc-native-prod lowmc-optimised-native-prod op-native-prod salsa-native-prod

bench-list: ; python3 bench-harness/runner.py --list
bench-run: ; python3 bench-harness/runner.py
bench-aggregate: ; python3 bench-harness/aggregate.py
bench-plot: ; python3 bench-harness/plot.py
bench-all: bench-run bench-aggregate bench-plot
bench-clean: ; rm -rf artifacts/benchmarks
lowmc-fn-breakdown: ; python3 bench-harness/lowmc_function_breakdown.py

clean: aes-clean aes-optimised-clean lowmc-clean lowmc-optimised-clean op-clean salsa-clean
