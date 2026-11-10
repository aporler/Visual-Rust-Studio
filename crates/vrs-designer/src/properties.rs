//! Panneau des propriétés.

use egui::{DragValue, RichText};
use vrs_components::{EventKind, NameError};
use vrs_i18n::I18n;

use crate::state::{DesignerOutput, DesignerState, HandlerRequest, now};

fn row(ui: &mut egui::Ui, label: String, add: impl FnOnce(&mut egui::Ui)) {
    ui.label(label);
    add(ui);
    ui.end_row();
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
            0 => form_properties(ui, state, i18n, &mut out),
            1 => {
                let name = state.selection[0].clone();
                control_properties(ui, state, &name, i18n, &mut out);
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
            ui.label(RichText::new(i18n.tr("prop-group-layout")).strong());
            ui.end_row();
            row(ui, i18n.tr("prop-x"), |ui| num(ui, &mut c.x, -1000.0));
            row(ui, i18n.tr("prop-y"), |ui| num(ui, &mut c.y, -1000.0));
            row(ui, i18n.tr("prop-width"), |ui| num(ui, &mut c.width, 8.0));
            row(ui, i18n.tr("prop-height"), |ui| num(ui, &mut c.height, 8.0));
            ui.label(RichText::new(i18n.tr("prop-group-behavior")).strong());
            ui.end_row();
            row(ui, i18n.tr("prop-visible"), |ui| {
                ui.checkbox(&mut c.visible, "");
            });
            row(ui, i18n.tr("prop-enabled"), |ui| {
                ui.checkbox(&mut c.enabled, "");
            });
        });
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
