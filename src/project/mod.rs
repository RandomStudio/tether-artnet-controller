use std::{collections::HashMap, fs, time::SystemTime};

use egui::Color32;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::project::fixture::FixtureConfig;

use self::artnetconfig::ArtNetConfigMode;
use self::fixture::FixtureInstance;
use self::midiconfig::MidiConfig;

pub mod artnetconfig;
pub mod fixture;
pub mod midiconfig;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub fixtures: Vec<FixtureInstance>,
    pub scenes: Vec<Scene>,
    #[serde(default)]
    pub midi_config: MidiConfig,
    pub artnet_config: Option<ArtNetConfigMode>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SceneValue {
    ControlValue(u8),
    ColourValue(Color32),
}

/// "Macro label": value
pub type SceneState = HashMap<String, SceneValue>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Scene {
    pub label: String,
    /// "Fixture instance label": { "macro label": value } }
    pub state: HashMap<String, SceneState>,
    #[serde(skip)]
    pub is_editing: bool,
    #[serde(skip)]
    pub last_active: Option<SystemTime>,
}

impl Project {
    pub fn new() -> Project {
        Project {
            fixtures: Vec::new(),
            scenes: Vec::new(),
            midi_config: MidiConfig::default(),
            artnet_config: None,
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
                    "... loaded project with {} fixtures OK",
                    project.fixtures.len()
                );

                let all_fixtures_json = include_str!("../all_fixtures.json");
                let all_fixtures = serde_json::from_str::<Vec<FixtureConfig>>(all_fixtures_json)
                    .expect("failed to parse all_fixtures JSON");

                debug!(
                    "Loaded {} fixtures from all_fixtures JSON",
                    all_fixtures.len(),
                );

                for fixture_ref in project.fixtures.iter_mut() {
                    if let Some(fixture_config) = all_fixtures
                        .iter()
                        .find(|x| x.name.eq_ignore_ascii_case(&fixture_ref.config_name))
                    {
                        fixture_ref.config = fixture_config.clone();
                        fixture_ref.config.active_mode =
                            fixture_ref.config.modes[fixture_ref.mode_index].clone();
                    } else {
                        error!(
                            "Failed to match config name \"{}\" with any known fixtures",
                            &fixture_ref.config_name
                        );
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

        fs::write(path, json)?;

        info!("Saved Project JSON to \"{}\" OK", &path);

        Ok(())
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}
