//! Boîtes de dialogue.

use std::path::PathBuf;

use egui::{Modal, RichText};
use vrs_i18n::LanguageIdentifier;
use vrs_settings::Theme;

use crate::actions::TEMPLATES;
use crate::app::VrsApp;

pub fn dialogs(app: &mut VrsApp, ctx: &egui::Context) {
    new_project(app, ctx);
    settings(app, ctx);
    add_form(app, ctx);
    trust(app, ctx);
    unsaved(app, ctx);
    about(app, ctx);
    error(app, ctx);
}

fn buttons_right<R>(
    ui: &mut egui::Ui,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add)
}

fn new_project(app: &mut VrsApp, ctx: &egui::Context) {
    let Some(mut dlg) = app.dialogs.new_project.take() else {
        return;
    };
    let mut keep = true;
    let mut create = None;
    Modal::new(egui::Id::unique("vrs-new-project")).show(ctx, |ui| {
        ui.set_width(560.0);
        ui.heading(app.i18n.tr("newproj-title"));
        ui.separator();
        ui.strong(app.i18n.tr("newproj-template"));
        for (i, (_, key, desc)) in TEMPLATES.iter().enumerate() {
            ui.radio_value(
                &mut dlg.template,
                i,
                RichText::new(app.i18n.tr(key)).strong(),
            );
            ui.indent(("tpl", i), |ui| ui.weak(app.i18n.tr(desc)));
        }
        ui.separator();
        let existing = TEMPLATES[dlg.template].0.is_none();
        let valid = vrs_core::ident::is_valid_crate_name(&dlg.name);
        let target = PathBuf::from(&dlg.location).join(&dlg.name);
        let exists = !existing && target.exists();
        egui::Grid::new("newproj-grid")
            .num_columns(2)
            .show(ui, |ui| {
                if !existing {
                    ui.label(app.i18n.tr("newproj-name"));
                    ui.text_edit_singleline(&mut dlg.name);
                    ui.end_row();
                }
                ui.label(app.i18n.tr("newproj-location"));
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut dlg.location).desired_width(320.0));
                    if ui.button(app.i18n.tr("button-browse")).clicked()
                        && let Some(dir) = rfd::FileDialog::new()
                            .set_title(app.i18n.tr("newproj-choose-folder"))
                            .set_directory(&dlg.location)
                            .pick_folder()
                    {
                        dlg.location = dir.display().to_string();
                    }
                });
                ui.end_row();
            });
        if !existing {
            if !valid {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    app.i18n.tr("newproj-invalid-name"),
                );
            } else if exists {
                ui.colored_label(ui.visuals().error_fg_color, app.i18n.tr("newproj-exists"));
            } else {
                ui.weak(app.i18n.trf(
                    "newproj-path-preview",
                    &[("path", target.display().to_string().into())],
                ));
            }
        }
        ui.separator();
        buttons_right(ui, |ui| {
            if ui.button(app.i18n.tr("button-cancel")).clicked() {
                keep = false;
            }
            let label = if existing {
                "button-open"
            } else {
                "button-create"
            };
            let enabled = existing || (valid && !exists && !dlg.location.trim().is_empty());
            if ui
                .add_enabled(enabled, egui::Button::new(app.i18n.tr(label)))
                .clicked()
            {
                create = Some((
                    TEMPLATES[dlg.template].0,
                    PathBuf::from(&dlg.location),
                    dlg.name.clone(),
                ));
                keep = false;
            }
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        keep = false;
    }
    if keep {
        app.dialogs.new_project = Some(dlg);
    }
    match create {
        Some((Some(kind), parent, name)) => app.create_project(kind, &parent, &name),
        Some((None, parent, _)) => {
            if let Some(dir) = rfd::FileDialog::new()
                .set_title(app.i18n.tr("template-existing"))
                .set_directory(parent)
                .pick_folder()
            {
                app.open_project_at(&dir);
            }
        }
        None => {}
    }
}

fn settings(app: &mut VrsApp, ctx: &egui::Context) {
    let Some(mut dlg) = app.dialogs.settings.take() else {
        return;
    };
    let mut keep = true;
    let mut ok = false;
    let mut language_changed = false;
    Modal::new(egui::Id::unique("vrs-settings")).show(ctx, |ui| {
        ui.set_width(560.0);
        ui.heading(app.i18n.tr("settings-title"));
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(480.0)
            .show(ui, |ui| {
                // --- Langue ----------------------------------------------------
                ui.strong(app.i18n.tr("settings-language"));
                let system = crate::lang::system_locale()
                    .map(|l| l.to_string())
                    .unwrap_or_else(|| "?".into());
                let available: Vec<LanguageIdentifier> = app.i18n.catalog().available();
                let current = app.settings.language.clone();
                let label_for = |code: &Option<String>| match code {
                    None => app
                        .i18n
                        .trf("settings-language-auto", &[("lang", system.clone().into())]),
                    Some(c) => c
                        .parse::<LanguageIdentifier>()
                        .map(|l| format!("{} — {c}", app.i18n.catalog().display_name(&l)))
                        .unwrap_or_else(|_| c.clone()),
                };
                let mut selected = current.clone();
                egui::ComboBox::from_id_salt("vrs-lang")
                    .width(360.0)
                    .selected_text(label_for(&selected))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, None, label_for(&None));
                        for l in &available {
                            let code = Some(l.to_string());
                            ui.selectable_value(&mut selected, code.clone(), label_for(&code));
                        }
                    });
                if selected != current {
                    app.settings.language = selected;
                    language_changed = true;
                }
                let chain: Vec<String> = app.i18n.chain().iter().map(|l| l.to_string()).collect();
                ui.weak(app.i18n.trf(
                    "settings-language-chain",
                    &[("chain", chain.join(" → ").into())],
                ));
                if let Some(dir) = vrs_settings::external_locales_dir() {
                    ui.weak(app.i18n.trf(
                        "settings-language-external",
                        &[("path", dir.display().to_string().into())],
                    ));
                }
                ui.separator();

                // --- Apparence -------------------------------------------------
                ui.strong(app.i18n.tr("settings-appearance"));
                egui::Grid::new("settings-appearance")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(app.i18n.tr("settings-theme"));
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut app.settings.theme,
                                Theme::System,
                                app.i18n.tr("theme-system"),
                            );
                            ui.radio_value(
                                &mut app.settings.theme,
                                Theme::Light,
                                app.i18n.tr("theme-light"),
                            );
                            ui.radio_value(
                                &mut app.settings.theme,
                                Theme::Dark,
                                app.i18n.tr("theme-dark"),
                            );
                        });
                        ui.end_row();
                        ui.label(app.i18n.tr("settings-ui-scale"));
                        ui.add(
                            egui::Slider::new(&mut app.settings.ui_scale, 0.75..=2.0).step_by(0.05),
                        );
                        ui.end_row();
                    });
                ui.separator();

                // --- Éditeur ---------------------------------------------------
                ui.strong(app.i18n.tr("settings-editor"));
                egui::Grid::new("settings-editor")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(app.i18n.tr("settings-editor-font-size"));
                        ui.add(
                            egui::Slider::new(&mut app.settings.editor_font_size, 8.0..=32.0)
                                .step_by(1.0),
                        );
                        ui.end_row();
                        ui.label(app.i18n.tr("settings-tab-width"));
                        ui.add(egui::Slider::new(&mut app.settings.tab_width, 1..=8));
                        ui.end_row();
                    });
                ui.checkbox(
                    &mut app.settings.format_on_save,
                    app.i18n.tr("settings-format-on-save"),
                );
                ui.separator();

                // --- Concepteur ------------------------------------------------
                ui.strong(app.i18n.tr("settings-designer"));
                egui::Grid::new("settings-designer")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(app.i18n.tr("settings-grid-size"));
                        ui.add(
                            egui::Slider::new(&mut app.settings.designer.grid_size, 2.0..=32.0)
                                .step_by(1.0),
                        );
                        ui.end_row();
                    });
                ui.checkbox(
                    &mut app.settings.designer.snap_to_grid,
                    app.i18n.tr("settings-snap-to-grid"),
                );
                ui.checkbox(
                    &mut app.settings.designer.show_grid,
                    app.i18n.tr("settings-show-grid"),
                );
                ui.separator();

                // --- Chaîne d'outils -------------------------------------------
                ui.strong(app.i18n.tr("settings-toolchain"));
                ui.horizontal(|ui| {
                    ui.label(app.i18n.tr("settings-cargo-path"));
                    ui.add(
                        egui::TextEdit::singleline(&mut dlg.cargo_path)
                            .hint_text(app.i18n.tr("settings-cargo-path-hint"))
                            .desired_width(300.0),
                    );
                });
                let configured = (!dlg.cargo_path.trim().is_empty())
                    .then(|| PathBuf::from(dlg.cargo_path.trim()));
                match vrs_compiler::find_cargo(configured.as_deref()) {
                    Some(p) => ui.weak(app.i18n.trf(
                        "settings-cargo-detected",
                        &[("path", p.display().to_string().into())],
                    )),
                    None => ui.colored_label(
                        ui.visuals().error_fg_color,
                        app.i18n.tr("settings-cargo-not-found"),
                    ),
                };
                if let Some(p) = &app.settings_path {
                    ui.add_space(6.0);
                    ui.weak(app.i18n.trf(
                        "settings-config-file",
                        &[("path", p.display().to_string().into())],
                    ));
                }
            });
        ui.separator();
        buttons_right(ui, |ui| {
            if ui.button(app.i18n.tr("button-cancel")).clicked() {
                keep = false;
            }
            if ui.button(app.i18n.tr("button-ok")).clicked() {
                ok = true;
                keep = false;
            }
        });
    });
    if language_changed {
        // Changement immédiat, sans redémarrage.
        let lang = crate::lang::requested_language(app.settings.language.as_deref());
        app.i18n.set_language(lang);
        app.status = app.i18n.tr("status-language-changed");
    }
    if !keep {
        if ok {
            let p = dlg.cargo_path.trim();
            app.settings.build.cargo_path = (!p.is_empty()).then(|| PathBuf::from(p));
            app.settings = app.settings.clone().sanitized();
            app.save_settings();
        } else {
            let lang_before = dlg.original.language.clone();
            app.settings = dlg.original.clone();
            if lang_before != app.i18n.requested().to_string().into() {
                let lang = crate::lang::requested_language(lang_before.as_deref());
                app.i18n.set_language(lang);
            }
        }
    } else {
        app.dialogs.settings = Some(dlg);
    }
}

fn add_form(app: &mut VrsApp, ctx: &egui::Context) {
    let Some(mut name) = app.dialogs.add_form.take() else {
        return;
    };
    let mut keep = true;
    let mut create = false;
    let check = app.project.as_ref().map(|p| p.check_form_name(&name));
    Modal::new(egui::Id::unique("vrs-add-form")).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.heading(app.i18n.tr("addform-title"));
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(app.i18n.tr("addform-name"));
            let r = ui.text_edit_singleline(&mut name);
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                create = matches!(check, Some(Ok(())));
            }
        });
        match &check {
            Some(Err(vrs_project::ProjectError::AlreadyExists(_))) => {
                ui.colored_label(ui.visuals().error_fg_color, app.i18n.tr("addform-exists"));
            }
            Some(Err(_)) => {
                ui.colored_label(ui.visuals().error_fg_color, app.i18n.tr("addform-invalid"));
            }
            _ => {}
        }
        ui.separator();
        buttons_right(ui, |ui| {
            if ui.button(app.i18n.tr("button-cancel")).clicked() {
                keep = false;
            }
            if ui
                .add_enabled(
                    matches!(check, Some(Ok(()))),
                    egui::Button::new(app.i18n.tr("button-create")),
                )
                .clicked()
            {
                create = true;
            }
        });
    });
    if create {
        app.add_form(&name);
    } else if keep {
        app.dialogs.add_form = Some(name);
    }
}

fn trust(app: &mut VrsApp, ctx: &egui::Context) {
    let Some(req) = app.dialogs.trust.take() else {
        return;
    };
    let mut keep = true;
    let mut granted = false;
    Modal::new(egui::Id::unique("vrs-trust")).show(ctx, |ui| {
        ui.set_width(480.0);
        ui.heading(format!("⚠ {}", app.i18n.tr("trust-title")));
        ui.separator();
        ui.label(app.i18n.trf(
            "trust-message",
            &[("path", req.root.display().to_string().into())],
        ));
        ui.separator();
        buttons_right(ui, |ui| {
            if ui.button(app.i18n.tr("button-cancel")).clicked() {
                keep = false;
            }
            if ui.button(app.i18n.tr("button-trust")).clicked() {
                granted = true;
                keep = false;
            }
        });
    });
    if granted {
        app.grant_trust(req, ctx);
    } else if keep {
        app.dialogs.trust = Some(req);
    }
}

fn unsaved(app: &mut VrsApp, ctx: &egui::Context) {
    let Some(then) = app.dialogs.unsaved.take() else {
        return;
    };
    let mut keep = true;
    let mut choice = None;
    Modal::new(egui::Id::unique("vrs-unsaved")).show(ctx, |ui| {
        ui.set_width(420.0);
        ui.heading(app.i18n.tr("unsaved-title"));
        ui.separator();
        ui.label(app.i18n.tr("unsaved-message"));
        ui.separator();
        buttons_right(ui, |ui| {
            if ui.button(app.i18n.tr("button-cancel")).clicked() {
                keep = false;
            }
            if ui.button(app.i18n.tr("button-discard")).clicked() {
                choice = Some(false);
            }
            if ui.button(app.i18n.tr("button-save")).clicked() {
                choice = Some(true);
            }
        });
    });
    match choice {
        Some(true) => {
            if app.save_all() {
                app.discard_and_continue(then, ctx);
            }
        }
        Some(false) => app.discard_and_continue(then, ctx),
        None if keep => app.dialogs.unsaved = Some(then),
        None => {}
    }
}

fn about(app: &mut VrsApp, ctx: &egui::Context) {
    if !app.dialogs.about {
        return;
    }
    let r = Modal::new(egui::Id::unique("vrs-about")).show(ctx, |ui| {
        ui.set_width(400.0);
        ui.heading(app.i18n.tr("about-title"));
        ui.separator();
        ui.label(app.i18n.trf(
            "about-version",
            &[("version", env!("CARGO_PKG_VERSION").into())],
        ));
        ui.label(app.i18n.tr("about-description"));
        ui.label(RichText::new(app.i18n.tr("app-tagline")).italics());
        ui.add_space(6.0);
        ui.weak(app.i18n.tr("about-license"));
        ui.separator();
        buttons_right(ui, |ui| ui.button(app.i18n.tr("button-close")).clicked()).inner
    });
    if r.inner || r.should_close() {
        app.dialogs.about = false;
    }
}

fn error(app: &mut VrsApp, ctx: &egui::Context) {
    let Some(text) = app.dialogs.error.clone() else {
        return;
    };
    let r = Modal::new(egui::Id::unique("vrs-error")).show(ctx, |ui| {
        ui.set_width(460.0);
        ui.heading(format!("✖ {}", app.i18n.tr("error-title")));
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                ui.label(&text);
            });
        ui.separator();
        buttons_right(ui, |ui| ui.button(app.i18n.tr("button-ok")).clicked()).inner
    });
    if r.inner || r.should_close() {
        app.dialogs.error = None;
    }
}
