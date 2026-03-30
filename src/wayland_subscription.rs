use crate::app::Message;
use crate::wayland_handler::{spawn_wayland_handler, WaylandUpdate};
use cosmic::iced::Subscription;
use tokio::sync::mpsc;

pub fn wayland_subscription() -> Subscription<Message> {
    Subscription::run(wayland_stream)
}

fn wayland_stream() -> impl futures::Stream<Item = Message> {
    let (tx, rx) = mpsc::unbounded_channel::<WaylandUpdate>();

    // Spawn the wayland handler in a background thread
    spawn_wayland_handler(tx);

    futures::stream::unfold(rx, |mut rx: mpsc::UnboundedReceiver<WaylandUpdate>| async move {
        match rx.recv().await {
            Some(WaylandUpdate::Focused { app_id, title: _ }) => {
                let message = Message::ToplevelFocused(app_id.clone());
                Some((message, rx))
            }
            None => None,
        }
    })
}
