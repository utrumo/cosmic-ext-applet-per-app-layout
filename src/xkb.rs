//! XKB keyboard layout management via cosmic-config.
//!
//! Mirrors cosmic-comp's `XkbConfig` struct to read/write layout configuration.
//! The active layout is determined by position: the first entry in the comma-separated
//! `layout` string is active. Switching layouts means reordering the string.

use cosmic_config::{ConfigGet, ConfigSet};
use serde::{Deserialize, Serialize};

/// Config namespace and version for cosmic-comp. Must match libcosmic's pin.
const COSMIC_COMP_CONFIG: &str = "com.system76.CosmicComp";
const COSMIC_COMP_VERSION: u64 = 1;

/// Mirror of cosmic-comp-config's XkbConfig.
/// We define our own to avoid adding cosmic-comp-config as a dependency.
/// Fields must match upstream; unknown fields are silently ignored via serde defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XkbConfig {
    #[serde(default)]
    pub rules: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub layout: String,
    #[serde(default)]
    pub variant: String,
    #[serde(default)]
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
    let config = cosmic_config::Config::new(COSMIC_COMP_CONFIG, COSMIC_COMP_VERSION).ok()?;
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
    let config = match cosmic_config::Config::new(COSMIC_COMP_CONFIG, COSMIC_COMP_VERSION) {
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

/// Empty strings are kept (unlike `available_layouts`) because "" is a valid
/// "no variant" value in XKB, while "" is not a valid layout name.
fn parse_variants(xkb: &XkbConfig) -> Vec<String> {
    xkb.variant
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_xkb(layout: &str, variant: &str) -> XkbConfig {
        XkbConfig {
            layout: layout.to_string(),
            variant: variant.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_available_layouts_two() {
        let xkb = make_xkb("ru,us", ",");
        assert_eq!(available_layouts(&xkb), vec!["ru", "us"]);
    }

    #[test]
    fn test_available_layouts_single() {
        let xkb = make_xkb("us", "");
        assert_eq!(available_layouts(&xkb), vec!["us"]);
    }

    #[test]
    fn test_available_layouts_empty() {
        let xkb = make_xkb("", "");
        assert!(available_layouts(&xkb).is_empty());
    }

    #[test]
    fn test_active_layout() {
        let xkb = make_xkb("ru,us", ",");
        assert_eq!(active_layout(&xkb), Some("ru".to_string()));
    }

    #[test]
    fn test_active_layout_empty() {
        let xkb = make_xkb("", "");
        assert_eq!(active_layout(&xkb), None);
    }

    #[test]
    fn test_make_layout_active_swap() {
        let xkb = make_xkb("ru,us", ",");
        let result = make_layout_active(&xkb, "us").unwrap();
        assert_eq!(result.layout, "us,ru");
        assert_eq!(result.variant, ",");
    }

    #[test]
    fn test_make_layout_active_already_first() {
        let xkb = make_xkb("us,ru", ",");
        let result = make_layout_active(&xkb, "us").unwrap();
        assert_eq!(result.layout, "us,ru");
    }

    #[test]
    fn test_make_layout_active_not_found() {
        let xkb = make_xkb("us,ru", ",");
        assert!(make_layout_active(&xkb, "de").is_none());
    }

    #[test]
    fn test_make_layout_active_three_layouts() {
        let xkb = make_xkb("us,ru,de", ",,nodeadkeys");
        let result = make_layout_active(&xkb, "de").unwrap();
        assert_eq!(result.layout, "de,us,ru");
        assert_eq!(result.variant, "nodeadkeys,,");
    }

    #[test]
    fn test_make_layout_active_preserves_other_fields() {
        let mut xkb = make_xkb("us,ru", ",");
        xkb.repeat_delay = 300;
        xkb.repeat_rate = 50;
        let result = make_layout_active(&xkb, "ru").unwrap();
        assert_eq!(result.repeat_delay, 300);
        assert_eq!(result.repeat_rate, 50);
    }
}
