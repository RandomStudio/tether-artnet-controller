use std::fs;

use log::{debug, error, info, warn};
use scene::Scene;
use serde::{Deserialize, Serialize};

use crate::project::fixture::{FixtureConfig, FixtureMacro};

use self::artnetconfig::ArtNetConfigMode;
use self::fixture::FixtureInstance;
use self::midiconfig::MidiConfig;

pub mod artnetconfig;
pub mod fixture;
pub mod midiconfig;
pub mod scene;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub fixtures: Vec<FixtureInstance>,
    pub scenes: Vec<Scene>,
    #[serde(default)]
    pub midi_config: MidiConfig,
    pub artnet_config: Option<ArtNetConfigMode>,
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
        match fs::read_to_string(path) {
            Ok(d) => {
                info!("Found project {}; parsing...", &path);
                let mut project =
                    serde_json::from_str::<Project>(&d).expect("failed to parse project file");
                info!(
                    "... loaded project with {} fixtures OK",
                    project.fixtures.len()
                );

                // let all_fixtures_json = include_str!("../all_fixtures.json");
                // let all_fixture_configs =
                //     serde_json::from_str::<Vec<FixtureConfig>>(all_fixtures_json)
                //         .expect("failed to parse all_fixtures JSON");

                // debug!(
                //     "Loaded {} fixtures from all_fixtures JSON",
                //     all_fixture_configs.len(),
                // );

                let all_fixture_configs = load_all_fixture_configs();

                let mut global_index = 0;

                for fixture_ref in project.fixtures.iter_mut() {
                    if let Some(fixture_config) = all_fixture_configs
                        .iter()
                        .find(|x| x.name.eq_ignore_ascii_case(&fixture_ref.config_name))
                    {
                        fixture_ref.config = fixture_config.clone();
                        fixture_ref.config.active_mode =
                            fixture_ref.config.modes[fixture_ref.mode_index].clone();

                        for m in fixture_ref.config.active_mode.macros.iter_mut() {
                            match m {
                                FixtureMacro::Control(control_macro) => {
                                    control_macro.global_index = global_index;
                                    global_index += 1;
                                }
                                FixtureMacro::Colour(_colour_macro) => {
                                    // Ignore colour macros for now
                                }
                            }
                        }
                    } else {
                        error!(
                            "Failed to match config name \"{}\" with any known fixtures",
                            &fixture_ref.config_name
                        );
                    }
                }

                project.fixtures.sort_by_key(|x| x.label.clone());

                // Sort macros in Fixtures, alphabetically...
                for fixture in project.fixtures.iter_mut() {
                    let mut mode_macros_ordered = fixture.config.active_mode.clone();
                    mode_macros_ordered.macros.sort_by_key(|m| match m {
                        fixture::FixtureMacro::Control(cm) => cm.label.clone(),
                        fixture::FixtureMacro::Colour(cm) => cm.label.clone(),
                    });
                    fixture.config.active_mode = mode_macros_ordered;
                }

                // Level 1: Scenes sorted by their labels
                project.scenes.sort_by_key(|x| x.label.clone());

                // Level 2: Fixture for each Scene sorted by label
                for scene in project.scenes.iter_mut() {
                    scene.state.sort_keys();
                    for (_fixture_key, macro_key) in scene.state.iter_mut() {
                        macro_key.sort_keys();
                    }
                }

                debug!("Final ordered scenes: {:?}", project.scenes);

                // Level 3: Sort each Macro entry within each Scene Fixture entry
                // for scene in project.scenes.iter_mut() {
                //     for fixture in scene.state.iter_mut() {}
                // }
                // for (_fixture_instance_key, macro_contents) in scene.state.iter_mut() {
                //     let mut ordered_macros_vec = Vec::new();
                //     // let ordered_
                //     for (macro_key, macro_value) in macro_contents.clone() {
                //         ordered_macros_vec.push((macro_key, macro_value));
                //     }
                //     ordered_macros_vec.sort_by_key(|(k, _v)| String::from(k));

                //     macro_contents.clear();
                //     for (i, (k, v)) in ordered_macros_vec.iter().enumerate() {
                //         debug!("#{} Insert macro {}", i, k);
                //         macro_contents.insert((*k).clone(), (*v).clone());
                //     }
                //     debug!("Fixtures")
                // }

                Ok(project)
            }
            Err(e) => {
                warn!("Failed to load Project from disk: {:?}", e);
                Err(e.into())
            }
        }
    }

    pub fn save(path: &str, project: &Project) -> anyhow::Result<()> {
        let mut new_project = project.clone();
        new_project.fixtures.sort();

        let json = serde_json::to_string_pretty(&new_project)?;
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

/// Get the statically-defined DMX fixture configurations known to the system. This
/// list is built at compile-time using the JSON definitions found in the `fixtures` folder;
/// these are automatically concatenated into the file `all_fixtures.json` by the
/// build script.
pub fn load_all_fixture_configs() -> Vec<FixtureConfig> {
    let all_fixtures_json = include_str!("../all_fixtures.json");
    let all_fixture_configs = serde_json::from_str::<Vec<FixtureConfig>>(all_fixtures_json)
        .expect("failed to parse all_fixtures JSON");

    debug!(
        "Loaded {} fixtures from all_fixtures JSON",
        all_fixture_configs.len(),
    );
    all_fixture_configs
}
