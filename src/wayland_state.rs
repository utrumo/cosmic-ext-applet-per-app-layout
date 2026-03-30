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
        if let Some(info) = self.toplevel_info_state.info(toplevel) {
            tracing::debug!("New toplevel: app_id={}, title={}", info.app_id, info.title);
            if info
                .state
                .contains(&zcosmic_toplevel_handle_v1::State::Activated)
            {
                let _ = self.tx.send(WaylandUpdate::Focused {
                    app_id: info.app_id.clone(),
                    title: info.title.clone(),
                });
            }
        }
    }

    fn update_toplevel(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    ) {
        if let Some(info) = self.toplevel_info_state.info(toplevel) {
            if info
                .state
                .contains(&zcosmic_toplevel_handle_v1::State::Activated)
            {
                tracing::debug!(
                    "Toplevel focused: app_id={}, title={}",
                    info.app_id,
                    info.title
                );
                let _ = self.tx.send(WaylandUpdate::Focused {
                    app_id: info.app_id.clone(),
                    title: info.title.clone(),
                });
            }
        }
    }

    fn toplevel_closed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        toplevel: &ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    ) {
        if let Some(info) = self.toplevel_info_state.info(toplevel) {
            tracing::debug!(
                "Toplevel closed: app_id={}, title={}",
                info.app_id,
                info.title
            );
        }
    }
}

sctk::delegate_output!(AppData);
sctk::delegate_registry!(AppData);
cosmic_client_toolkit::delegate_toplevel_info!(AppData);
