NAME := cosmic-ext-applet-per-app-layout
APPID := io.github.utrumo.CosmicExtAppletPerAppLayout

CARGO_TARGET_DIR ?= target
BIN_SRC := $(CARGO_TARGET_DIR)/release/$(NAME)

PREFIX ?= $(HOME)/.local
BIN_DST := $(PREFIX)/bin/$(NAME)
SHARE_DST := $(PREFIX)/share
DESKTOP_DST := $(SHARE_DST)/applications/$(APPID).desktop
ICON_DST := $(SHARE_DST)/icons/hicolor/scalable/apps/$(APPID)-symbolic.svg

STATE_HOME ?= $(or $(XDG_STATE_HOME),$(HOME)/.local/state)
STATE_DIR := $(STATE_HOME)/cosmic/$(APPID)

.PHONY: build install uninstall clean check fmt clippy audit deny udeps lint fix

# Build release binary
build:
	cargo build --release

# Install binary, icon, desktop file, and register in panel
install: build
	install -Dm0755 $(BIN_SRC) $(BIN_DST)
	install -Dm0644 data/$(APPID)-symbolic.svg $(ICON_DST)
	install -Dm0644 /dev/null $(DESKTOP_DST)
	sed 's|^Exec=.*|Exec=$(BIN_DST)|' data/$(APPID).desktop > $(DESKTOP_DST)
	$(BIN_DST) --register
	@echo "Installed to $(PREFIX)"

# Unregister from panel, remove files and state
uninstall:
	-$(BIN_DST) --unregister
	rm -f $(BIN_DST)
	rm -f $(ICON_DST)
	rm -f $(DESKTOP_DST)
	rm -rf $(STATE_DIR)
	@echo "Uninstalled $(NAME)"
	@echo "Run 'killall cosmic-panel' to reload the panel."

# Remove build artifacts
clean:
	cargo clean

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
