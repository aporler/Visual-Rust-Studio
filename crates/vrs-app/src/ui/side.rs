//! Panneau de droite : boîte à outils et propriétés.

use crate::app::VrsApp;
use crate::commands::Command;
use crate::tabs::Tab;

pub fn properties_panel(app: &mut VrsApp, ui: &mut egui::Ui) {
    let i18n = &app.i18n;
    let mut open_handler = None;
    match app.tabs.get_mut(app.active) {
        Some(Tab::Designer(d)) => {
            egui::CollapsingHeader::new(i18n.tr("panel-toolbox"))
                .id_salt("vrs-toolbox")
                .default_open(true)
                .show(ui, |ui| vrs_designer::toolbox_ui(ui, Some(d), i18n));
            ui.separator();
            ui.heading(i18n.tr("panel-properties"));
            let out = vrs_designer::properties_ui(ui, d, i18n);
            open_handler = out.open_handler;
        }
        _ => {
            ui.heading(i18n.tr("panel-toolbox"));
            vrs_designer::toolbox_ui(ui, None, i18n);
            ui.separator();
            ui.heading(i18n.tr("panel-properties"));
        }
    }
    if let Some(req) = open_handler {
        app.cmd(Command::OpenHandler(req));
    }
}
