use std::os::{
    fd::{FromRawFd, RawFd},
    unix::net::UnixStream,
};

use cosmic_client_toolkit::{
    sctk::{
        output::OutputState,
        registry::RegistryState,
        reexports::{calloop, calloop_wayland_source::WaylandSource},
    },
    toplevel_info::ToplevelInfoState,
    wayland_client::{Connection, globals::registry_queue_init},
};
use tokio::sync::mpsc;

use crate::wayland_state::AppData;

#[derive(Debug, Clone)]
pub enum WaylandUpdate {
    Focused { app_id: String, title: String },
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

fn run_wayland_loop(tx: mpsc::UnboundedSender<WaylandUpdate>) -> anyhow::Result<()> {
    let socket = std::env::var("X_PRIVILEGED_WAYLAND_SOCKET")
        .ok()
        .and_then(|fd| {
            fd.parse::<RawFd>()
                .ok()
                .map(|fd| unsafe { UnixStream::from_raw_fd(fd) })
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

    let toplevel_info_state = match toplevel_info_state {
        Some(state) => state,
        None => {
            tracing::error!(
                "zcosmic-toplevel-info protocol not available. \
                 Is X_PRIVILEGED_WAYLAND_SOCKET set?"
            );
            anyhow::bail!("ToplevelInfoState not available");
        }
    };

    tracing::info!("Wayland handler started, tracking toplevel focus events");

    let mut app_data = AppData {
        exit: false,
        registry_state,
        output_state: OutputState::new(&globals, &qh),
        toplevel_info_state,
        tx,
    };

    loop {
        if app_data.exit {
            break;
        }
        event_loop.dispatch(None, &mut app_data)?;
    }

    Ok(())
}
