use cosmic_config::{ConfigGet, ConfigSet};
use serde::{Deserialize, Serialize};

/// Mirror of cosmic-comp-config's XkbConfig.
/// We define our own to avoid adding cosmic-comp-config as a dependency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XkbConfig {
    pub rules: String,
    pub model: String,
    pub layout: String,
    pub variant: String,
    pub options: Option<String>,
    #[serde(default = "default_repeat_delay")]
    pub repeat_delay: u32,
    #[serde(default = "default_repeat_rate")]
    pub repeat_rate: u32,
}

fn default_repeat_rate() -> u32 {
    25
}

fn default_repeat_delay() -> u32 {
    600
}

impl Default for XkbConfig {
    fn default() -> Self {
        Self {
            rules: String::new(),
            model: String::new(),
            layout: String::new(),
            variant: String::new(),
            options: None,
            repeat_delay: default_repeat_delay(),
            repeat_rate: default_repeat_rate(),
        }
    }
}

/// Read the current XKB config from cosmic-comp settings.
pub fn read_xkb_config() -> Option<XkbConfig> {
    let config = cosmic_config::Config::new("com.system76.CosmicComp", 1).ok()?;
    match config.get::<XkbConfig>("xkb_config") {
        Ok(xkb) => Some(xkb),
        Err(e) => {
            tracing::warn!("Failed to read xkb_config: {e}");
            None
        }
    }
}

/// Write the XKB config to cosmic-comp settings.
/// cosmic-comp watches via inotify and applies changes automatically.
pub fn write_xkb_config(xkb: &XkbConfig) -> bool {
    let config = match cosmic_config::Config::new("com.system76.CosmicComp", 1) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create cosmic config: {e}");
            return false;
        }
    };
    if let Err(e) = config.set("xkb_config", xkb) {
        tracing::error!("Failed to write xkb_config: {e}");
        return false;
    }
    true
}

/// Parse layout string "ru,us" into vec ["ru", "us"].
pub fn available_layouts(xkb: &XkbConfig) -> Vec<String> {
    xkb.layout
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// The active layout is the first one in the list.
pub fn active_layout(xkb: &XkbConfig) -> Option<String> {
    available_layouts(xkb).into_iter().next()
}

fn parse_variants(xkb: &XkbConfig) -> Vec<String> {
    xkb.variant.split(',').map(|s| s.trim().to_string()).collect()
}

/// Reorder layout and variant strings so that `target_layout` is first.
/// Returns None if target_layout is not in the available layouts.
pub fn make_layout_active(xkb: &XkbConfig, target_layout: &str) -> Option<XkbConfig> {
    let layouts = available_layouts(xkb);
    let variants = parse_variants(xkb);

    let idx = layouts.iter().position(|l| l == target_layout)?;

    let mut new_layouts = Vec::with_capacity(layouts.len());
    let mut new_variants = Vec::with_capacity(layouts.len());

    new_layouts.push(layouts[idx].clone());
    new_variants.push(variants.get(idx).cloned().unwrap_or_default());

    for (i, layout) in layouts.iter().enumerate() {
        if i != idx {
            new_layouts.push(layout.clone());
            new_variants.push(variants.get(i).cloned().unwrap_or_default());
        }
    }

    Some(XkbConfig {
        layout: new_layouts.join(","),
        variant: new_variants.join(","),
        ..xkb.clone()
    })
}
