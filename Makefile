.PHONY: check fmt clippy audit deny udeps lint fix

# Run all checks including unused dependencies
check: fmt clippy audit deny udeps

# Format check
fmt:
	cargo fmt --all -- --check

# Clippy (deny warnings)
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Security audit
audit:
	cargo audit

# Dependency policy (skip licenses — COSMIC crates lack Cargo.toml license fields)
deny:
	cargo deny check advisories bans sources

# Unused dependencies (requires nightly; use full paths to bypass system rustc)
NIGHTLY_CARGO := $(shell rustup which cargo --toolchain nightly 2>/dev/null)
NIGHTLY_RUSTC := $(shell rustup which rustc --toolchain nightly 2>/dev/null)
udeps:
ifdef NIGHTLY_CARGO
	RUSTC=$(NIGHTLY_RUSTC) $(NIGHTLY_CARGO) udeps --all-targets
else
	@echo "SKIP: nightly toolchain not installed (rustup toolchain install nightly)"
endif

# Fix auto-fixable issues
fix:
	cargo fmt --all
	cargo clippy --all-targets --all-features --fix --allow-dirty

# Lint = fmt + clippy only (fast, no network)
lint: fmt clippy
