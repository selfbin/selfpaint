//! Application preferences persisted with `craft-settings`.

use craft_settings::{DocumentMut, MigrateError, Settings};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(default)]
pub struct Prefs {
    pub general: General,
    pub tools: ToolsPrefs,
    pub view: ViewPrefs,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct General {
    pub language: String,
}

impl Default for General {
    fn default() -> Self {
        Self {
            language: "pt".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct ToolsPrefs {
    pub tool: String,
    pub stroke: u8,
    pub color1: [u8; 4],
    pub color2: [u8; 4],
    pub active_color: u8,
    pub filled_shapes: bool,
    pub last_dir: String,
    pub text_size: u16,
    pub text_font: String,
    pub text_bold: bool,
    pub text_italic: bool,
}

impl Default for ToolsPrefs {
    fn default() -> Self {
        Self {
            tool: "brush".into(),
            stroke: 8,
            color1: [0, 0, 0, 255],
            color2: [255, 255, 255, 255],
            active_color: 1,
            filled_shapes: false,
            last_dir: String::new(),
            text_size: 32,
            text_font: "Open Sans".into(),
            text_bold: false,
            text_italic: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct ViewPrefs {
    pub zoom: f32,
}

impl Default for ViewPrefs {
    fn default() -> Self {
        Self { zoom: 1.0 }
    }
}

impl Settings for Prefs {
    const APP_ID: &'static str = "selfpaint";
    const SCHEMA_VERSION: u32 = 1;

    fn migrate(_from: u32, _doc: &mut DocumentMut) -> Result<(), MigrateError> {
        Ok(())
    }
}
