# Rename Project: cosmic-keyboard-context -> cosmic-ext-applet-per-app-layout

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename the project from `cosmic-keyboard-context` to `cosmic-ext-applet-per-app-layout` following COSMIC community naming conventions.

**Architecture:** Pure rename operation across all project files. No code logic changes. The rename touches: Cargo.toml, binary name, APP_ID constant, .desktop file, icon SVG file, i18n fluent file, justfile, and Serena config. Directory rename is done last.

**Tech Stack:** Rust, COSMIC DE ecosystem

**Rename mapping:**
| What | Old | New |
|------|-----|-----|
| Crate name | `cosmic-keyboard-context` | `cosmic-ext-applet-per-app-layout` |
| Binary name | `cosmic-keyboard-context` | `cosmic-ext-applet-per-app-layout` |
| APP_ID | `io.github.utrumo.CosmicKeyboardContext` | `io.github.utrumo.CosmicExtAppletPerAppLayout` |
| Desktop file | `io.github.utrumo.CosmicKeyboardContext.desktop` | `io.github.utrumo.CosmicExtAppletPerAppLayout.desktop` |
| Icon file | `io.github.utrumo.CosmicKeyboardContext-symbolic.svg` | `io.github.utrumo.CosmicExtAppletPerAppLayout-symbolic.svg` |
| Desktop Name | `Keyboard Context` | `Per-App Layout` |
| i18n file | `cosmic_keyboard_context.ftl` | `cosmic_ext_applet_per_app_layout.ftl` |
| Struct name | `KeyboardContextApplet` | `PerAppLayoutApplet` |
| Project dir | `cosmic-keyboard-context/` | `cosmic-ext-applet-per-app-layout/` |

---

### Task 1: Update Cargo.toml

**Files:**
- Modify: `Cargo.toml:2` (package name)
- Modify: `Cargo.toml:7` (description)
- Modify: `Cargo.toml:56` (bin name)

- [ ] **Step 1: Update package name and bin name**

In `Cargo.toml`, change:

```toml
# Line 2
name = "cosmic-ext-applet-per-app-layout"

# Line 7
description = "COSMIC panel applet that remembers keyboard layout for each application"

# Line 56
name = "cosmic-ext-applet-per-app-layout"
```

- [ ] **Step 2: Verify Cargo.toml parses**

Run: `cargo metadata --format-version 1 --no-deps 2>&1 | head -1`
Expected: JSON output (no parse errors)

---

### Task 2: Update APP_ID and struct name in src/app.rs

**Files:**
- Modify: `src/app.rs:11` (APP_ID constant)
- Modify: `src/app.rs:15-16` (run function — applet type)
- Modify: `src/app.rs:19` (struct name)
- Modify: `src/app.rs:50` (impl block)

- [ ] **Step 1: Update APP_ID constant**

In `src/app.rs` line 11, change:

```rust
const APP_ID: &str = "io.github.utrumo.CosmicExtAppletPerAppLayout";
```

- [ ] **Step 2: Rename struct and impl**

In `src/app.rs`, rename all occurrences of `KeyboardContextApplet` to `PerAppLayoutApplet`:

Line 15: `cosmic::applet::run::<PerAppLayoutApplet>(())`
Line 19: `struct PerAppLayoutApplet {`
Line 40: `impl PerAppLayoutApplet {`
Line 50: `impl Application for PerAppLayoutApplet {`
Line 78: `let applet = PerAppLayoutApplet {`

---

### Task 3: Update i18n fluent file

**Files:**
- Rename: `i18n/en/cosmic_keyboard_context.ftl` -> `i18n/en/cosmic_ext_applet_per_app_layout.ftl`

- [ ] **Step 1: Rename the fluent file**

```bash
cd /home/devpa/Projects/cosmic-keyboard-context
git mv i18n/en/cosmic_keyboard_context.ftl i18n/en/cosmic_ext_applet_per_app_layout.ftl
```

- [ ] **Step 2: Update the fluent file contents**

In `i18n/en/cosmic_ext_applet_per_app_layout.ftl`, change:

```fluent
app-title = Per-App Layout
app-description = Remember keyboard layout for each application

message-no-apps = No applications remembered yet
```

Note: The `i18n_embed` / `rust_embed` macro in `i18n.rs` uses `#[folder = "i18n/"]` which loads all .ftl files from the directory. The fluent file name must match the crate name with hyphens replaced by underscores — `cosmic_ext_applet_per_app_layout.ftl` matches crate `cosmic-ext-applet-per-app-layout`.

---

### Task 4: Rename data files (.desktop and icon)

**Files:**
- Rename: `data/io.github.utrumo.CosmicKeyboardContext.desktop` -> `data/io.github.utrumo.CosmicExtAppletPerAppLayout.desktop`
- Rename: `data/io.github.utrumo.CosmicKeyboardContext-symbolic.svg` -> `data/io.github.utrumo.CosmicExtAppletPerAppLayout-symbolic.svg`
- Modify: `.desktop` file contents

- [ ] **Step 1: Rename files**

```bash
cd /home/devpa/Projects/cosmic-keyboard-context
git mv data/io.github.utrumo.CosmicKeyboardContext.desktop data/io.github.utrumo.CosmicExtAppletPerAppLayout.desktop
git mv data/io.github.utrumo.CosmicKeyboardContext-symbolic.svg data/io.github.utrumo.CosmicExtAppletPerAppLayout-symbolic.svg
```

- [ ] **Step 2: Update .desktop file contents**

In `data/io.github.utrumo.CosmicExtAppletPerAppLayout.desktop`:

```ini
[Desktop Entry]
Name=Per-App Layout
Type=Application
Icon=io.github.utrumo.CosmicExtAppletPerAppLayout-symbolic
Exec=cosmic-ext-applet-per-app-layout
Terminal=false
NoDisplay=true
Categories=COSMIC;Utility;
Keywords=keyboard;layout;language;per-app;
X-CosmicApplet=true
X-CosmicHoverPopup=Auto
```

---

### Task 5: Update justfile

**Files:**
- Modify: `justfile:6-7` (NAME and APPID variables)

- [ ] **Step 1: Update variables**

In `justfile`, change lines 6-7:

```just
export NAME := 'cosmic-ext-applet-per-app-layout'
export APPID := 'io.github.utrumo.CosmicExtAppletPerAppLayout'
```

---

### Task 6: Update Serena config

**Files:**
- Modify: `.serena/project.yml:88` (project_name)

- [ ] **Step 1: Update project name**

In `.serena/project.yml` line 88, change:

```yaml
project_name: "cosmic-ext-applet-per-app-layout"
```

---

### Task 7: Regenerate Cargo.lock and verify build

- [ ] **Step 1: Update Cargo.lock**

Run: `cargo generate-lockfile`
Expected: Cargo.lock updated with new package name

- [ ] **Step 2: Run cargo check**

Run: `cargo check 2>&1`
Expected: Compiles without errors

- [ ] **Step 3: Run cargo test**

Run: `cargo test 2>&1`
Expected: All tests pass (10 xkb tests)

---

### Task 8: Clean up old installation and commit

- [ ] **Step 1: Remove old installed files (if present)**

```bash
rm -f ~/.local/bin/cosmic-keyboard-context
rm -f ~/.local/share/applications/io.github.utrumo.CosmicKeyboardContext.desktop
rm -f ~/.local/share/icons/hicolor/scalable/apps/io.github.utrumo.CosmicKeyboardContext-symbolic.svg
```

- [ ] **Step 2: Clean old persisted state (optional)**

Old state lives at `~/.local/state/cosmic/io.github.utrumo.CosmicKeyboardContext/`. It can be removed since the APP_ID changed and the old state won't be found by the new binary:

```bash
rm -rf ~/.local/state/cosmic/io.github.utrumo.CosmicKeyboardContext/
```

- [ ] **Step 3: Commit all changes**

```bash
git add -A
git commit -m "rename: cosmic-keyboard-context -> cosmic-ext-applet-per-app-layout

Follow COSMIC community naming convention (cosmic-ext-applet-*).
Update APP_ID, .desktop, icon, i18n, justfile, and Serena config."
```

---

### Task 9: Rename project directory

This is done LAST and outside the git repo, as it changes the working directory.

- [ ] **Step 1: Rename directory**

```bash
mv ~/Projects/cosmic-keyboard-context ~/Projects/cosmic-ext-applet-per-app-layout
```

- [ ] **Step 2: Update Joplin Index note**

Update the project name and all references in the Joplin "cosmic-keyboard-context" notebook Index note to reflect the new name.

- [ ] **Step 3: Verify final build in new directory**

```bash
cd ~/Projects/cosmic-ext-applet-per-app-layout
cargo build --release 2>&1 | tail -3
```

Expected: `Finished release [optimized] target(s)`

- [ ] **Step 4: Install and test**

```bash
cd ~/Projects/cosmic-ext-applet-per-app-layout
just install
```

Expected: Installed to `~/.local/bin/cosmic-ext-applet-per-app-layout`
