# ToplevelInfoHandler Implementation Design

## Summary

Replace the stub `wayland_handler.rs` with a real implementation that tracks focused windows via Wayland `zcosmic-toplevel-info-unstable-v1` protocol and sends focus events to the main application.

## Architecture (Approach B: Separated)

```
src/
  wayland_state.rs  (NEW)  — AppData struct + trait implementations
  wayland_handler.rs       — thread spawn + calloop event loop
  wayland_subscription.rs  — unchanged (mpsc bridge to iced)
  app.rs                   — unchanged (handles Message::ToplevelFocused)
```

## Component: wayland_state.rs

### AppData struct

```rust
pub(crate) struct AppData {
    pub exit: bool,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub toplevel_info_state: ToplevelInfoState,
    pub tx: mpsc::UnboundedSender<WaylandUpdate>,
}
```

### Trait implementations

1. **ProvidesRegistryState** — required by sctk, delegates to `registry_state`
2. **OutputHandler** — required by ToplevelInfoState (toplevel-to-output binding); all methods are no-ops
3. **ToplevelInfoHandler** — core logic:
   - `new_toplevel()` — log new window, check if already Activated
   - `update_toplevel()` — check `State::Activated` in ToplevelInfo.state, send `WaylandUpdate::Focused { app_id, title }` via channel
   - `toplevel_closed()` — log closure
4. **Delegate macros**: `delegate_output!`, `delegate_registry!`, `delegate_toplevel_info!`

### Focus detection

```rust
fn update_toplevel(&mut self, conn, qh, toplevel) {
    if let Some(info) = self.toplevel_info_state.info(toplevel) {
        if info.state.contains(&zcosmic_toplevel_handle_v1::State::Activated) {
            let _ = self.tx.send(WaylandUpdate::Focused {
                app_id: info.app_id.clone(),
                title: info.title.clone(),
            });
        }
    }
}
```

## Component: wayland_handler.rs

### spawn_wayland_handler()

1. Connect to Wayland via `X_PRIVILEGED_WAYLAND_SOCKET` (fallback: `Connection::connect_to_env()`)
2. `registry_queue_init()` to get globals
3. Create `calloop::EventLoop<AppData>`
4. Create `WaylandSource` from connection + event queue, insert into calloop
5. Initialize `AppData` with `RegistryState`, `OutputState`, `ToplevelInfoState`
6. Run `event_loop.dispatch(None, &mut app_data)` in a loop

### Pattern

Follows the established pattern from `libcosmic/src/applet/token/wayland_handler.rs`:
- calloop-based event loop (not blocking_dispatch)
- WaylandSource integration
- X_PRIVILEGED_WAYLAND_SOCKET support for unstable protocols

## Cargo.toml changes

- Update `wayland-client` from `0.30` to match cosmic-client-toolkit's `0.31`
- Use re-exports from `cosmic-client-toolkit` where possible (`sctk`, `wayland_client`, `calloop`)
- May need `smithay-client-toolkit` with output feature

## Error handling

- `ToplevelInfoState::try_new()` instead of `new()` — graceful fallback if protocol unavailable
- Log errors via `tracing::error!`, don't panic in the handler thread
- Channel send errors (receiver dropped) — break the loop

## Testing

- `cargo check` / `cargo build --release` — compilation verification
- Manual: `sudo just install`, add to panel, switch windows, observe logs
