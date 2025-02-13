use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MidiConfig {
    /// Which controller number counts as the first, i.e. macro index 0
    pub controller_start: u8,
    /// Which note count as the first, i.e. fixture index 0
    pub note_start: u8,
}

impl Default for MidiConfig {
    fn default() -> Self {
        MidiConfig {
            controller_start: 48,
            note_start: 49,
        }
    }
}
