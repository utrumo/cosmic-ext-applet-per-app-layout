use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::Subscription;
use cosmic::{Application, Element};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use crate::xkb;

const APP_ID: &str = "io.github.utrumo.CosmicKeyboardContext";

pub(crate) fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<KeyboardContextApplet>(())
}

#[derive(Default)]
struct KeyboardContextApplet {
    core: Core,
    popup: Option<Id>,
    layout_map: BTreeMap<String, String>,  // identifier → layout (sorted for popup)
    app_names: BTreeMap<String, String>,    // identifier → app_id (for display)
    current_app: Option<String>,           // current identifier
    current_layout: String,
    last_write_time: Option<Instant>,      // cooldown after our own xkb writes
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleWindow,
    PopupClosed(Id),
    ToplevelFocused { app_id: String, identifier: String },
    ToplevelClosed(String),
    PollLayout,
}

impl Application for KeyboardContextApplet {
    type Executor = cosmic::executor::Default;
    type Message = Message;
    type Flags = ();

    const APP_ID: &'static str = APP_ID;

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let current_layout = match xkb::read_xkb_config() {
            Some(cfg) => {
                let layouts = xkb::available_layouts(&cfg);
                let active = xkb::active_layout(&cfg).unwrap_or_else(|| "us".to_string());
                tracing::info!("XKB layouts: {:?}, active: {}", layouts, active);
                active
            }
            None => {
                tracing::warn!("Could not read XKB config, using defaults");
                "us".to_string()
            }
        };

        let applet = KeyboardContextApplet {
            core,
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
                let popup_settings = self.core.applet.get_popup_settings(
                    self.core.main_window_id().unwrap(),
                    new_id,
                    None,
                    None,
                    None,
                );
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

                // Restore stored layout for the incoming window
                if let Some(desired) = self.layout_map.get(&identifier).cloned() {
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
                if let Some(active) = xkb::read_xkb_config()
                    .and_then(|cfg| xkb::active_layout(&cfg))
                {
                    if active != self.current_layout {
                        self.current_layout = active.clone();
                        // Save for current window
                        if let Some(ref app) = self.current_app {
                            self.layout_map.insert(app.clone(), active.clone());
                            tracing::debug!("Layout → '{}' for '{}'", active, app);
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
            let display_name = self
                .app_names
                .get(identifier)
                .map(|s| s.as_str())
                .unwrap_or("?");
            let item = cosmic::iced_widget::row![
                cosmic::widget::text(display_name).width(cosmic::iced::Length::Fill),
                cosmic::widget::text(layout.to_uppercase()),
            ]
            .padding([8, 12])
            .spacing(8);

            app_items.push(item.into());
        }

        let content = if app_items.is_empty() {
            cosmic::iced_widget::column![cosmic::widget::text(crate::fl!(
                "message-no-apps"
            )),]
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
