use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::animation::Animation;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixtureInstance {
    pub label: String,
    pub path: String,
    pub offset_channels: u16,
    #[serde(default)]
    pub mode_index: usize,
    #[serde(skip)]
    /// The actual configuration, once loaded via the path
    pub config: FixtureConfig,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FixtureConfig {
    pub name: String,
    pub reference: String,
    pub modes: Vec<ControlMode>,
    #[serde(skip)]
    /// The active mode actually in use, copied from the list on loading
    pub active_mode: ControlMode,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ControlMode {
    pub name: String,
    pub mappings: Vec<Mapping>,
    pub macros: Vec<FixtureMacro>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapping {
    pub channel: u16,
    pub label: String,
    pub notes: Option<String>,
    pub home: Option<u8>,
    pub ranges: Option<Vec<RangeDescription>>,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelMacro {
    pub label: String,
    pub channels: Vec<u16>,
    #[serde(skip)]
    pub current_value: u8,
    #[serde(skip)]
    pub animation: Option<Animation>,
}

// Cloning an Animation is tricky, and we don't need it anyway
impl Clone for ChannelMacro {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            channels: self.channels.clone(),
            current_value: self.current_value.clone(),
            animation: None, // Just ignore
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RGBWChannels {
    pub red: Vec<u16>,
    pub green: Vec<u16>,
    pub blue: Vec<u16>,
    pub white: Vec<u16>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CMYChannels {
    pub cyan: Vec<u16>,
    pub magenta: Vec<u16>,
    pub yellow: Vec<u16>,
    pub white: Vec<u16>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ChannelList {
    Additive(RGBWChannels),
    Subtractive(CMYChannels),
}

fn default_rgb() -> Color32 {
    Color32::LIGHT_YELLOW
}

#[derive(Serialize, Deserialize)]
pub struct ColourMacro {
    pub label: String,
    pub channels: ChannelList,
    #[serde(skip, default = "default_rgb")]
    pub current_value: Color32,
    #[serde(skip)]
    pub animation: Option<(Animation, Color32, Color32)>,
}

impl Clone for ColourMacro {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            channels: self.channels.clone(),
            current_value: self.current_value.clone(),
            animation: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FixtureMacro {
    Control(ChannelMacro),
    Colour(ColourMacro),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RangeDescription {
    pub range: [u8; 2],
    pub label: String,
}
