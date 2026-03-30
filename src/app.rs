use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::Subscription;
use cosmic::{Application, Element};
use std::collections::HashMap;

const APP_ID: &str = "io.github.utrumo.CosmicKeyboardContext";

pub(crate) fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<KeyboardContextApplet>(())
}

#[derive(Default)]
struct KeyboardContextApplet {
    core: Core,
    popup: Option<Id>,
    layout_map: HashMap<String, String>,
    #[allow(dead_code)]
    current_app: Option<String>,
    current_layout: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleWindow,
    PopupClosed(Id),
    ToplevelFocused(String), // app_id
}

impl Application for KeyboardContextApplet {
    type Executor = cosmic::executor::Default;
    type Message = Message;
    type Flags = ();

    const APP_ID: &'static str = APP_ID;

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        let applet = KeyboardContextApplet {
            core,
            current_layout: "US".to_string(),
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
            Message::ToplevelFocused(app_id) => {
                // TODO: Save old layout and restore new layout
                if let Some(stored_layout) = self.layout_map.get(&app_id) {
                    self.current_layout = stored_layout.clone();
                    // TODO: Apply layout via cosmic-config
                } else {
                    self.current_layout = "US".to_string();
                }
                self.current_app = Some(app_id);
            }
        }
        Task::none()
    }

    fn view(&'_ self) -> Element<'_, Message> {
        let btn = cosmic::iced_widget::button(cosmic::widget::text(&self.current_layout))
            .on_press(Message::ToggleWindow);

        cosmic::widget::autosize::autosize(btn, cosmic::widget::Id::unique()).into()
    }

    fn view_window(&'_ self, _id: Id) -> Element<'_, Message> {
        let mut app_items = Vec::new();
        for (app_id, layout) in &self.layout_map {
            let item = cosmic::iced_widget::row![
                cosmic::widget::text(app_id.as_str()).width(cosmic::iced::Length::Fill),
                cosmic::widget::text(layout.as_str()),
            ]
            .padding([8, 12])
            .spacing(8);

            app_items.push(item.into());
        }

        let content = if app_items.is_empty() {
            cosmic::iced_widget::column![
                cosmic::widget::text("No applications remembered yet"),
            ]
            .padding([16, 12])
        } else {
            cosmic::iced_widget::column(app_items)
                .padding([4, 0])
        };

        self.core
            .applet
            .popup_container(cosmic::widget::container(content))
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        crate::wayland_subscription::wayland_subscription()
    }
}
