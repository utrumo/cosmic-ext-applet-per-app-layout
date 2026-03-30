use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum WaylandUpdate {
    Focused { app_id: String, title: String },
}

pub fn spawn_wayland_handler(
    _tx: mpsc::UnboundedSender<WaylandUpdate>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        // TODO: Implement Wayland event loop
        // This is a stub that prevents blocking startup
        tracing::info!("Wayland handler starting (stub)");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    })
}
