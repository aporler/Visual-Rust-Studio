//! Interface de l'IDE.

pub mod bottom;
pub mod central;
pub mod dialogs;
pub mod explorer;
pub mod menus;
pub mod side;
pub mod toolbar;

use egui::{Key, KeyboardShortcut, Modifiers};
use vrs_compiler::{CargoCommand, Level};

use crate::app::VrsApp;
use crate::commands::Command;
use crate::tabs::Tab;

pub fn sc(mods: Modifiers, key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(mods, key)
}

/// Raccourcis clavier globaux (style Visual Studio) : (raccourci, commande).
pub fn shortcut_table() -> Vec<(KeyboardShortcut, Command)> {
    let cmd = Modifiers::COMMAND;
    let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
    vec![
        (sc(cmd, Key::N), Command::NewProject),
        (sc(cmd, Key::O), Command::OpenProject),
        (sc(cmd_shift, Key::S), Command::SaveAll),
        (sc(cmd, Key::S), Command::Save),
        (sc(cmd_shift, Key::B), Command::Cargo(CargoCommand::Build)),
        (sc(Modifiers::SHIFT, Key::F5), Command::Stop),
        (
            sc(Modifiers::NONE, Key::F5),
            Command::Cargo(CargoCommand::Run),
        ),
        (sc(Modifiers::NONE, Key::F7), Command::ViewCode),
        (sc(Modifiers::SHIFT, Key::F7), Command::ViewDesigner),
        (sc(cmd, Key::F), Command::Find),
        (sc(cmd, Key::W), Command::CloseTab(usize::MAX)),
    ]
}

/// Raccourcis propres au concepteur (inactifs pendant la saisie de texte).
fn designer_shortcuts() -> Vec<(KeyboardShortcut, Command)> {
    let cmd = Modifiers::COMMAND;
    vec![
        (sc(cmd | Modifiers::SHIFT, Key::Z), Command::Redo),
        (sc(cmd, Key::Z), Command::Undo),
        (sc(cmd, Key::Y), Command::Redo),
        (sc(cmd, Key::C), Command::Copy),
        (sc(cmd, Key::X), Command::Cut),
        (sc(cmd, Key::V), Command::Paste),
        (sc(cmd, Key::A), Command::SelectAll),
        (sc(Modifiers::NONE, Key::Delete), Command::Delete),
    ]
}

/// Texte d'un raccourci pour les menus (« Ctrl+S »).
pub fn shortcut_text(ctx: &egui::Context, c: &Command) -> String {
    shortcut_table()
        .into_iter()
        .chain(designer_shortcuts())
        .find(|(_, cmd)| cmd == c)
        .map(|(s, _)| ctx.format_shortcut(&s))
        .unwrap_or_default()
}

/// Traite les raccourcis clavier.
pub fn shortcuts(app: &mut VrsApp, ctx: &egui::Context) {
    let modal_open = app.dialogs.new_project.is_some()
        || app.dialogs.settings.is_some()
        || app.dialogs.add_form.is_some()
        || app.dialogs.trust.is_some()
        || app.dialogs.unsaved.is_some();
    if modal_open {
        return;
    }
    for (s, c) in shortcut_table() {
        if ctx.input_mut(|i| i.consume_shortcut(&s)) {
            let c = match c {
                Command::CloseTab(_) => Command::CloseTab(app.active),
                other => other,
            };
            app.cmd(c);
        }
    }
    let typing = ctx.memory(|m| m.focused().is_some());
    if matches!(app.active_tab(), Some(Tab::Designer(_))) && !typing {
        for (s, c) in designer_shortcuts() {
            if ctx.input_mut(|i| i.consume_shortcut(&s)) {
                app.cmd(c);
            }
        }
    }
}

/// Barre d'état.
pub fn status_bar(app: &mut VrsApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if app.is_building() {
            ui.spinner();
        }
        ui.label(&app.status);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(app.i18n.active().to_string());
            ui.separator();
            let mode = if app.settings.build.release {
                "status-release"
            } else {
                "status-debug-build"
            };
            ui.label(app.i18n.tr(mode));
            ui.separator();
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
            let w = ui.link(
                app.i18n
                    .trf("status-warnings", &[("count", warnings.into())]),
            );
            let e = ui.link(app.i18n.trf("status-errors", &[("count", errors.into())]));
            if e.clicked() || w.clicked() {
                app.show_output = true;
                app.bottom_tab = crate::app::BottomTab::Errors;
            }
            if let (Some((l, c)), Some(Tab::Code(_))) = (app.cursor, app.active_tab()) {
                ui.separator();
                ui.label(
                    app.i18n
                        .trf("editor-line-col", &[("line", l.into()), ("col", c.into())]),
                );
            }
        });
    });
}
