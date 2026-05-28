## Configuration
## =============

.DEFAULT_GOAL := help
.DELETE_ON_ERROR: ;
.SECONDARY:
.SUFFIXES: ;
.PHONY: .FORCE

## Verbs
## =====

help:
	@mkhelp $(firstword $(MAKEFILE_LIST))

## Checks
## ------

check: check_build check_docs check_format check_generated_files check_lint check_tests

## _
check_build:
	cargo build \
		--locked \
		--workspace
.PHONY: check_build

## _
check_docs:
	RUSTDOCFLAGS="-Dwarnings" cargo doc \
		--document-private-items \
		--locked \
		--no-deps \
		--workspace
.PHONY: check_docs

## _
check_format: check_format_nix check_format_rs
.PHONY: check_format

check_format_nix:
	fd --type f '.*+.nix$$' \
	| xargs nixfmt --check
.PHONY: check_format_nix

check_format_rs:
	cargo fmt -- --check --config imports_granularity=Module,group_imports=StdExternalCrate
.PHONY: check_format_rs

## _
check_generated_files: Cargo.lock
	git update-index -q --refresh
	git --no-pager diff --exit-code HEAD -- $^
.PHONY: check_generated_files

## _
check_lint:
	cargo clippy \
		--all-targets \
		--locked \
		--no-deps \
		--workspace \
		-- \
		-Dwarnings
.PHONY: check_lint

## _
check_tests:
	cargo test \
		--all-targets \
		--locked \
		--workspace
	@for pkg in $$(cargo metadata --format-version=1 --no-deps -q \
		| jq -r '.packages[].name'); do \
		echo "--- testing $$pkg ---"; \
		cargo test --all-targets --locked -p "$$pkg" || exit 1; \
	done
.PHONY: check_tests

## Fixes
## -----

## _
fix_format: fix_format_nix fix_format_rs ;
.PHONY: fix_format

fix_format_nix:
	fd --type f '.*+.nix$$' \
	| xargs nixfmt
.PHONY: fix_format_nix

fix_format_rs:
	cargo fmt -- --config imports_granularity=Module,group_imports=StdExternalCrate
.PHONY: fix_format_rs

## _
fix_lint:
	cargo clippy --fix
.PHONY: fix_lint


## Nouns
## =====

Cargo.lock: $(wildcard crates/*/Cargo.toml) $(wildcard examples/*/Cargo.toml)
	cargo metadata --format-version=1 > /dev/null
