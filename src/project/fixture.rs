use std::cmp::Ordering;

use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::animation::Animation;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// A Fixture, as configured in a Project file
pub struct FixtureInstance {
    /// The label (should be unique per project) of this fixture instance
    pub label: String,
    /// The exact match for the fixture name as it appears in the fixture config JSON
    pub config_name: String,
    pub offset_channels: u16,
    #[serde(default)]
    pub mode_index: usize,
    #[serde(skip)]
    /// The actual configuration, once loaded via the path
    pub config: FixtureConfig,
}

impl PartialEq for FixtureInstance {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}
impl PartialOrd for FixtureInstance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for FixtureInstance {}
impl Ord for FixtureInstance {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.label > other.label {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl From<&FixtureConfig> for FixtureInstance {
    fn from(config: &FixtureConfig) -> Self {
        FixtureInstance {
            label: format!("My {}", config.name),
            config_name: String::from(&config.name),
            offset_channels: 0,
            mode_index: 0,
            config: config.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
/// Stores the underlying configuration for a specific model of DMX fixture.
///
/// Unlike a FixtureInstance which could appear multiple times in a single Project,
/// there can only be one static FixtureConfig per type of fixture.
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
    #[serde(skip)]
    pub global_index: u8,
}

// Cloning an Animation is tricky, and we don't need it anyway
impl Clone for ChannelMacro {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            channels: self.channels.clone(),
            current_value: self.current_value,
            animation: None, // Just ignore
            global_index: self.global_index,
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
            current_value: self.current_value,
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
