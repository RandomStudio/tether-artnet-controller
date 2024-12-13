use egui::Color32;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SceneValue {
    ControlValue(u8),
    ColourValue(Color32),
}

/// { "macro label": value }
pub type SceneState = IndexMap<String, SceneValue>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Scene {
    pub label: String,
    /// { "fixture instance label": { "macro label": value } }
    pub state: IndexMap<String, SceneState>,
    #[serde(skip)]
    pub is_editing: bool,
    #[serde(skip)]
    pub last_active: bool,
    #[serde(skip)]
    pub next_transition: f32,
}
