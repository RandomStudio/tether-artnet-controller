use std::{collections::HashMap, fs};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::animation::Animation;

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub fixtures: Vec<FixtureInstance>,
    pub scenes: Vec<Scene>,
}

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
    pub macros: Vec<ControlMacro>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapping {
    pub channel: u16,
    pub label: String,
    pub notes: Option<String>,
    pub default: Option<u8>,
    pub ranges: Option<Vec<RangeDescription>>,
}

#[derive(Serialize, Deserialize)]
pub struct ControlMacro {
    pub label: String,
    pub channels: Vec<u16>,
    #[serde(skip)]
    pub current_value: u8,
    #[serde(skip)]
    pub animation: Option<Animation>,
}

// Cloning an Animation is tricky, and we don't need it anyway
impl Clone for ControlMacro {
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
pub struct RangeDescription {
    pub range: [u8; 2],
    pub label: String,
}

/// "Macro label": value
pub type SceneState = HashMap<String, u8>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Scene {
    pub label: String,
    /// "Fixture instance label": { "macro label": value } }
    pub state: HashMap<String, SceneState>,
    #[serde(skip)]
    pub is_editing: bool,
}

impl Project {
    pub fn new() -> Project {
        Project {
            fixtures: Vec::new(),
            scenes: Vec::new(),
        }
    }

    pub fn load(path: &str) -> anyhow::Result<Project> {
        let text = fs::read_to_string(path);
        match text {
            Ok(d) => {
                info!("Found project {}; parsing...", &path);
                let mut project =
                    serde_json::from_str::<Project>(&d).expect("failed to parse project file");
                info!(
                    "... loaded project with {} fictures OK",
                    project.fixtures.len()
                );

                // // Order macros in scenes
                // for scene in project.scenes.iter_mut() {
                //     for (fixture, state) in scene.state.iter_mut() {
                //         let mut macros: Vec<(&String, &u8)> = state.iter().collect();
                //         macros.sort();
                //         debug!("Sorted macros: {:?}", macros);
                //     }
                // }

                for fixture in project.fixtures.iter_mut() {
                    match fs::read_to_string(&fixture.path) {
                        Ok(d) => {
                            info!("Loaded fixture data from {}; parsing...", &fixture.path);
                            let parsed = serde_json::from_str::<FixtureConfig>(&d)?;
                            info!("... Parsed fixture OK");
                            fixture.config = parsed;
                            fixture.config.active_mode =
                                fixture.config.modes[fixture.mode_index].clone();
                        }
                        Err(e) => {
                            error!("Failed to load fixture: {}", e);
                        }
                    }
                }

                Ok(project)
            }
            Err(e) => {
                warn!("Failed to load widgets from disk: {:?}", e);
                Err(e.into())
            }
        }
    }

    pub fn save(path: &str, project: &Project) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(&project)?;
        debug!("{}", json);

        fs::write(&path, json)?;

        info!("Saved Project JSON to \"{}\" OK", &path);

        Ok(())
    }
}
