//! Panneau inférieur : sortie de Cargo et liste d'erreurs.

use vrs_compiler::Level;

use crate::app::{BottomTab, LineKind, VrsApp};
use crate::commands::Command;

pub fn bottom_panel(app: &mut VrsApp, ui: &mut egui::Ui) {
    let errors = app
        .diagnostics
        .iter()
        .filter(|d| d.level == Level::Error)
        .count();
    let warnings = app
        .diagnostics
        .iter()
        .filter(|d| d.level == Level::Warning)
        .count();
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut app.bottom_tab,
            BottomTab::Output,
            app.i18n.tr("panel-output"),
        );
        ui.selectable_value(
            &mut app.bottom_tab,
            BottomTab::Errors,
            format!("{} ({})", app.i18n.tr("panel-errors"), errors + warnings),
        );
        ui.selectable_value(
            &mut app.bottom_tab,
            BottomTab::Debug,
            app.i18n.tr("panel-debug"),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if app.bottom_tab == BottomTab::Output
                && ui.small_button(app.i18n.tr("button-clear")).clicked()
            {
                app.console.clear();
            }
            if app.bottom_tab == BottomTab::Errors {
                ui.checkbox(
                    &mut app.show_warnings,
                    app.i18n
                        .trf("status-warnings", &[("count", warnings.into())]),
                );
                ui.checkbox(
                    &mut app.show_errors,
                    app.i18n.trf("status-errors", &[("count", errors.into())]),
                );
            }
        });
    });
    ui.separator();
    match app.bottom_tab {
        BottomTab::Output => output(app, ui),
        BottomTab::Errors => error_list(app, ui),
        BottomTab::Debug => debug_panel(app, ui),
    }
}

fn output(app: &mut VrsApp, ui: &mut egui::Ui) {
    if app.console.is_empty() {
        ui.weak(app.i18n.tr("output-empty"));
        return;
    }
    let v = ui.visuals().clone();
    egui::ScrollArea::both()
        .id_salt("vrs-console")
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show_rows(
            ui,
            ui.text_style_height(&egui::TextStyle::Monospace),
            app.console.len(),
            |ui, range| {
                for line in &app.console[range] {
                    let color = match line.kind {
                        LineKind::Info => v.weak_text_color(),
                        LineKind::Stdout => v.text_color(),
                        LineKind::Stderr => v.text_color().gamma_multiply(0.8),
                        LineKind::Success => egui::Color32::from_rgb(60, 170, 90),
                        LineKind::Failure => v.error_fg_color,
                    };
                    ui.add(
                        egui::Label::new(egui::RichText::new(&line.text).monospace().color(color))
                            .extend(),
                    );
                }
            },
        );
}

fn error_list(app: &mut VrsApp, ui: &mut egui::Ui) {
    if app.diagnostics.is_empty() {
        ui.weak(app.i18n.tr("errors-empty"));
        return;
    }
    let root = app.project.as_ref().map(|p| p.root.clone());
    let mut open = None;
    egui::ScrollArea::vertical()
        .id_salt("vrs-errors")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (i, d) in app.diagnostics.iter().enumerate() {
                let visible = match d.level {
                    Level::Error => app.show_errors,
                    Level::Warning => app.show_warnings,
                    _ => true,
                };
                if !visible {
                    continue;
                }
                let (icon, color) = match d.level {
                    Level::Error => ("✖", ui.visuals().error_fg_color),
                    Level::Warning => ("⚠", ui.visuals().warn_fg_color),
                    _ => ("ℹ", ui.visuals().text_color()),
                };
                let location = d
                    .span
                    .as_ref()
                    .map(|s| {
                        let file = root
                            .as_ref()
                            .and_then(|r| s.file.strip_prefix(r).ok())
                            .unwrap_or(&s.file);
                        format!("{}:{}:{}", file.display(), s.line_start, s.column_start)
                    })
                    .unwrap_or_default();
                let selected = app.selected_diagnostic == Some(i);
                let header = format!(
                    "{icon} {}  {}  —  {location}",
                    d.code.as_deref().unwrap_or(""),
                    d.message
                );
                let r = ui.selectable_label(selected, egui::RichText::new(header).color(color));
                if r.clicked() {
                    app.selected_diagnostic = if selected { None } else { Some(i) };
                }
                if r.double_clicked()
                    && let Some(s) = &d.span
                {
                    open = Some((s.file.clone(), s.line_start));
                }
                if selected {
                    ui.indent(("diag", i), |ui| {
                        if let Some(key) = d.explanation_key()
                            && let Some(text) = app.i18n.try_tr(&key, None)
                        {
                            ui.strong(format!("💡 {}", app.i18n.tr("diag-explanation")));
                            ui.label(text);
                            ui.add_space(4.0);
                        }
                        for s in &d.suggestions {
                            ui.label(app.i18n.trf(
                                "diag-suggestion",
                                &[("replacement", s.replacement.clone().into())],
                            ));
                        }
                        ui.horizontal(|ui| {
                            if let Some(s) = &d.span
                                && ui.button(app.i18n.tr("diag-open-location")).clicked()
                            {
                                open = Some((s.file.clone(), s.line_start));
                            }
                            if let Some(url) = d.doc_url() {
                                ui.hyperlink_to("doc.rust-lang.org", url);
                            }
                        });
                        egui::CollapsingHeader::new(app.i18n.tr("diag-original"))
                            .id_salt(("orig", i))
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(egui::RichText::new(&d.rendered).monospace())
                                        .extend(),
                                );
                            });
                    });
                }
            }
        });
    if let Some((path, line)) = open {
        app.cmd(Command::OpenFile {
            path,
            line: Some(line),
        });
    }
}

fn debug_panel(app: &mut VrsApp, ui: &mut egui::Ui) {
    let Some(d) = &app.debug else {
        ui.weak(app.i18n.tr("debug-idle"));
        return;
    };
    let frames = d.frames.clone();
    let selected = d.selected_frame;
    let locals = d.locals;
    let variables = d.variables.clone();
    let threads = d.threads.clone();
    let paused = d.paused;
    let mut select = None;
    let mut expand = Vec::new();
    let mut remove_watch = None;
    let mut add_watch = None;
    ui.columns(3, |cols| {
        // Pile d'appels.
        cols[0].strong(app.i18n.tr("debug-call-stack"));
        egui::ScrollArea::vertical()
            .id_salt("vrs-stack")
            .show(&mut cols[0], |ui| {
                if !paused {
                    ui.weak(app.i18n.tr("debug-running"));
                }
                for (i, f) in frames.iter().enumerate() {
                    let file = f
                        .path
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .map(|n| format!("{}:{}", n.to_string_lossy(), f.line))
                        .unwrap_or_default();
                    if ui
                        .selectable_label(
                            i == selected,
                            egui::RichText::new(format!("{}  {file}", f.name)).monospace(),
                        )
                        .clicked()
                    {
                        select = Some(i);
                    }
                }
                if !threads.is_empty() {
                    ui.separator();
                    ui.strong(app.i18n.tr("debug-threads"));
                    for t in &threads {
                        ui.monospace(format!("#{} {}", t.id, t.name));
                    }
                }
            });
        // Variables locales.
        cols[1].strong(app.i18n.tr("debug-locals"));
        egui::ScrollArea::vertical()
            .id_salt("vrs-locals")
            .show(&mut cols[1], |ui| {
                if let Some(r) = locals {
                    variables_tree(ui, r, &variables, &mut expand, 0);
                }
            });
        // Espions.
        cols[2].strong(app.i18n.tr("debug-watch"));
        egui::ScrollArea::vertical()
            .id_salt("vrs-watch")
            .show(&mut cols[2], |ui| {
                for (i, w) in app.watches.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.small_button("✖").clicked() {
                            remove_watch = Some(i);
                        }
                        ui.monospace(format!(
                            "{} = {}",
                            w.expression,
                            w.value.clone().unwrap_or_else(|| "…".into())
                        ));
                    });
                }
                ui.horizontal(|ui| {
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut app.watch_input)
                            .hint_text(app.i18n.tr("debug-watch-add"))
                            .desired_width(180.0),
                    );
                    if r.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !app.watch_input.trim().is_empty()
                    {
                        add_watch = Some(app.watch_input.trim().to_owned());
                    }
                });
            });
    });
    if let Some(i) = select {
        app.select_frame(i);
    }
    for r in expand {
        app.load_variables(r);
    }
    if let Some(i) = remove_watch {
        app.watches.remove(i);
    }
    if let Some(e) = add_watch {
        app.watches.push(crate::debug::Watch {
            expression: e,
            value: None,
        });
        app.watch_input.clear();
        app.refresh_watches();
    }
}

fn variables_tree(
    ui: &mut egui::Ui,
    reference: i64,
    variables: &std::collections::HashMap<i64, Vec<vrs_debugger::Variable>>,
    expand: &mut Vec<i64>,
    depth: usize,
) {
    let Some(list) = variables.get(&reference) else {
        ui.spinner();
        expand.push(reference);
        return;
    };
    for v in list {
        let ty = v
            .type_name
            .as_deref()
            .map(|t| format!(": {t}"))
            .unwrap_or_default();
        let text = format!("{}{ty} = {}", v.name, v.value);
        if v.children > 0 && depth < 8 {
            egui::CollapsingHeader::new(egui::RichText::new(text).monospace())
                .id_salt(("var", reference, &v.name))
                .show(ui, |ui| {
                    variables_tree(ui, v.children, variables, expand, depth + 1)
                });
        } else {
            ui.monospace(text);
        }
    }
}
