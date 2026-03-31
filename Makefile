NAME := cosmic-ext-applet-per-app-layout
APPID := io.github.utrumo.CosmicExtAppletPerAppLayout

CARGO_TARGET_DIR ?= target
BIN_SRC := $(CARGO_TARGET_DIR)/release/$(NAME)

DESTDIR ?=
PREFIX ?= /usr
SHARE_DST := $(PREFIX)/share

BIN_DST := $(PREFIX)/bin/$(NAME)
DESKTOP_DST := $(SHARE_DST)/applications/$(APPID).desktop
ICON_DST := $(SHARE_DST)/icons/hicolor/scalable/apps/$(APPID)-symbolic.svg

.PHONY: build install uninstall clean check fmt clippy audit deny udeps lint fix

# Build release binary
build:
	cargo build --release

# Install binary, icon, desktop file, and register in panel
install:
	@[ -f $(BIN_SRC) ] || { echo "Run 'make build' first"; exit 1; }
	install -Dm0755 $(BIN_SRC) $(DESTDIR)$(BIN_DST)
	install -Dm0644 data/$(APPID)-symbolic.svg $(DESTDIR)$(ICON_DST)
	install -Dm0644 data/$(APPID).desktop $(DESTDIR)$(DESKTOP_DST)
	@if [ -z "$(DESTDIR)" ] && [ -n "$$SUDO_USER" ]; then \
	    su "$$SUDO_USER" -c "$(BIN_DST) --register"; \
	    su "$$SUDO_USER" -c "killall cosmic-panel 2>/dev/null || true"; \
	  elif [ -z "$(DESTDIR)" ]; then \
	    echo "Run '$(NAME) --register && killall cosmic-panel' to activate."; \
	  fi
	@echo "Installed to $(DESTDIR)$(PREFIX)"

# Unregister from panel and remove files
uninstall:
	@-if [ -z "$(DESTDIR)" ] && [ -n "$$SUDO_USER" ]; then \
	    su "$$SUDO_USER" -c "$(BIN_DST) --unregister" || true; \
	    STATE_DIR=$$(su "$$SUDO_USER" -c 'echo $${XDG_STATE_HOME:-$$HOME/.local/state}')/cosmic/$(APPID); \
	    rm -rf "$$STATE_DIR" 2>/dev/null || true; \
	  fi
	rm -f $(DESTDIR)$(BIN_DST)
	rm -f $(DESTDIR)$(ICON_DST)
	rm -f $(DESTDIR)$(DESKTOP_DST)
	@echo "Uninstalled $(NAME)"

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
