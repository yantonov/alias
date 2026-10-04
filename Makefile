# Typical development actions for the alias project.
#
# Every target is reachable with a single tool (make) and every target's
# purpose is visible at a glance.

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
	@rustup default stable

.PHONY: tag-release
tag-release:       ## Tag a release: set version, commit, tag (usage: make tag-release VERSION=0.3.1)
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make tag-release VERSION=<version>"; \
		echo "Example: make tag-release VERSION=0.3.1"; \
		exit 1; \
	fi
	@if ! printf '%s' "$(VERSION)" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$$'; then \
		echo "A version looks like 1.2.3, got: $(VERSION)"; \
		exit 1; \
	fi
	@if [ "$$(git rev-parse --abbrev-ref HEAD)" != "master" ]; then \
		echo "Releases are cut from master, this is $$(git rev-parse --abbrev-ref HEAD)"; \
		exit 1; \
	fi
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "The working tree is not clean:"; \
		git status --short; \
		exit 1; \
	fi
	@if git rev-parse -q --verify "refs/tags/$(VERSION)" > /dev/null 2>&1; then \
		echo "Tag $(VERSION) already exists"; \
		exit 1; \
	fi
	@VERSION_LINES=$$(grep -c '^version = ' Cargo.toml); \
	if [ "$$VERSION_LINES" != "1" ]; then \
		echo "Expected one 'version =' line in Cargo.toml, found $$VERSION_LINES"; \
		exit 1; \
	fi
	@CURRENT=$$(grep '^version = ' Cargo.toml | sed -E 's/^version = "(.*)"$$/\1/'); \
	if [ "$$CURRENT" = "$(VERSION)" ]; then \
		echo "Cargo.toml is already at $(VERSION)"; \
		exit 1; \
	fi; \
	echo "$$CURRENT -> $(VERSION)"; \
	sed -i.bak -E 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml && rm -f Cargo.toml.bak; \
	cargo check --quiet; \
	git add Cargo.toml Cargo.lock; \
	git commit -m "release $(VERSION)"; \
	git tag -a "$(VERSION)" -m "release $(VERSION)"; \
	echo; \
	echo "Tagged $(VERSION). Nothing pushed yet:"; \
	echo "  git push origin master && git push origin $(VERSION)"; \
	echo "The workflow leaves the release as a draft; publish it, otherwise the"; \
	echo "installer keeps offering the previous one."
