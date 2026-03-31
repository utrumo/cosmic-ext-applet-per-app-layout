use std::os::{
    fd::{FromRawFd, RawFd},
    unix::net::UnixStream,
};
use std::time::Duration;

use cosmic_client_toolkit::{
    sctk::{
        output::OutputState,
        reexports::{calloop, calloop_wayland_source::WaylandSource},
        registry::RegistryState,
    },
    toplevel_info::ToplevelInfoState,
    wayland_client::{globals::registry_queue_init, Connection},
};
use tokio::sync::mpsc;

use crate::wayland_state::AppData;

#[derive(Debug, Clone)]
pub enum WaylandUpdate {
    Focused { app_id: String, identifier: String },
    Closed { identifier: String },
}

pub fn spawn_wayland_handler(
    tx: mpsc::UnboundedSender<WaylandUpdate>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(e) = run_wayland_loop(tx) {
            tracing::error!("Wayland handler error: {e}");
        }
    })
}

#[allow(unsafe_code)]
fn run_wayland_loop(tx: mpsc::UnboundedSender<WaylandUpdate>) -> anyhow::Result<()> {
    let socket = std::env::var("X_PRIVILEGED_WAYLAND_SOCKET")
        .ok()
        .and_then(|fd| {
            fd.parse::<RawFd>().ok().map(|fd| {
                // SAFETY: cosmic-panel sets X_PRIVILEGED_WAYLAND_SOCKET to a valid
                // Wayland socket fd. We take ownership; it will be closed on drop.
                unsafe { UnixStream::from_raw_fd(fd) }
            })
        });

    let conn = if let Some(socket) = socket {
        Connection::from_socket(socket)?
    } else {
        Connection::connect_to_env()?
    };

    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let mut event_loop = calloop::EventLoop::<AppData>::try_new()?;
    let wayland_source = WaylandSource::new(conn, event_queue);
    wayland_source
        .insert(event_loop.handle())
        .map_err(|e| anyhow::anyhow!("Failed to insert wayland source: {e}"))?;

    let registry_state = RegistryState::new(&globals);
    let toplevel_info_state = ToplevelInfoState::try_new(&registry_state, &qh);

    let Some(toplevel_info_state) = toplevel_info_state else {
        tracing::error!(
            "zcosmic-toplevel-info protocol not available. \
             Is X_PRIVILEGED_WAYLAND_SOCKET set?"
        );
        anyhow::bail!("ToplevelInfoState not available");
    };

    tracing::info!("Wayland handler started, tracking toplevel focus events");

    let mut app_data = AppData {
        registry_state,
        output_state: OutputState::new(&globals, &qh),
        toplevel_info_state,
        tx,
        last_focused_app: None,
        last_focus_time: None,
    };

    loop {
        // Timeout allows detecting channel closure (subscription restart)
        event_loop.dispatch(Some(Duration::from_secs(1)), &mut app_data)?;

        if app_data.tx.is_closed() {
            tracing::info!("Wayland handler shutting down (channel closed)");
            break;
        }
    }

    Ok(())
}
