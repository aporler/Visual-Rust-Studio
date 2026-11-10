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
