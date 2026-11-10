//! Explorateur de projet.

use vrs_project::FileNode;

use crate::app::VrsApp;
use crate::commands::Command;

fn file_icon(name: &str) -> &'static str {
    if name.ends_with(".rs") {
        "📄"
    } else if name.ends_with(".vrsform") {
        "⏹"
    } else if name.ends_with(".toml") || name.ends_with(".vrsproj") {
        "⚙"
    } else {
        "📄"
    }
}

fn node_ui(app: &mut VrsApp, ui: &mut egui::Ui, node: &FileNode, depth: usize) {
    if node.is_dir {
        let open_default = depth == 0 || node.name == "src" || node.name == "forms";
        egui::CollapsingHeader::new(format!("📁 {}", node.name))
            .id_salt(&node.path)
            .default_open(open_default)
            .show(ui, |ui| {
                for c in &node.children {
                    node_ui(app, ui, c, depth + 1);
                }
            });
    } else {
        let r = ui.selectable_label(false, format!("{} {}", file_icon(&node.name), node.name));
        if r.double_clicked() || r.clicked() {
            app.cmd(Command::OpenFile {
                path: node.path.clone(),
                line: None,
            });
        }
    }
}

pub fn explorer(app: &mut VrsApp, ui: &mut egui::Ui) {
    ui.heading(app.i18n.tr("panel-explorer"));
    ui.separator();
    let Some(project) = app.project.clone() else {
        ui.weak(app.i18n.tr("explorer-no-project"));
        ui.add_space(8.0);
        if ui.button(app.i18n.tr("cmd-new-project")).clicked() {
            app.cmd(Command::NewProject);
        }
        if ui.button(app.i18n.tr("cmd-open-project")).clicked() {
            app.cmd(Command::OpenProject);
        }
        return;
    };
    egui::ScrollArea::both()
        .id_salt("vrs-explorer-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.strong(format!("📂 {}", project.name()));
            if project.is_gui() {
                let r = egui::CollapsingHeader::new(format!("⏹ {}", app.i18n.tr("explorer-forms")))
                    .id_salt("vrs-forms")
                    .default_open(true)
                    .show(ui, |ui| {
                        for form in &project.file.forms {
                            let startup =
                                project.file.startup_form.as_deref() == Some(form.as_str());
                            let label = if startup {
                                format!("{form} ★")
                            } else {
                                form.clone()
                            };
                            let r = ui.selectable_label(false, label);
                            if r.clicked() {
                                app.cmd(Command::OpenForm(form.clone()));
                            }
                            r.context_menu(|ui| {
                                if ui.button(app.i18n.tr("explorer-open-designer")).clicked() {
                                    app.cmd(Command::OpenForm(form.clone()));
                                    ui.close();
                                }
                                if ui.button(app.i18n.tr("explorer-view-code")).clicked() {
                                    let path = vrs_codegen::form_paths(&project.root, form).user;
                                    app.cmd(Command::OpenFile { path, line: None });
                                    ui.close();
                                }
                                if ui
                                    .add_enabled(
                                        !startup,
                                        egui::Button::new(app.i18n.tr("cmd-set-startup-form")),
                                    )
                                    .clicked()
                                {
                                    app.cmd(Command::SetStartupForm(form.clone()));
                                    ui.close();
                                }
                                if ui.button(app.i18n.tr("cmd-rename-form")).clicked() {
                                    app.cmd(Command::RenameForm(form.clone()));
                                    ui.close();
                                }
                            });
                        }
                        if ui
                            .small_button(format!("+ {}", app.i18n.tr("designer-add-window")))
                            .clicked()
                        {
                            app.cmd(Command::AddForm);
                        }
                    });
                r.header_response.context_menu(|ui| {
                    if ui.button(app.i18n.tr("cmd-add-form")).clicked() {
                        app.cmd(Command::AddForm);
                        ui.close();
                    }
                });
            }
            if let Some(tree) = app.tree.clone() {
                egui::CollapsingHeader::new(format!("📁 {}", app.i18n.tr("explorer-files")))
                    .id_salt("vrs-files")
                    .default_open(true)
                    .show(ui, |ui| {
                        for c in &tree.children {
                            node_ui(app, ui, c, 1);
                        }
                    });
            }
        });
}
