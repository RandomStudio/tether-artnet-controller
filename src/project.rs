use std::fs;

use log::{error, info, warn};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub fixtures: Vec<FixtureConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixtureConfig {
    pub path: String,
    pub fixture: Option<Fixture>,
    pub offset_channels: u16,
    pub mode: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fixture {
    pub name: String,
    pub reference: String,
    pub modes: Vec<ControlMode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ControlMode {
    pub name: String,
    pub mappings: Vec<Mapping>,
    pub groups: Vec<Group>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mapping {
    pub channel: u16,
    pub label: String,
    pub default: Option<u8>,
    pub ranges: Option<Vec<RangeDescription>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Group {
    pub label: String,
    pub channels: Vec<u16>,
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
                            let parsed = serde_json::from_str::<Fixture>(&d)
                                .expect("failed to parse Fixture data file");
                            fixture.fixture = Some(parsed);
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
