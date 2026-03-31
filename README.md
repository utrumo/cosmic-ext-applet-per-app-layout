# cosmic-ext-applet-per-app-layout

A [COSMIC](https://github.com/pop-os/cosmic-epoch) panel applet that remembers keyboard layout for each application.

![Applet on the COSMIC panel showing per-app keyboard layouts](assets/panel-popup.png)

## Problem

COSMIC Desktop uses a single global keyboard layout. When you switch between
applications, the layout stays the same — so if you were typing in Russian in
one app and switch to a terminal, you have to change the layout manually.

This applet fixes that by automatically saving and restoring the keyboard layout
per application window.

## Features

- Automatic per-window keyboard layout tracking
- Layout restored on window focus
- Detects manual layout changes via polling
- Persists layouts across restarts (per app\_id via `cosmic-config` state)
- Panel button shows the current active layout
- Popup with a list of remembered applications
- Multi-monitor support with focus event deduplication
- `--register` / `--unregister` CLI flags for panel management
- Internationalization support (i18n via Fluent)

## Requirements

- [COSMIC Desktop Environment](https://github.com/pop-os/cosmic-epoch)
- Rust 1.82+ (stable)

## Installation

### From source

```sh
git clone https://github.com/utrumo/cosmic-ext-applet-per-app-layout.git
cd cosmic-ext-applet-per-app-layout
make install
```

This builds a release binary, installs it to `~/.local/bin`, registers the
applet in the COSMIC panel, and sets up the desktop entry and icon.

After installation, restart the panel:

```sh
killall cosmic-panel
```

cosmic-session will restart it automatically.

### System-wide installation

```sh
sudo make PREFIX=/usr install
```

### Packaging

```sh
make DESTDIR=/tmp/pkg PREFIX=/usr install
```

When `DESTDIR` is set, panel registration and state directory cleanup are
skipped — the package manager should handle those.

### Uninstall

```sh
make uninstall
```

## Usage

Once installed, the applet appears in the panel and shows the current keyboard
layout (e.g. **US** or **RU**). Click on it to see a popup with all remembered
applications and their layouts.

The applet works automatically — just switch between windows and it will save
and restore layouts for you.

### CLI

```sh
cosmic-ext-applet-per-app-layout --register    # add to panel
cosmic-ext-applet-per-app-layout --unregister  # remove from panel
```

## How it works

1. A background thread connects to the Wayland compositor via the
   `zcosmic-toplevel-info-unstable-v1` protocol and tracks window focus/close
   events.
2. When a window loses focus, the current XKB layout is saved for that window.
3. When a window gains focus, the saved layout is restored by rewriting
   `~/.config/cosmic/com.system76.CosmicComp/v1/xkb_config` — the compositor
   picks up the change via inotify.
4. A 250ms polling loop detects manual layout switches so the applet stays in
   sync.

## Building

```sh
cargo build --release    # or: make build
```

### Linting

```sh
make lint     # fast: rustfmt + clippy
make check    # full: fmt + clippy + cargo-audit + cargo-deny + cargo-udeps
make fix      # auto-fix formatting and clippy issues
```

## Contributing

Contributions are welcome! Please make sure `make check` passes before
submitting a pull request.

## License

[GPL-3.0](https://www.gnu.org/licenses/gpl-3.0.html)
