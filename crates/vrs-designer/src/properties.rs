//! Panneau des propriétés.

use egui::{DragValue, RichText};
use vrs_components::{EventKind, Feature, NameError};
use vrs_i18n::I18n;

use crate::state::{DesignerOutput, DesignerState, HandlerRequest, now};

fn row(ui: &mut egui::Ui, label: String, add: impl FnOnce(&mut egui::Ui)) {
    ui.label(label);
    add(ui);
    ui.end_row();
}

/// Images du dossier `resources/` du projet (chemins relatifs au projet).
fn resource_images(root: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.join("resources"))
        .map(|d| {
            d.filter_map(Result::ok)
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let ext = name.rsplit('.').next()?.to_lowercase();
                    ["png", "jpg", "jpeg", "gif", "bmp"]
                        .contains(&ext.as_str())
                        .then(|| format!("resources/{name}"))
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// Couleur facultative : case « personnalisée » + sélecteur.
fn color_row(ui: &mut egui::Ui, value: &mut Option<[u8; 4]>, i18n: &I18n) {
    let mut custom = value.is_some();
    if ui.checkbox(&mut custom, "").changed() {
        *value = custom.then_some([0, 120, 215, 255]);
    }
    if let Some(rgba) = value {
        let mut c = egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);
        if egui::color_picker::color_edit_button_srgba(
            ui,
            &mut c,
            egui::color_picker::Alpha::Opaque,
        )
        .changed()
        {
            *rgba = c.to_srgba_unmultiplied();
        }
    } else {
        ui.weak(i18n.tr("prop-default"));
    }
}

fn num(ui: &mut egui::Ui, v: &mut f32, min: f32) {
    ui.add(
        DragValue::new(v)
            .range(min..=10_000.0)
            .speed(1.0)
            .max_decimals(1),
    );
}

fn events_rows(
    ui: &mut egui::Ui,
    i18n: &I18n,
    events: &std::collections::BTreeMap<EventKind, String>,
    available: &[EventKind],
    control: Option<&str>,
    out: &mut DesignerOutput,
) {
    for ev in available {
        ui.label(i18n.tr(ev.i18n_key()));
        ui.horizontal(|ui| match events.get(ev) {
            Some(h) => {
                ui.monospace(h);
                if ui
                    .small_button("⏵")
                    .on_hover_text(i18n.tr("events-go-to"))
                    .clicked()
                {
                    out.open_handler = Some(HandlerRequest {
                        control: control.map(str::to_owned),
                        event: *ev,
                    });
                }
            }
            None => {
                ui.weak(i18n.tr("events-none"));
                if ui
                    .small_button("+")
                    .on_hover_text(i18n.tr("events-create"))
                    .clicked()
                {
                    out.open_handler = Some(HandlerRequest {
                        control: control.map(str::to_owned),
                        event: *ev,
                    });
                }
            }
        });
        ui.end_row();
    }
}

/// Affiche les propriétés de la sélection (ou de la fenêtre si rien n'est sélectionné).
pub fn properties_ui(ui: &mut egui::Ui, state: &mut DesignerState, i18n: &I18n) -> DesignerOutput {
    let mut out = DesignerOutput::default();
    let before = state.doc.clone();

    egui::ScrollArea::vertical()
        .id_salt("vrs-properties")
        .auto_shrink([false, true])
        .show(ui, |ui| match state.selection.len() {
            0 => {
                form_properties(ui, state, i18n, &mut out);
                menu_editor(ui, state, i18n, &mut out, None);
            }
            1 => {
                let name = state.selection[0].clone();
                control_properties(ui, state, &name, i18n, &mut out);
                if state.doc.control(&name).is_some() {
                    menu_editor(ui, state, i18n, &mut out, Some(name));
                }
            }
            n => {
                ui.label(i18n.trf("designer-multi-selection", &[("count", n.into())]));
            }
        });

    if state.doc != before {
        out.changed = true;
        // Regroupe les frappes successives dans le même champ.
        let key = ui
            .ctx()
            .memory(|m| m.focused())
            .or_else(|| ui.ctx().dragged_id())
            .map(|id| format!("prop-{id:?}"));
        state.history.record(before, key, now());
    }
    out
}

fn form_properties(
    ui: &mut egui::Ui,
    state: &mut DesignerState,
    i18n: &I18n,
    out: &mut DesignerOutput,
) {
    ui.weak(i18n.tr("designer-no-selection"));
    ui.add_space(4.0);
    let doc = &mut state.doc;
    egui::Grid::new("vrs-form-props")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            row(ui, i18n.tr("prop-type"), |ui| {
                ui.label(RichText::new(i18n.tr("designer-form")).italics());
            });
            row(ui, i18n.tr("prop-name"), |ui| {
                ui.monospace(&doc.name);
            });
            row(ui, i18n.tr("prop-title"), |ui| {
                ui.text_edit_singleline(&mut doc.title);
            });
            row(ui, i18n.tr("prop-width"), |ui| {
                num(ui, &mut doc.width, 120.0)
            });
            row(ui, i18n.tr("prop-height"), |ui| {
                num(ui, &mut doc.height, 80.0)
            });
        });
    ui.add_space(6.0);
    ui.strong(i18n.tr("prop-group-events"));
    let events = doc.events.clone();
    egui::Grid::new("vrs-form-events")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            events_rows(ui, i18n, &events, &EventKind::FORM_EVENTS, None, out);
        });
}

fn menu_editor(
    ui: &mut egui::Ui,
    state: &mut DesignerState,
    i18n: &I18n,
    out: &mut DesignerOutput,
    target: Option<String>,
) {
    ui.add_space(6.0);
    ui.strong(i18n.tr(if target.is_some() {
        "prop-context-menu"
    } else {
        "prop-menu"
    }));
    let current = match &target {
        Some(name) => state
            .doc
            .control(name)
            .map(|c| c.context_menu.clone())
            .unwrap_or_default(),
        None => state.doc.menu.clone(),
    };
    let mut text = match &state.menu_text {
        Some((t, buf)) if *t == target => buf.clone(),
        _ => vrs_components::MenuItem::to_outline(&current),
    };
    let r = ui
        .add(
            egui::TextEdit::multiline(&mut text)
                .id_salt(("vrs-menu-edit", &target))
                .code_editor()
                .desired_rows(4)
                .hint_text(i18n.tr("prop-menu-placeholder")),
        )
        .on_hover_text(i18n.tr("prop-menu-hint"));
    if r.changed() {
        let taken: Vec<String> = state
            .doc
            .used_names()
            .into_iter()
            .filter(|n| {
                let mut own = Vec::new();
                vrs_components::MenuItem::walk(&current, &mut own);
                !own.iter().any(|m| &m.name == n)
            })
            .collect();
        let parsed = vrs_components::MenuItem::from_outline(&text, &current, &taken);
        match &target {
            Some(name) => {
                if let Some(c) = state.doc.control_mut(name) {
                    c.context_menu = parsed;
                }
            }
            None => state.doc.menu = parsed,
        }
    }
    if r.has_focus() {
        state.menu_text = Some((target.clone(), text));
    } else if state.menu_text.as_ref().is_some_and(|(t, _)| *t == target) {
        state.menu_text = None;
    }
    let menu = match &target {
        Some(name) => state
            .doc
            .control(name)
            .map(|c| c.context_menu.clone())
            .unwrap_or_default(),
        None => state.doc.menu.clone(),
    };
    let mut items = Vec::new();
    vrs_components::MenuItem::walk(&menu, &mut items);
    let leaves: Vec<(String, String, Option<String>)> = items
        .iter()
        .filter(|m| m.children.is_empty())
        .map(|m| (m.name.clone(), m.text.clone(), m.click.clone()))
        .collect();
    if leaves.is_empty() {
        return;
    }
    egui::Grid::new("vrs-menu-events")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            for (name, text, click) in leaves {
                ui.label(text);
                ui.horizontal(|ui| {
                    match &click {
                        Some(h) => {
                            ui.monospace(h);
                        }
                        None => {
                            ui.weak(i18n.tr("events-none"));
                        }
                    }
                    let (icon, tip) = if click.is_some() {
                        ("⏵", "events-go-to")
                    } else {
                        ("+", "events-create")
                    };
                    if ui.small_button(icon).on_hover_text(i18n.tr(tip)).clicked() {
                        out.open_handler = Some(HandlerRequest {
                            control: Some(name.clone()),
                            event: EventKind::Click,
                        });
                    }
                });
                ui.end_row();
            }
        });
}

fn control_properties(
    ui: &mut egui::Ui,
    state: &mut DesignerState,
    name: &str,
    i18n: &I18n,
    out: &mut DesignerOutput,
) {
    let Some(kind) = state.doc.control(name).map(|c| c.kind) else {
        return;
    };

    // Nom : validé à la sortie du champ (le code généré en dépend).
    let (orig, buffer) = state
        .name_edit
        .get_or_insert_with(|| (name.to_owned(), name.to_owned()));
    if orig != name {
        *orig = name.to_owned();
        *buffer = name.to_owned();
    }
    let mut commit = None;

    egui::Grid::new("vrs-control-props")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label(RichText::new(i18n.tr("prop-group-design")).strong());
            ui.end_row();
            row(ui, i18n.tr("prop-name"), |ui| {
                let r = ui.text_edit_singleline(buffer);
                if r.lost_focus() && buffer != orig {
                    commit = Some(buffer.clone());
                }
            });
            row(ui, i18n.tr("prop-type"), |ui| {
                ui.label(RichText::new(kind.type_name()).italics());
            });
        });

    if let Some(new_name) = commit {
        match state.doc.check_name(&new_name, Some(name)) {
            Ok(()) => {
                let old = name.to_owned();
                let n2 = new_name.clone();
                state.edit(|d| {
                    let _ = d.rename_control(&old, &n2);
                });
                state.selection = vec![new_name.clone()];
                state.name_edit = Some((new_name.clone(), new_name));
                return;
            }
            Err(e) => {
                state.name_edit = Some((name.to_owned(), name.to_owned()));
                ui.ctx().data_mut(|d| {
                    d.insert_temp(
                        egui::Id::NULL.with("vrs-name-error"),
                        (e == NameError::Taken, now()),
                    )
                });
            }
        }
    }
    // Afficher l'erreur quelques secondes.
    let recent = ui
        .ctx()
        .data(|d| d.get_temp::<(bool, f64)>(egui::Id::NULL.with("vrs-name-error")));
    if let Some((taken, t)) = recent
        && now() - t < 6.0
    {
        let msg = if taken {
            "prop-name-taken"
        } else {
            "prop-invalid-name"
        };
        ui.colored_label(ui.visuals().error_fg_color, i18n.tr(msg));
    }

    let root = state.project_root.clone();
    let mut import = None;
    let Some(c) = state.doc.control_mut(name) else {
        return;
    };
    egui::Grid::new("vrs-control-props-2")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label(RichText::new(i18n.tr("prop-group-appearance")).strong());
            ui.end_row();
            row(ui, i18n.tr("prop-text"), |ui| {
                ui.text_edit_singleline(&mut c.text);
            });
            row(ui, i18n.tr("prop-tooltip"), |ui| {
                ui.text_edit_singleline(&mut c.tooltip);
            });
            row(ui, i18n.tr("prop-back-color"), |ui| {
                color_row(ui, &mut c.back_color, i18n)
            });
            row(ui, i18n.tr("prop-fore-color"), |ui| {
                color_row(ui, &mut c.fore_color, i18n)
            });
            row(ui, i18n.tr("prop-font-size"), |ui| {
                let mut custom = c.font_size.is_some();
                if ui.checkbox(&mut custom, "").changed() {
                    c.font_size = custom.then_some(14.0);
                }
                if let Some(s) = &mut c.font_size {
                    ui.add(DragValue::new(s).range(6.0..=96.0).speed(0.5));
                } else {
                    ui.weak(i18n.tr("prop-default"));
                }
            });
            ui.label(RichText::new(i18n.tr("prop-group-layout")).strong());
            ui.end_row();
            row(ui, i18n.tr("prop-x"), |ui| num(ui, &mut c.x, -1000.0));
            row(ui, i18n.tr("prop-y"), |ui| num(ui, &mut c.y, -1000.0));
            row(ui, i18n.tr("prop-width"), |ui| num(ui, &mut c.width, 8.0));
            row(ui, i18n.tr("prop-height"), |ui| num(ui, &mut c.height, 8.0));
            row(ui, i18n.tr("prop-anchor"), |ui| {
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut c.anchor.left, i18n.tr("anchor-left"));
                    ui.toggle_value(&mut c.anchor.top, i18n.tr("anchor-top"));
                    ui.toggle_value(&mut c.anchor.right, i18n.tr("anchor-right"));
                    ui.toggle_value(&mut c.anchor.bottom, i18n.tr("anchor-bottom"));
                })
                .response
                .on_hover_text(i18n.tr("prop-anchor-hint"));
            });
            ui.label(RichText::new(i18n.tr("prop-group-behavior")).strong());
            ui.end_row();
            row(ui, i18n.tr("prop-visible"), |ui| {
                ui.checkbox(&mut c.visible, "");
            });
            row(ui, i18n.tr("prop-enabled"), |ui| {
                ui.checkbox(&mut c.enabled, "");
            });
            if c.kind.has(Feature::Checked) {
                row(ui, i18n.tr("prop-checked"), |ui| {
                    ui.checkbox(&mut c.checked, "");
                });
            }
            if c.kind.has(Feature::Group) {
                row(ui, i18n.tr("prop-group"), |ui| {
                    ui.text_edit_singleline(&mut c.group)
                        .on_hover_text(i18n.tr("prop-group-hint"));
                });
            }
            if c.kind.has(Feature::Items) {
                ui.label(RichText::new(i18n.tr("prop-group-data")).strong());
                ui.end_row();
                let tabs = c.kind == vrs_components::ComponentKind::TabControl;
                row(
                    ui,
                    i18n.tr(if tabs { "prop-pages" } else { "prop-items" }),
                    |ui| {
                        let mut text = c.items.join("\n");
                        let r = ui
                            .add(egui::TextEdit::multiline(&mut text).desired_rows(4))
                            .on_hover_text(i18n.tr("prop-items-hint"));
                        if r.changed() {
                            c.items = text.lines().map(str::to_owned).collect();
                            if text.ends_with('\n') {
                                c.items.push(String::new());
                            }
                        }
                    },
                );
                row(
                    ui,
                    i18n.tr(if tabs {
                        "prop-selected-page"
                    } else {
                        "prop-selected"
                    }),
                    |ui| {
                        let none = i18n.tr("events-none");
                        let shown = c
                            .selected
                            .and_then(|i| c.items.get(i))
                            .cloned()
                            .unwrap_or_else(|| none.clone());
                        egui::ComboBox::from_id_salt("vrs-prop-selected")
                            .selected_text(shown)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut c.selected, None, none);
                                for (i, item) in c.items.iter().enumerate() {
                                    ui.selectable_value(&mut c.selected, Some(i), item.as_str());
                                }
                            });
                    },
                );
            }
            if c.kind.has(Feature::Image) {
                row(ui, i18n.tr("prop-image"), |ui| {
                    let none = i18n.tr("events-none");
                    let files = resource_images(&root);
                    let shown = if c.image.is_empty() {
                        none.clone()
                    } else {
                        c.image.clone()
                    };
                    egui::ComboBox::from_id_salt("vrs-prop-image")
                        .selected_text(shown)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut c.image, String::new(), none);
                            for f in files {
                                ui.selectable_value(&mut c.image, f.clone(), f);
                            }
                        });
                    if ui
                        .small_button("…")
                        .on_hover_text(i18n.tr("prop-image-import"))
                        .clicked()
                    {
                        import = Some(c.name.clone());
                    }
                });
            }
            if c.kind.has(Feature::Value) {
                ui.label(RichText::new(i18n.tr("prop-group-data")).strong());
                ui.end_row();
                row(ui, i18n.tr("prop-minimum"), |ui| {
                    ui.add(DragValue::new(&mut c.minimum).speed(1.0));
                });
                row(ui, i18n.tr("prop-maximum"), |ui| {
                    ui.add(DragValue::new(&mut c.maximum).speed(1.0));
                });
                let (min, max) = (c.minimum.min(c.maximum), c.maximum.max(c.minimum));
                row(ui, i18n.tr("prop-value"), |ui| {
                    ui.add(DragValue::new(&mut c.value).range(min..=max).speed(1.0));
                });
            }
        });
    if import.is_some() {
        out.import_image = import;
    }
    ui.add_space(6.0);
    ui.strong(i18n.tr("prop-group-events"));
    let events = c.events.clone();
    egui::Grid::new("vrs-control-events")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            events_rows(ui, i18n, &events, kind.events(), Some(name), out);
        });
}
