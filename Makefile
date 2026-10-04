# Typical development actions for the alias project.
#
# Commands mirror the scripts in bin/dev/ and bin/support/, reimplemented
# here so that every target is reachable with a single tool (make) and every
# target's purpose is visible at a glance.

.PHONY: help
help:              ## Show this help
	@grep -E '^[a-zA-Z_.-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| sort \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build:             ## Debug build
	cargo build

.PHONY: release
release:           ## Release build (size-optimized)
	cargo build --release

.PHONY: test
test:              ## Run all tests
	cargo test

.PHONY: lint
lint:              ## Run clippy (deny warnings) and check formatting
	cargo clippy --all-targets -- -D warnings
	cargo fmt --check

.PHONY: fmt
fmt:               ## Apply rustfmt to the whole workspace
	cargo fmt

.PHONY: check
check: lint test   ## Full verification: lint + tests
	@echo "ALL CHECKS PASSED"

.PHONY: outdated
outdated:          ## List outdated dependencies (requires 'cargo install cargo-outdated')
	cargo outdated -R

.PHONY: clean
clean:             ## Remove build artifacts
	cargo clean

.PHONY: setup
setup:             ## Install rustup if missing, ensure the stable toolchain is present
	@command -v rustup > /dev/null 2>&1 || { \
		echo "rustup is not installed. Get it from https://rustup.rs/"; \
		exit 1; \
	}
	rustup default stable
