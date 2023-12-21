use std::fs;

use log::{error, info, warn};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub fixtures: Vec<FixtureInstance>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FixtureConfig {
    pub name: String,
    pub reference: String,
    pub modes: Vec<ControlMode>,
    #[serde(skip)]
    pub active_mode: ControlMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ControlMode {
    pub name: String,
    pub mappings: Vec<Mapping>,
    pub macros: Vec<ControlMacro>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mapping {
    pub channel: u16,
    pub label: String,
    pub notes: Option<String>,
    pub default: Option<u8>,
    pub ranges: Option<Vec<RangeDescription>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ControlMacro {
    pub label: String,
    pub channels: Vec<u16>,
    #[serde(skip)]
    pub current_value: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RangeDescription {
    pub range: [u8; 2],
    pub label: String,
}

impl Project {
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

                for fixture in project.fixtures.iter_mut() {
                    match fs::read_to_string(&fixture.path) {
                        Ok(d) => {
                            info!("Loaded fixture data from {}; parsing...", &fixture.path);
                            let parsed = serde_json::from_str::<FixtureConfig>(&d)
                                .expect("failed to parse Fixture data file");
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
}
