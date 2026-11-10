//! Barre de menus.

use vrs_compiler::CargoCommand;
use vrs_components::ops::LayoutOp;

use crate::app::VrsApp;
use crate::commands::Command;
use crate::tabs::Tab;

/// Ajoute une entrée de menu. Retourne vrai si elle a été cliquée.
fn item(app: &mut VrsApp, ui: &mut egui::Ui, key: &str, command: Command, enabled: bool) {
    let shortcut = super::shortcut_text(ui.ctx(), &command);
    let button = egui::Button::new(app.i18n.tr(key)).shortcut_text(shortcut);
    if ui.add_enabled(enabled, button).clicked() {
        app.cmd(command);
        ui.close();
    }
}

fn check(app: &mut VrsApp, ui: &mut egui::Ui, key: &str, value: bool, command: Command) {
    let mut v = value;
    if ui.checkbox(&mut v, app.i18n.tr(key)).clicked() {
        app.cmd(command);
        ui.close();
    }
}

pub fn menu_bar(app: &mut VrsApp, ui: &mut egui::Ui) {
    let has_project = app.project.is_some();
    let is_gui = app.project.as_ref().is_some_and(|p| p.is_gui());
    let building = app.is_building();
    let in_designer = matches!(app.active_tab(), Some(Tab::Designer(_)));
    let in_code = matches!(app.active_tab(), Some(Tab::Code(_)));
    let (can_undo, can_redo, has_sel, sel_count) = match app.active_tab() {
        Some(Tab::Designer(d)) => (
            d.can_undo(),
            d.can_redo(),
            d.has_selection(),
            d.selection.len(),
        ),
        _ => (false, false, false, 0),
    };
    let i18n_menu = |app: &VrsApp, k: &str| app.i18n.tr(k);

    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button(i18n_menu(app, "menu-file"), |ui| {
            item(app, ui, "cmd-new-project", Command::NewProject, true);
            item(app, ui, "cmd-open-project", Command::OpenProject, true);
            item(app, ui, "cmd-examples", Command::ShowExamples, true);
            let recent = app.settings.recent_projects.clone();
            ui.add_enabled_ui(!recent.is_empty(), |ui| {
                ui.menu_button(app.i18n.tr("menu-recent-projects"), |ui| {
                    for p in recent {
                        if ui.button(p.display().to_string()).clicked() {
                            app.cmd(Command::OpenPath(p));
                            ui.close();
                        }
                    }
                });
            });
            ui.separator();
            item(app, ui, "cmd-save", Command::Save, in_designer || in_code);
            item(app, ui, "cmd-save-all", Command::SaveAll, has_project);
            ui.separator();
            item(
                app,
                ui,
                "cmd-close-project",
                Command::CloseProject,
                has_project,
            );
            ui.separator();
            item(app, ui, "cmd-exit", Command::Exit, true);
        });
        ui.menu_button(i18n_menu(app, "menu-edit"), |ui| {
            item(app, ui, "cmd-undo", Command::Undo, can_undo);
            item(app, ui, "cmd-redo", Command::Redo, can_redo);
            ui.separator();
            item(app, ui, "cmd-cut", Command::Cut, has_sel);
            item(app, ui, "cmd-copy", Command::Copy, has_sel);
            item(
                app,
                ui,
                "cmd-paste",
                Command::Paste,
                in_designer && !app.clipboard.is_empty(),
            );
            item(app, ui, "cmd-delete", Command::Delete, has_sel);
            item(app, ui, "cmd-select-all", Command::SelectAll, in_designer);
            ui.separator();
            item(app, ui, "cmd-find", Command::Find, in_code);
            item(
                app,
                ui,
                "cmd-format-document",
                Command::FormatDocument,
                in_code,
            );
        });
        ui.menu_button(i18n_menu(app, "menu-view"), |ui| {
            check(
                app,
                ui,
                "cmd-view-explorer",
                app.show_explorer,
                Command::ToggleExplorer,
            );
            check(
                app,
                ui,
                "cmd-view-properties",
                app.show_properties,
                Command::ToggleProperties,
            );
            check(
                app,
                ui,
                "cmd-view-output",
                app.show_output,
                Command::ToggleOutput,
            );
            ui.separator();
            item(
                app,
                ui,
                "cmd-view-code",
                Command::ViewCode,
                in_designer || in_code,
            );
            item(
                app,
                ui,
                "cmd-view-designer",
                Command::ViewDesigner,
                in_code && is_gui,
            );
            ui.separator();
            item(app, ui, "tab-start", Command::ShowStart, true);
        });
        ui.menu_button(i18n_menu(app, "menu-project"), |ui| {
            item(app, ui, "cmd-add-form", Command::AddForm, is_gui);
            item(
                app,
                ui,
                "cmd-add-resource",
                Command::ImportImage(None),
                is_gui,
            );
            item(app, ui, "cmd-generate-code", Command::GenerateCode, is_gui);
            ui.separator();
            let last = app.project.as_ref().is_some_and(|p| {
                p.file.close_strategy == vrs_project::CloseStrategy::LastFormCloses
            });
            ui.add_enabled_ui(is_gui, |ui| {
                ui.menu_button(app.i18n.tr("menu-close-strategy"), |ui| {
                    if ui
                        .radio(!last, app.i18n.tr("close-strategy-main"))
                        .clicked()
                    {
                        app.cmd(Command::SetCloseStrategy(false));
                        ui.close();
                    }
                    if ui.radio(last, app.i18n.tr("close-strategy-last")).clicked() {
                        app.cmd(Command::SetCloseStrategy(true));
                        ui.close();
                    }
                });
            });
            ui.separator();
            item(
                app,
                ui,
                "cmd-reload-project",
                Command::ReloadProject,
                has_project,
            );
        });
        ui.menu_button(i18n_menu(app, "menu-build"), |ui| {
            let ok = has_project && !building;
            item(
                app,
                ui,
                "cmd-build",
                Command::Cargo(CargoCommand::Build),
                ok,
            );
            item(app, ui, "cmd-run", Command::Cargo(CargoCommand::Run), ok);
            item(
                app,
                ui,
                "cmd-check",
                Command::Cargo(CargoCommand::Check),
                ok,
            );
            item(
                app,
                ui,
                "cmd-clippy",
                Command::Cargo(CargoCommand::Clippy),
                ok,
            );
            item(app, ui, "cmd-test", Command::Cargo(CargoCommand::Test), ok);
            item(
                app,
                ui,
                "cmd-clean",
                Command::Cargo(CargoCommand::Clean),
                ok,
            );
            ui.separator();
            item(app, ui, "cmd-stop", Command::Stop, building);
            ui.separator();
            check(
                app,
                ui,
                "cmd-release-mode",
                app.settings.build.release,
                Command::ToggleRelease,
            );
        });
        ui.menu_button(i18n_menu(app, "menu-debug"), |ui| {
            let debugging = app.is_debugging();
            let paused = app.debug_paused();
            item(
                app,
                ui,
                "cmd-debug-start",
                Command::DebugStart,
                has_project && !debugging && !building,
            );
            item(
                app,
                ui,
                "cmd-run-without-debugging",
                Command::Cargo(CargoCommand::Run),
                has_project && !building && !debugging,
            );
            ui.separator();
            item(
                app,
                ui,
                "cmd-debug-continue",
                Command::DebugStartOrContinue,
                paused,
            );
            item(
                app,
                ui,
                "cmd-debug-pause",
                Command::Debug(crate::debug::DebugAction::Pause),
                debugging && !paused,
            );
            item(app, ui, "cmd-debug-stop", Command::Stop, debugging);
            ui.separator();
            item(
                app,
                ui,
                "cmd-step-over",
                Command::Debug(crate::debug::DebugAction::StepOver),
                paused,
            );
            item(
                app,
                ui,
                "cmd-step-into",
                Command::Debug(crate::debug::DebugAction::StepInto),
                paused,
            );
            item(
                app,
                ui,
                "cmd-step-out",
                Command::Debug(crate::debug::DebugAction::StepOut),
                paused,
            );
            ui.separator();
            item(
                app,
                ui,
                "cmd-toggle-breakpoint",
                Command::ToggleBreakpoint,
                in_code,
            );
            item(
                app,
                ui,
                "cmd-clear-breakpoints",
                Command::ClearBreakpoints,
                true,
            );
        });
        ui.add_enabled_ui(in_designer, |ui| {
            ui.menu_button(i18n_menu(app, "menu-format"), |ui| {
                for op in LayoutOp::ALL {
                    let enabled = sel_count >= op.min_selection();
                    item(app, ui, op.i18n_key(), Command::Layout(op), enabled);
                    if matches!(op, LayoutOp::AlignCenterV | LayoutOp::SameSize) {
                        ui.separator();
                    }
                }
                ui.separator();
                item(
                    app,
                    ui,
                    "cmd-bring-front",
                    Command::Reorder { to_front: true },
                    has_sel,
                );
                item(
                    app,
                    ui,
                    "cmd-send-back",
                    Command::Reorder { to_front: false },
                    has_sel,
                );
            });
        });
        ui.menu_button(i18n_menu(app, "menu-tools"), |ui| {
            item(app, ui, "cmd-settings", Command::Settings, true);
        });
        ui.menu_button(i18n_menu(app, "menu-window"), |ui| {
            item(
                app,
                ui,
                "cmd-close-tab",
                Command::CloseTab(app.active),
                !app.tabs.is_empty(),
            );
            item(
                app,
                ui,
                "cmd-close-all-tabs",
                Command::CloseAllTabs,
                !app.tabs.is_empty(),
            );
            ui.separator();
            let titles: Vec<String> = app.tabs.iter().map(|t| t.title(&app.i18n)).collect();
            for (i, t) in titles.into_iter().enumerate() {
                if ui.selectable_label(i == app.active, t).clicked() {
                    app.active = i;
                    ui.close();
                }
            }
        });
        ui.menu_button(i18n_menu(app, "menu-help"), |ui| {
            item(
                app,
                ui,
                "cmd-rust-book",
                Command::OpenUrl("https://doc.rust-lang.org/book/".into()),
                true,
            );
            ui.separator();
            item(app, ui, "cmd-about", Command::About, true);
        });
    });
}
