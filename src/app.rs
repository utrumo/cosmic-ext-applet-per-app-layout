use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::Subscription;
use cosmic::{Application, Element};
use cosmic_config::{ConfigGet, ConfigSet};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::xkb;

const APP_ID: &str = "io.github.utrumo.CosmicExtAppletPerAppLayout";
const STATE_VERSION: u64 = 1;
const PANEL_CONFIG_PREFIX: &str = "com.system76.CosmicPanel.";
const PANEL_CONFIG_VERSION: u64 = 1;

pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<PerAppLayoutApplet>(())
}

const MAIN_PANEL_CONFIG: &str = "com.system76.CosmicPanel.Panel";

/// Add this applet to the right wing of the main COSMIC panel.
pub fn register() {
    let Ok(config) = cosmic_config::Config::new(MAIN_PANEL_CONFIG, PANEL_CONFIG_VERSION) else {
        tracing::error!("Cannot open {MAIN_PANEL_CONFIG} config");
        return;
    };

    let mut wings: Option<(Vec<String>, Vec<String>)> = config
        .get("plugins_wings")
        .unwrap_or_else(|_| Some((Vec::new(), Vec::new())));

    let pair = wings.get_or_insert_with(|| (Vec::new(), Vec::new()));

    if pair.0.iter().any(|s| s == APP_ID) || pair.1.iter().any(|s| s == APP_ID) {
        tracing::info!("Already registered in {MAIN_PANEL_CONFIG}");
        return;
    }

    pair.1.insert(0, APP_ID.to_owned());

    if let Err(e) = config.set("plugins_wings", &wings) {
        tracing::error!("Failed to register in {MAIN_PANEL_CONFIG}: {e}");
    } else {
        tracing::info!("Registered in right wing of {MAIN_PANEL_CONFIG}");
    }
}

/// Remove this applet from all COSMIC panel configurations.
///
/// Scans `$XDG_CONFIG_HOME/cosmic/com.system76.CosmicPanel.*/` and removes
/// `APP_ID` from `plugins_wings` and `plugins_center` using the same
/// `cosmic_config` API that wrote them — no manual RON parsing needed.
pub fn unregister() {
    let cosmic_dir = cosmic_config_dir();
    let Ok(entries) = std::fs::read_dir(&cosmic_dir) else {
        tracing::warn!("Cannot read {}", cosmic_dir.display());
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.starts_with(PANEL_CONFIG_PREFIX) {
            continue;
        }

        let Ok(config) = cosmic_config::Config::new(name, PANEL_CONFIG_VERSION) else {
            continue;
        };

        unregister_from_wings(&config, name);
        unregister_from_center(&config, name);
    }
}

fn cosmic_config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map_or_else(
            |_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/root"));
                PathBuf::from(home).join(".config")
            },
            PathBuf::from,
        )
        .join("cosmic")
}

fn unregister_from_wings(config: &cosmic_config::Config, panel_name: &str) {
    let Ok(mut wings) = config.get::<Option<(Vec<String>, Vec<String>)>>("plugins_wings") else {
        return;
    };

    let Some(ref mut pair) = wings else { return };
    let left_before = pair.0.len();
    let right_before = pair.1.len();
    pair.0.retain(|s| s != APP_ID);
    pair.1.retain(|s| s != APP_ID);

    if pair.0.len() < left_before || pair.1.len() < right_before {
        if let Err(e) = config.set("plugins_wings", &wings) {
            tracing::error!("Failed to update plugins_wings for {panel_name}: {e}");
        } else {
            tracing::info!("Removed from plugins_wings in {panel_name}");
        }
    }
}

fn unregister_from_center(config: &cosmic_config::Config, panel_name: &str) {
    let Ok(mut center) = config.get::<Option<Vec<String>>>("plugins_center") else {
        return;
    };

    let Some(ref mut list) = center else { return };
    let before = list.len();
    list.retain(|s| s != APP_ID);

    if list.len() < before {
        if let Err(e) = config.set("plugins_center", &center) {
            tracing::error!("Failed to update plugins_center for {panel_name}: {e}");
        } else {
            tracing::info!("Removed from plugins_center in {panel_name}");
        }
    }
}

#[derive(Default)]
struct PerAppLayoutApplet {
    core: Core,
    popup: Option<Id>,
    layout_map: BTreeMap<String, String>, // identifier → layout (runtime, per-window)
    app_names: BTreeMap<String, String>,  // identifier → app_id (for display)
    persisted_layouts: BTreeMap<String, String>, // app_id → layout (survives restart)
    config_state: Option<cosmic_config::Config>, // state handle for persistence
    current_app: Option<String>,          // current identifier
    current_layout: String,
    last_write_time: Option<Instant>, // cooldown after our own xkb writes
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleWindow,
    PopupClosed(Id),
    ToplevelFocused { app_id: String, identifier: String },
    ToplevelClosed(String),
    PollLayout,
}

impl PerAppLayoutApplet {
    fn save_persisted_layouts(&self) {
        if let Some(ref config) = self.config_state {
            if let Err(e) = config.set("app_layouts", &self.persisted_layouts) {
                tracing::warn!("Failed to save app layouts: {e}");
            }
        }
    }
}

impl Application for PerAppLayoutApplet {
    type Executor = cosmic::executor::Default;
    type Message = Message;
    type Flags = ();

    const APP_ID: &'static str = APP_ID;

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let current_layout = if let Some(cfg) = xkb::read_xkb_config() {
            let layouts = xkb::available_layouts(&cfg);
            let active = xkb::active_layout(&cfg).unwrap_or_else(|| "us".to_owned());
            tracing::info!("XKB layouts: {:?}, active: {}", layouts, active);
            active
        } else {
            tracing::warn!("Could not read XKB config, using defaults");
            "us".to_owned()
        };

        let config_state = cosmic_config::Config::new_state(APP_ID, STATE_VERSION).ok();
        let persisted_layouts = config_state
            .as_ref()
            .and_then(|s| s.get::<BTreeMap<String, String>>("app_layouts").ok())
            .unwrap_or_default();
        tracing::info!("Loaded {} persisted app layouts", persisted_layouts.len());

        let applet = Self {
            core,
            persisted_layouts,
            config_state,
            current_layout,
            ..Default::default()
        };
        (applet, Task::none())
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleWindow => {
                if let Some(id) = self.popup.take() {
                    return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(
                        id,
                    );
                }
                let new_id = Id::unique();
                self.popup.replace(new_id);
                let Some(main_id) = self.core.main_window_id() else {
                    return Task::none();
                };
                let popup_settings = self
                    .core
                    .applet
                    .get_popup_settings(main_id, new_id, None, None, None);
                return cosmic::iced::platform_specific::shell::commands::popup::get_popup(
                    popup_settings,
                );
            }
            Message::PopupClosed(id) => {
                self.popup.take_if(|stored| stored == &id);
            }
            Message::ToplevelFocused { app_id, identifier } => {
                self.app_names.insert(identifier.clone(), app_id.clone());

                // Save current layout for the OUTGOING window before switching
                if let Some(ref old_app) = self.current_app {
                    self.layout_map
                        .insert(old_app.clone(), self.current_layout.clone());
                }

                // Update current_app FIRST so PollLayout won't mis-attribute
                self.current_app = Some(identifier.clone());

                // Restore stored layout: runtime map first, then persisted by app_id
                let desired = self
                    .layout_map
                    .get(&identifier)
                    .or_else(|| self.persisted_layouts.get(&app_id))
                    .cloned();

                if let Some(desired) = desired {
                    // Seed runtime map so subsequent focus switches skip persisted lookup
                    self.layout_map
                        .entry(identifier)
                        .or_insert_with(|| desired.clone());

                    if desired != self.current_layout {
                        if let Some(cfg) = xkb::read_xkb_config() {
                            if let Some(new_cfg) = xkb::make_layout_active(&cfg, &desired) {
                                if xkb::write_xkb_config(&new_cfg) {
                                    tracing::debug!("Restored '{}' for '{}'", desired, app_id);
                                    self.current_layout = desired;
                                    self.last_write_time = Some(Instant::now());
                                }
                            }
                        }
                    }
                }
            }
            Message::ToplevelClosed(identifier) => {
                self.layout_map.remove(&identifier);
                self.app_names.remove(&identifier);
            }
            Message::PollLayout => {
                // Skip polling during write cooldown to avoid race with our own writes
                if let Some(t) = self.last_write_time {
                    if t.elapsed() < Duration::from_millis(500) {
                        return Task::none();
                    }
                }

                // Detect layout changes (user toggled via hotkey)
                if let Some(active) =
                    xkb::read_xkb_config().and_then(|cfg| xkb::active_layout(&cfg))
                {
                    if active != self.current_layout {
                        self.current_layout.clone_from(&active);
                        // Save for current window (runtime)
                        if let Some(ref id) = self.current_app {
                            self.layout_map.insert(id.clone(), active.clone());
                            // Save by app_id (persistent)
                            if let Some(app_id) = self.app_names.get(id) {
                                self.persisted_layouts
                                    .insert(app_id.clone(), active.clone());
                                self.save_persisted_layouts();
                            }
                            tracing::debug!("Layout → '{}' for '{}'", active, id);
                        }
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&'_ self) -> Element<'_, Message> {
        let label = self.current_layout.to_uppercase();
        let btn = cosmic::iced_widget::button(cosmic::widget::text(label))
            .on_press(Message::ToggleWindow);

        cosmic::widget::autosize::autosize(btn, cosmic::widget::Id::unique()).into()
    }

    fn view_window(&'_ self, _id: Id) -> Element<'_, Message> {
        let mut app_items = Vec::new();
        for (identifier, layout) in &self.layout_map {
            let display_name = self.app_names.get(identifier).map_or("?", String::as_str);
            let item = cosmic::iced_widget::row![
                cosmic::widget::text(display_name).width(cosmic::iced::Length::Fill),
                cosmic::widget::text(layout.to_uppercase()),
            ]
            .padding([8, 12])
            .spacing(8);

            app_items.push(item.into());
        }

        let content = if app_items.is_empty() {
            cosmic::iced_widget::column![cosmic::widget::text(crate::fl!("message-no-apps")),]
                .padding([16, 12])
        } else {
            cosmic::iced_widget::column(app_items).padding([4, 0])
        };

        self.core
            .applet
            .popup_container(cosmic::widget::container(content))
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            crate::wayland_subscription::wayland_subscription(),
            cosmic::iced::time::every(Duration::from_millis(250)).map(|_| Message::PollLayout),
        ])
    }
}
