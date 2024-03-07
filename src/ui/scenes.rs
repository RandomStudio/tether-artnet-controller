use indexmap::IndexMap;
use std::time::SystemTime;

use egui::{Grid, RichText, ScrollArea, Slider, Spinner, Ui};
use log::debug;

use crate::{
    model::Model,
    project::{fixture::FixtureMacro, Scene, SceneState, SceneValue},
};

pub fn render_scenes(model: &mut Model, ui: &mut Ui) {
    ui.heading("Scenes");

    ui.separator();

    let mut go_scene: Option<(usize, Option<u64>)> = None;
    let mut edit_scene: Option<usize> = None;
    let mut update_scene: Option<usize> = None;
    let mut delete_scene: Option<usize> = None;
    let mut add_scene: Option<Scene> = None;

    ScrollArea::new([false, true]).show(ui, |ui| {
        if ui.button("+ Add New").clicked() {
            let label = format!("New Scene {}", model.project.scenes.len());

            let mut state = IndexMap::<String, SceneState>::new();

            for fixture in model.project.fixtures.iter() {
                let mut m_state: IndexMap<String, SceneValue> = IndexMap::new();
                for m in fixture.config.active_mode.macros.iter() {
                    match m {
                        FixtureMacro::Control(control_macro) => {
                            m_state.insert(String::from(&control_macro.label), SceneValue::ControlValue(control_macro.current_value));
                        },
                        FixtureMacro::Colour(colour_macro) => {
                            m_state.insert(String::from(&colour_macro.label), SceneValue::ColourValue(colour_macro.current_value));
                        },
                    };
                }
                state.insert(String::from(&fixture.label), m_state);
            }

            add_scene = Some(Scene {
                label,
                state,
                is_editing: true,
                last_active: None
            });
        }

        ui.separator();

        for (scene_index, scene) in model.project.scenes.iter_mut().enumerate() {
            ui.group(|ui| {
                if scene.is_editing {
                    ui.text_edit_singleline(&mut scene.label);
                } else {
                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new(&scene.label).size(24.0))
                            .clicked()
                        {
                            go_scene = Some((scene_index, None)); // go to scene "immediately"
                        };
                        if let Some(t) = scene.last_active {
                            let progress = t.elapsed().unwrap().as_secs_f32() / 1.0;
                            if progress >= 1.0 { scene.last_active = None; }
                            ui.add(Spinner::new());
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Transition:");
                    if ui.button("1s").clicked() {
                        go_scene = Some((scene_index, Some(1000)));
                    }
                    if ui.button("3s").clicked() {
                        go_scene = Some((scene_index, Some(3000)));
                    }
                    if ui.button("10s").clicked() {
                        go_scene = Some((scene_index, Some(10000)));
                    }
                    });
                }

                if scene.is_editing {
                    for (fixture_index, s) in scene.state.iter_mut().enumerate() {
                        let (fixture_label, states) = s;
                        // ----------------
                        ui.heading(fixture_label);
                        // ----------------
                        ui.add_enabled_ui(true, |ui| {
                            Grid::new(format!("scene-{}-state-{}", scene_index, fixture_index))
                                .num_columns(2)
                                .show(ui, |ui| {
                                    for m in states.iter_mut() {
                                        let (macro_label, _scene_value) = m;
                                        // ----------------
                                        ui.label(macro_label);
                                        // ----------------
                                        // ui.add(Slider::new(scene_value, 0..=255));
                                        if let Some(matched_fixture) = model
                                            .project
                                            .fixtures
                                            .iter_mut()
                                            .find(|x| x.label.eq(fixture_label))
                                        {
                                            if let Some(matched_macro) = matched_fixture
                                                .config
                                                .active_mode
                                                .macros
                                                .iter_mut()
                                                .find(|x|  match x {
                                                    FixtureMacro::Control(m) => m.label.eq(macro_label),
                                                    FixtureMacro::Colour(m) => m.label.eq(macro_label),
                                                }

                                            )
                                            {
                                                match matched_macro {
                                                    FixtureMacro::Control(m) => {
                                                        // let mut dummy_value = m.current_value;
                                                        ui.add(Slider::new(&mut m.current_value, 0..=255));

                                                    } ,
                                                    FixtureMacro::Colour(m) => {
                                                        // let mut dummy_value = m.current_value;
                                                        ui.color_edit_button_srgba(&mut m.current_value);
                                                    }
                                                };
                                                ui.small("Adjust values in Macros panel");
                                            }
                                        } else {
                                            ui.label("Something went wrong matching fixture macros to scene macros!");
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                    }
                    if ui.button("Save ✅").clicked() {
                        update_scene = Some(scene_index);
                        edit_scene = None;
                    }
                } else {
                    ui.horizontal(|ui| {
                        if ui.button("✏").clicked() {
                            // Mark scene for editing
                            edit_scene = Some(scene_index);
                            // Also go to this scene (immediately)
                            go_scene = Some((scene_index, None));
                        }
                        if ui.button("🗑").clicked() {
                            delete_scene = Some(scene_index);
                        }
                    });
                }
                ui.separator();
            });
        }
    });

    if let Some(scene_index) = edit_scene {
        // First, mark any CURRENTLY-edited scene for update (save)
        for (index, scene) in model.project.scenes.iter_mut().enumerate() {
            if scene.is_editing {
                debug!("Scene {} should get saved", index);
                update_scene = Some(index);
            }
        }

        // Then mark is_editing exclusively to the target Scene
        for (index, scene) in model.project.scenes.iter_mut().enumerate() {
            scene.is_editing = index == scene_index;
        }
    }

    if let Some(scene_index) = update_scene {
        let scene = &mut model.project.scenes[scene_index];
        scene.is_editing = false;

        for fixture in model.project.fixtures.iter() {
            let mut m_state = IndexMap::new();
            for m in fixture.config.active_mode.macros.iter() {
                match m {
                    FixtureMacro::Control(control_macro) => {
                        m_state.insert(
                            String::from(&control_macro.label),
                            SceneValue::ControlValue(control_macro.current_value),
                        );
                    }
                    FixtureMacro::Colour(colour_macro) => {
                        m_state.insert(
                            String::from(&colour_macro.label),
                            SceneValue::ColourValue(colour_macro.current_value),
                        );
                    }
                }
            }
            scene.state.insert(String::from(&fixture.label), m_state);
        }
    }

    if let Some((scene_index, ms)) = go_scene {
        model.apply_scene(scene_index, ms, None);

        for (index, scene) in model.project.scenes.iter_mut().enumerate() {
            if index == scene_index {
                // This one
                scene.last_active = Some(SystemTime::now());
            } else {
                // Others
                scene.is_editing = false;
            }
        }
    }

    if let Some(scene_index) = delete_scene {
        model.project.scenes.remove(scene_index);
    }

    if let Some(scene) = add_scene {
        model.project.scenes.push(scene);
    }
}
