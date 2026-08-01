SHELL := /bin/sh

CARGO ?= cargo
PYTHON ?= python3
FUZZ_SECONDS ?= 60

.PHONY: all build fmt-check lint test parity unicode-sweep fuzz bench verify

all: build

# Port Mortem's required one-command build. The produced executable is
# target/release/inflection-jsonl.
build:
	$(CARGO) build --release --locked

fmt-check:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --locked --all-targets -- -D warnings

test:
	$(CARGO) test --locked --all-targets

parity: build
	$(PYTHON) scripts/parity.py

unicode-sweep: build
	$(PYTHON) scripts/unicode_sweep.py

fuzz: build
	$(PYTHON) fuzz/differential.py --seconds $(FUZZ_SECONDS)

# Benchmarks are evidence tied to a declared environment, not a correctness
# gate, so they remain separate from verify.
bench: build
	$(PYTHON) bench/benchmark.py

verify:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --locked --all-targets -- -D warnings
	$(CARGO) test --locked --all-targets
	$(MAKE) parity
	$(MAKE) unicode-sweep
	$(MAKE) fuzz
