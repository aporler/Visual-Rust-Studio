//! Zone centrale : onglets, concepteur, éditeur, page de démarrage.

use vrs_editor::EditorOptions;

use crate::app::VrsApp;
use crate::commands::Command;
use crate::tabs::Tab;

pub fn central(app: &mut VrsApp, ui: &mut egui::Ui) {
    // Barre d'onglets.
    let titles: Vec<String> = app.tabs.iter().map(|t| t.title(&app.i18n)).collect();
    egui::ScrollArea::horizontal()
        .id_salt("vrs-tabs")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (i, title) in titles.iter().enumerate() {
                    let r = ui.selectable_label(i == app.active, title);
                    if r.clicked() {
                        app.active = i;
                    }
                    if r.middle_clicked() {
                        app.cmd(Command::CloseTab(i));
                    }
                    if ui
                        .small_button("✖")
                        .on_hover_text(app.i18n.tr("cmd-close-tab"))
                        .clicked()
                    {
                        app.cmd(Command::CloseTab(i));
                    }
                    ui.add_space(6.0);
                }
            });
        });
    ui.separator();

    let opts = EditorOptions {
        font_size: app.settings.editor_font_size,
        tab_width: app.settings.tab_width,
    };
    let designer_settings = app.settings.designer.clone();
    let mut open_handler = None;
    let mut cursor = None;
    let mut start = false;
    match app.tabs.get_mut(app.active) {
        Some(Tab::Designer(d)) => {
            let out = vrs_designer::canvas_ui(ui, d, &designer_settings, &app.i18n);
            open_handler = out.open_handler;
        }
        Some(Tab::Code(e)) => {
            let out = e.show(ui, &app.i18n, &opts);
            cursor = out.cursor;
        }
        Some(Tab::Start) | None => start = true,
    }
    if cursor.is_some() {
        app.cursor = cursor;
    }
    if let Some(req) = open_handler {
        app.cmd(Command::OpenHandler(req));
    }
    if start {
        start_page(app, ui);
    }
}

fn start_page(app: &mut VrsApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .id_salt("vrs-start")
        .show(ui, |ui| {
            ui.add_space(24.0);
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new(app.i18n.tr("start-welcome")).size(28.0));
                ui.label(egui::RichText::new(app.i18n.tr("app-tagline")).italics());
            });
            ui.add_space(24.0);
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.set_max_width(360.0);
                    let big = |t: String| {
                        egui::Button::new(egui::RichText::new(t).size(16.0))
                            .min_size(egui::vec2(320.0, 40.0))
                    };
                    if ui
                        .add(big(format!("🗋  {}", app.i18n.tr("start-new-project"))))
                        .clicked()
                    {
                        app.cmd(Command::NewProject);
                    }
                    ui.add_space(6.0);
                    if ui
                        .add(big(format!("📂  {}", app.i18n.tr("start-open-project"))))
                        .clicked()
                    {
                        app.cmd(Command::OpenProject);
                    }
                    ui.add_space(6.0);
                    if ui
                        .add(big(format!(
                            "★  {}",
                            app.i18n.tr("cmd-examples").trim_end_matches('…')
                        )))
                        .clicked()
                    {
                        app.cmd(Command::ShowExamples);
                    }
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new(app.i18n.tr("start-learn")).weak());
                });
                ui.add_space(48.0);
                ui.vertical(|ui| {
                    ui.strong(app.i18n.tr("start-recent"));
                    ui.add_space(4.0);
                    let recent = app.settings.recent_projects.clone();
                    if recent.is_empty() {
                        ui.weak(app.i18n.tr("start-no-recent"));
                    }
                    for p in recent {
                        let name = p
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let r = ui.link(name).on_hover_text(p.display().to_string());
                        ui.small(p.display().to_string());
                        if r.clicked() {
                            app.cmd(Command::OpenPath(p));
                        }
                        ui.add_space(4.0);
                    }
                });
            });
        });
}
