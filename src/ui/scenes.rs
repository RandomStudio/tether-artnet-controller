use std::collections::HashMap;

use egui::{Grid, RichText, ScrollArea, Slider, Ui};

use crate::{
    model::Model,
    project::{Scene, SceneState},
};

pub fn render_scenes(model: &mut Model, ui: &mut Ui) {
    ui.heading("Scenes");

    ui.separator();

    let mut go_scene: Option<usize> = None;
    let mut update_scene: Option<usize> = None;
    let mut delete_scene: Option<usize> = None;
    let mut add_scene: Option<Scene> = None;

    ScrollArea::new([false, true]).show(ui, |ui| {
        if ui.button("+ Add New").clicked() {
            let label = format!("New Scene {}", model.project.scenes.len());

            let mut state = HashMap::<String, SceneState>::new();

            for fixture in model.project.fixtures.iter() {
                let mut m_state = HashMap::new();
                for m in fixture.config.active_mode.macros.iter() {
                    m_state.insert(String::from(&m.label), m.current_value);
                }
                state.insert(String::from(&fixture.label), m_state);
            }

            add_scene = Some(Scene {
                label,
                state,
                is_editing: false,
            });
        }

        ui.separator();

        for (scene_index, scene) in model.project.scenes.iter_mut().enumerate() {
            ui.group(|ui| {
                if scene.is_editing {
                    ui.text_edit_singleline(&mut scene.label);
                } else {
                    if ui
                        .button(RichText::new(&scene.label).size(24.0))
                        .on_hover_text("Click to GO")
                        .clicked()
                    {
                        go_scene = Some(scene_index);
                    };
                }

                if scene.is_editing {
                    for (fixture_index, s) in scene.state.iter_mut().enumerate() {
                        let (fixture_label, states) = s;
                        ui.label(fixture_label);
                        ui.add_enabled_ui(false, |ui| {
                            Grid::new(format!("scene-{}-state-{}", scene_index, fixture_index))
                                .num_columns(2)
                                .show(ui, |ui| {
                                    for m in states.iter_mut() {
                                        let (macro_label, scene_value) = m;
                                        ui.label(macro_label);
                                        // ui.add(Slider::new(scene_value, 0..=255));
                                        if let Some(matched_fixture) = model
                                            .project
                                            .fixtures
                                            .iter()
                                            .find(|x| x.label.eq(fixture_label))
                                        {
                                            if let Some(matched_macro) = matched_fixture
                                                .config
                                                .active_mode
                                                .macros
                                                .iter()
                                                .find(|x| x.label.eq(macro_label))
                                            {
                                                let mut value = matched_macro.current_value;
                                                ui.add(Slider::new(&mut value, 0..=255));
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
                    if ui.button("Update ✅").clicked() {
                        update_scene = Some(scene_index);
                        scene.is_editing = false;
                    }
                } else {
                    ui.horizontal(|ui| {
                        if ui.button("✏").clicked() {
                            scene.is_editing = true;
                            go_scene = Some(scene_index);
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

    if let Some(scene_index) = go_scene {
        model.apply_scene(scene_index, None);
        for (index, scene) in model.project.scenes.iter_mut().enumerate() {
            if index != scene_index {
                scene.is_editing = false;
            }
        }
    }

    if let Some(scene_index) = update_scene {
        let scene = &mut model.project.scenes[scene_index];
        // scene.state.clear();

        for fixture in model.project.fixtures.iter() {
            let mut m_state = HashMap::new();
            for m in fixture.config.active_mode.macros.iter() {
                m_state.insert(String::from(&m.label), m.current_value);
            }
            scene.state.insert(String::from(&fixture.label), m_state);
        }
    }

    if let Some(scene_index) = delete_scene {
        model.project.scenes.remove(scene_index);
    }

    if let Some(scene) = add_scene {
        model.project.scenes.push(scene);
    }
}
