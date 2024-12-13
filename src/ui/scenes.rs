use indexmap::IndexMap;

use egui::{RichText, ScrollArea, Slider, Ui};
use log::debug;

use crate::{
    model::Model,
    project::{
        fixture::FixtureMacro,
        scene::{Scene, SceneState, SceneValue},
    },
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
                    };
                }
                state.insert(String::from(&fixture.label), m_state);
            }

            add_scene = Some(Scene {
                label,
                state,
                is_editing: true,
                last_active: false,
                next_transition: 0.,
            });
        }

        ui.separator();

        for (scene_index, scene) in model.project.scenes.iter_mut().enumerate() {
            ui.group(|ui| {
                if scene.is_editing {
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut scene.label);
                    });
                } else {
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new(&scene.label).size(24.0)).clicked() {
                            go_scene = Some((scene_index, None)); // go to scene "immediately"
                        };
                        if scene.last_active {
                            ui.label("★");
                        }
                        for (_fixture_instance_label, macros_used) in scene.state.iter() {
                            for (_macro_name, value) in macros_used.iter() {
                                match value {
                                    SceneValue::ControlValue(_) => {}
                                    SceneValue::ColourValue(c) => {
                                        // TODO: good idea, but maybe just used coloured icons instead of full colour pickers
                                        // which are disabled anyway!
                                        let mut copy_c = *c;
                                        ui.add_enabled_ui(false, |ui| {
                                            ui.color_edit_button_srgba(&mut copy_c);
                                        });
                                    }
                                }
                            }
                        }
                    });
                    ui.separator();

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
                        ui.horizontal(|ui| {
                            ui.label("Custom (s)");
                            ui.add(
                                Slider::new(&mut scene.next_transition, 0. ..=10.0).step_by(0.1),
                            );
                            if ui.button("Go").clicked() {
                                go_scene = Some((
                                    scene_index,
                                    Some((scene.next_transition * 1000.) as u64),
                                ));
                            }
                        });
                    });
                }

                if scene.is_editing {
                    ui.label("Edit values in Macro panel, then click Save.");
                    ui.horizontal(|ui| {
                        if ui.button("Save ✅").clicked() {
                            update_scene = Some(scene_index);
                            edit_scene = None;
                        }
                        if ui.button("Cancel ❌").clicked() {
                            edit_scene = None;
                            scene.is_editing = false;
                        }
                    });
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
                scene.last_active = true;
            } else {
                // Others
                scene.last_active = false;
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
