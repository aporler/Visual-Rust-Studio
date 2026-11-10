//! Barre d'outils.

use vrs_compiler::CargoCommand;

use crate::app::VrsApp;
use crate::commands::Command;
use crate::tabs::Tab;

fn tool(
    app: &mut VrsApp,
    ui: &mut egui::Ui,
    icon: &str,
    key: &str,
    command: Command,
    enabled: bool,
) {
    let shortcut = super::shortcut_text(ui.ctx(), &command);
    let tip = if shortcut.is_empty() {
        app.i18n.tr(key)
    } else {
        format!("{} ({shortcut})", app.i18n.tr(key))
    };
    let r = ui
        .add_enabled(
            enabled,
            egui::Button::new(icon).min_size(egui::vec2(28.0, 24.0)),
        )
        .on_hover_text(&tip)
        .on_disabled_hover_text(&tip);
    if r.clicked() {
        app.cmd(command);
    }
}

pub fn toolbar(app: &mut VrsApp, ui: &mut egui::Ui) {
    let has_project = app.project.is_some();
    let building = app.is_building();
    let (can_undo, can_redo) = match app.active_tab() {
        Some(Tab::Designer(d)) => (d.can_undo(), d.can_redo()),
        _ => (false, false),
    };
    let can_save = app.active_tab().is_some_and(|t| !matches!(t, Tab::Start));
    ui.horizontal(|ui| {
        tool(app, ui, "🗋", "cmd-new-project", Command::NewProject, true);
        tool(
            app,
            ui,
            "📂",
            "cmd-open-project",
            Command::OpenProject,
            true,
        );
        tool(app, ui, "💾", "cmd-save", Command::Save, can_save);
        ui.separator();
        tool(app, ui, "⟲", "cmd-undo", Command::Undo, can_undo);
        tool(app, ui, "⟳", "cmd-redo", Command::Redo, can_redo);
        ui.separator();
        tool(
            app,
            ui,
            "🔨",
            "cmd-build",
            Command::Cargo(CargoCommand::Build),
            has_project && !building,
        );
        tool(
            app,
            ui,
            "▶",
            "cmd-run",
            Command::Cargo(CargoCommand::Run),
            has_project && !building,
        );
        tool(app, ui, "⏹", "cmd-stop", Command::Stop, building);
        tool(
            app,
            ui,
            "✔",
            "cmd-check",
            Command::Cargo(CargoCommand::Check),
            has_project && !building,
        );
        ui.add_enabled(
            false,
            egui::Button::new("🐞").min_size(egui::vec2(28.0, 24.0)),
        )
        .on_disabled_hover_text(format!(
            "{} — {}",
            app.i18n.tr("cmd-debug-start"),
            app.i18n.tr("debug-not-available")
        ));
        ui.separator();
        let mut release = app.settings.build.release;
        if ui
            .toggle_value(&mut release, app.i18n.tr("status-release"))
            .on_hover_text(app.i18n.tr("cmd-release-mode"))
            .clicked()
        {
            app.cmd(Command::ToggleRelease);
        }
        ui.separator();
        tool(app, ui, "⚙", "cmd-settings", Command::Settings, true);
    });
}
