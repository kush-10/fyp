.DEFAULT_GOAL := list

PROJECT_TARGETS := \
    aes-build-dev aes-build-prod aes-dev aes-prod aes-native-dev aes-native-prod aes-clean \
    aes-rustcrypto-build-dev aes-rustcrypto-build-prod aes-rustcrypto-dev aes-rustcrypto-prod aes-rustcrypto-native-dev aes-rustcrypto-native-prod aes-rustcrypto-clean \
    aes-ctr-build-dev aes-ctr-build-prod aes-ctr-dev aes-ctr-prod aes-ctr-native-dev aes-ctr-native-prod aes-ctr-clean \
    lowmc-build-dev lowmc-build-prod lowmc-dev lowmc-prod lowmc-native-dev lowmc-native-prod lowmc-clean \
    op-build-dev op-build-prod op-dev op-prod op-native-dev op-native-prod op-clean \
    salsa-build-dev salsa-build-prod salsa-dev salsa-prod salsa-native-dev salsa-native-prod salsa-clean \
    all-build-dev all-build-prod all-dev all-prod all-native-dev all-native-prod \
    bench-list bench-run bench-aggregate bench-plot bench-all bench-clean clean

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

aes-rustcrypto-build-dev: ; @$(MAKE) -C aes-rustcrypto-r0 build-dev
aes-rustcrypto-build-prod: ; @$(MAKE) -C aes-rustcrypto-r0 build-prod
aes-rustcrypto-dev: ; @$(MAKE) -C aes-rustcrypto-r0 run-dev
aes-rustcrypto-prod: ; @$(MAKE) -C aes-rustcrypto-r0 run-prod
aes-rustcrypto-native-dev: ; @$(MAKE) -C aes-rustcrypto-r0 run-native-dev
aes-rustcrypto-native-prod: ; @$(MAKE) -C aes-rustcrypto-r0 run-native-prod
aes-rustcrypto-clean: ; @$(MAKE) -C aes-rustcrypto-r0 clean

aes-ctr-build-dev: ; @$(MAKE) -C aes-ctr-r0 build-dev
aes-ctr-build-prod: ; @$(MAKE) -C aes-ctr-r0 build-prod
aes-ctr-dev: ; @$(MAKE) -C aes-ctr-r0 run-dev
aes-ctr-prod: ; @$(MAKE) -C aes-ctr-r0 run-prod
aes-ctr-native-dev: ; @$(MAKE) -C aes-ctr-r0 run-native-dev
aes-ctr-native-prod: ; @$(MAKE) -C aes-ctr-r0 run-native-prod
aes-ctr-clean: ; @$(MAKE) -C aes-ctr-r0 clean

lowmc-build-dev: ; @$(MAKE) -C lowmc-r0 build-dev
lowmc-build-prod: ; @$(MAKE) -C lowmc-r0 build-prod
lowmc-dev: ; @$(MAKE) -C lowmc-r0 run-dev
lowmc-prod: ; @$(MAKE) -C lowmc-r0 run-prod
lowmc-native-dev: ; @$(MAKE) -C lowmc-r0 run-native-dev
lowmc-native-prod: ; @$(MAKE) -C lowmc-r0 run-native-prod
lowmc-clean: ; @$(MAKE) -C lowmc-r0 clean

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

all-build-dev: aes-build-dev aes-rustcrypto-build-dev aes-ctr-build-dev lowmc-build-dev op-build-dev salsa-build-dev
all-build-prod: aes-build-prod aes-rustcrypto-build-prod aes-ctr-build-prod lowmc-build-prod op-build-prod salsa-build-prod
all-dev: aes-dev aes-rustcrypto-dev aes-ctr-dev lowmc-dev op-dev salsa-dev
all-prod: aes-prod aes-rustcrypto-prod aes-ctr-prod lowmc-prod op-prod salsa-prod
all-native-dev: aes-native-dev aes-rustcrypto-native-dev aes-ctr-native-dev lowmc-native-dev op-native-dev salsa-native-dev
all-native-prod: aes-native-prod aes-rustcrypto-native-prod aes-ctr-native-prod lowmc-native-prod op-native-prod salsa-native-prod

bench-list: ; python3 bench-harness/runner.py --list
bench-run: ; python3 bench-harness/runner.py
bench-aggregate: ; python3 bench-harness/aggregate.py
bench-plot: ; python3 bench-harness/plot.py
bench-all: bench-run bench-aggregate bench-plot
bench-clean: ; rm -rf artifacts/benchmarks

clean: aes-clean aes-rustcrypto-clean aes-ctr-clean lowmc-clean op-clean salsa-clean
