use cosmic_client_toolkit::{
    cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_handle_v1,
    sctk::{
        self,
        output::{OutputHandler, OutputState},
        registry::{ProvidesRegistryState, RegistryState},
    },
    toplevel_info::{ToplevelInfoHandler, ToplevelInfoState},
    wayland_client::{Connection, QueueHandle, protocol::wl_output},
    wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1,
};
use tokio::sync::mpsc;

use crate::wayland_handler::WaylandUpdate;

pub(crate) struct AppData {
    pub exit: bool,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub toplevel_info_state: ToplevelInfoState,
    pub tx: mpsc::UnboundedSender<WaylandUpdate>,
    pub last_focused_app: Option<String>,
    pub last_focus_time: Option<std::time::Instant>,
}

impl AppData {
    fn toplevel_key(app_id: &str, identifier: &str) -> String {
        if identifier.is_empty() {
            app_id.to_string()
        } else {
            identifier.to_string()
        }
    }

    fn send_focus(&mut self, app_id: &str, identifier: &str) {
        let key = Self::toplevel_key(app_id, identifier);

        // Dedup: skip if same window already focused
        if self.last_focused_app.as_deref() == Some(&*key) {
            return;
        }

        // Debounce: on multi-monitor, compositor sends Activated for both monitors
        // in the same batch (~100µs apart). Only accept the first event per batch.
        let now = std::time::Instant::now();
        if let Some(last_time) = self.last_focus_time {
            if now.duration_since(last_time) < std::time::Duration::from_millis(5) {
                return;
            }
        }

        tracing::info!("Focus: app_id='{}', id='{}'", app_id, key);
        self.last_focused_app = Some(key.clone());
        self.last_focus_time = Some(now);
        let _ = self.tx.send(WaylandUpdate::Focused {
            app_id: app_id.to_string(),
            identifier: key,
        });
    }

    fn send_closed(&self, app_id: &str, identifier: &str) {
        let key = Self::toplevel_key(app_id, identifier);
        tracing::debug!("Toplevel closed: id='{}'", key);
        let _ = self.tx.send(WaylandUpdate::Closed { identifier: key });
    }
}

impl ProvidesRegistryState for AppData {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    sctk::registry_handlers!(OutputState);
}

impl OutputHandler for AppData {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl ToplevelInfoHandler for AppData {
    fn toplevel_info_state(&mut self) -> &mut ToplevelInfoState {
        &mut self.toplevel_info_state
    }

    fn new_toplevel(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    ) {
        let data = self.toplevel_info_state.info(toplevel).map(|info| {
            let activated = info.state.contains(&zcosmic_toplevel_handle_v1::State::Activated);
            (info.app_id.clone(), info.identifier.clone(), activated)
        });
        if let Some((app_id, identifier, true)) = data {
            self.send_focus(&app_id, &identifier);
        }
    }

    fn update_toplevel(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    ) {
        let data = self.toplevel_info_state.info(toplevel).map(|info| {
            let activated = info.state.contains(&zcosmic_toplevel_handle_v1::State::Activated);
            (info.app_id.clone(), info.identifier.clone(), activated)
        });
        if let Some((app_id, identifier, true)) = data {
            self.send_focus(&app_id, &identifier);
        }
    }

    fn toplevel_closed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    ) {
        if let Some(info) = self.toplevel_info_state.info(toplevel) {
            self.send_closed(&info.app_id, &info.identifier);
        }
    }
}

sctk::delegate_output!(AppData);
sctk::delegate_registry!(AppData);
cosmic_client_toolkit::delegate_toplevel_info!(AppData);
