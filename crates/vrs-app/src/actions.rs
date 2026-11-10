//! Exécution des commandes.

use std::path::{Path, PathBuf};

use vrs_compiler::{BuildEvent, CargoCommand, Level, OutputStream};
use vrs_designer::{DesignerState, HandlerRequest};
use vrs_editor::assist::Fix;
use vrs_editor::{CodeEditor, EditorDiagnostic, Severity};
use vrs_project::{Project, ProjectKind};

use crate::app::{BottomTab, LineKind, NewProjectDialog, SettingsDialog, TrustRequest, VrsApp};
use crate::commands::Command;
use crate::lang::I18nComments;
use crate::tabs::{Tab, same_path};

/// Modèles de l'assistant, dans l'ordre affiché.
pub const TEMPLATES: [(Option<ProjectKind>, &str, &str); 4] = [
    (Some(ProjectKind::Gui), "template-gui", "template-gui-desc"),
    (
        Some(ProjectKind::Console),
        "template-console",
        "template-console-desc",
    ),
    (
        Some(ProjectKind::Library),
        "template-library",
        "template-library-desc",
    ),
    (None, "template-existing", "template-existing-desc"),
];

impl VrsApp {
    /// Exécute une commande.
    pub fn execute(&mut self, command: Command, ctx: &egui::Context) {
        match command {
            Command::NewProject => {
                let location = self
                    .settings
                    .projects_dir
                    .clone()
                    .unwrap_or_else(vrs_settings::default_projects_dir);
                let name = (1..)
                    .map(|i| format!("WindowsApp{i}"))
                    .find(|n| !location.join(n).exists())
                    .unwrap_or_default();
                self.dialogs.new_project = Some(NewProjectDialog {
                    template: 0,
                    name,
                    location: location.display().to_string(),
                });
            }
            Command::OpenProject => {
                let dir = self.settings.projects_dir.clone();
                let mut dialog = rfd::FileDialog::new()
                    .set_title(self.i18n.tr("cmd-open-project"))
                    .add_filter("Visual Rust Studio", &["vrsproj"])
                    .add_filter("Cargo", &["toml"]);
                if let Some(d) = dir {
                    dialog = dialog.set_directory(d);
                }
                if let Some(path) = dialog.pick_file() {
                    self.open_project_at(&path);
                }
            }
            Command::OpenPath(path) => self.open_project_at(&path),
            Command::Save => self.save_active(),
            Command::SaveAll => {
                self.save_all();
            }
            Command::CloseProject => {
                if self.guard_unsaved(Command::CloseProject) {
                    self.close_project();
                }
            }
            Command::Exit => {
                if self.dialogs.unsaved.is_none() && self.has_unsaved() {
                    self.dialogs.unsaved = Some(Command::Exit);
                    return;
                }
                self.exit_confirmed = true;
                self.on_exit();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Command::Undo => {
                if let Some(d) = self.active_designer() {
                    d.undo();
                }
            }
            Command::Redo => {
                if let Some(d) = self.active_designer() {
                    d.redo();
                }
            }
            Command::Copy | Command::Cut => {
                let cut = command == Command::Cut;
                if let Some(d) = self.active_designer() {
                    let copied = d.copy_selection();
                    if cut {
                        d.delete_selection();
                    }
                    if !copied.is_empty() {
                        self.clipboard = copied;
                    }
                }
            }
            Command::Paste => {
                let clip = self.clipboard.clone();
                let grid = self.settings.designer.grid_size;
                if let Some(d) = self.active_designer() {
                    d.paste(&clip, grid * 2.0);
                }
            }
            Command::Delete => {
                if let Some(d) = self.active_designer() {
                    d.delete_selection();
                }
            }
            Command::SelectAll => {
                if let Some(d) = self.active_designer() {
                    d.select_all();
                }
            }
            Command::Find => {
                if let Some(Tab::Code(e)) = self.active_tab_mut() {
                    e.open_find();
                }
            }
            Command::FormatDocument => self.format_active(),
            Command::ToggleExplorer => self.show_explorer = !self.show_explorer,
            Command::ToggleProperties => self.show_properties = !self.show_properties,
            Command::ToggleOutput => self.show_output = !self.show_output,
            Command::ViewCode => self.view_code(),
            Command::ViewDesigner => self.view_designer(),
            Command::AddForm => {
                if let Some(p) = &self.project
                    && p.is_gui()
                {
                    self.dialogs.add_form = Some(p.next_form_name());
                }
            }
            Command::GenerateCode => {
                self.save_all();
            }
            Command::ReloadProject => {
                if let Some(p) = self.project.as_ref().map(|p| p.root.clone()) {
                    self.open_project_at(&p);
                }
            }
            Command::Cargo(c) => self.run_cargo(c),
            Command::Stop => {
                if let Some(b) = &self.build {
                    b.kill();
                }
            }
            Command::ToggleRelease => {
                self.settings.build.release = !self.settings.build.release;
                self.save_settings();
            }
            Command::Settings => {
                self.dialogs.settings = Some(SettingsDialog {
                    original: self.settings.clone(),
                    cargo_path: self
                        .settings
                        .build
                        .cargo_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                });
            }
            Command::CloseTab(i) => self.close_tab(i),
            Command::CloseAllTabs => {
                if self.guard_unsaved(Command::CloseAllTabs) {
                    self.tabs.clear();
                    self.active = 0;
                }
            }
            Command::About => self.dialogs.about = true,
            Command::OpenUrl(url) => ctx.open_url(egui::OpenUrl::new_tab(url)),
            Command::Layout(op) => {
                if let Some(d) = self.active_designer() {
                    d.apply_layout(op);
                }
            }
            Command::Reorder { to_front } => {
                if let Some(d) = self.active_designer() {
                    d.reorder(to_front);
                }
            }
            Command::OpenForm(name) => self.open_form(&name),
            Command::OpenFile { path, line } => self.open_file(&path, line),
            Command::OpenHandler(req) => self.open_handler(req),
            Command::ShowStart => {
                if let Some(i) = self.tabs.iter().position(|t| matches!(t, Tab::Start)) {
                    self.active = i;
                } else {
                    self.tabs.insert(0, Tab::Start);
                    self.active = 0;
                }
            }
        }
    }

    /// S'il reste des modifications, demande quoi faire et retourne `false`.
    fn guard_unsaved(&mut self, then: Command) -> bool {
        if self.has_unsaved() && self.dialogs.unsaved.is_none() {
            self.dialogs.unsaved = Some(then);
            return false;
        }
        true
    }

    // ----- Projets ---------------------------------------------------------

    /// Ouvre un projet (fichier .vrsproj, Cargo.toml ou dossier).
    pub fn open_project_at(&mut self, path: &Path) {
        match vrs_project::open_project(path) {
            Ok(p) => self.set_project(p),
            Err(e) => {
                let msg = self
                    .i18n
                    .trf("error-open-project", &[("error", e.to_string().into())]);
                self.error(msg);
            }
        }
    }

    fn set_project(&mut self, project: Project) {
        self.tabs.retain(|t| matches!(t, Tab::Start));
        self.active = 0;
        self.diagnostics.clear();
        let recent = project
            .file_path
            .clone()
            .unwrap_or_else(|| project.root.clone());
        self.settings.push_recent(&recent);
        self.save_settings();
        self.status = self.i18n.trf(
            "status-project-opened",
            &[("name", project.name().to_owned().into())],
        );
        let first_form = project.file.startup_form.clone();
        let main_file = ["src/main.rs", "src/lib.rs"]
            .iter()
            .map(|f| project.root.join(f))
            .find(|p| p.is_file());
        self.project = Some(project);
        self.refresh_tree();
        if let Some(f) = first_form {
            self.open_form(&f);
        } else if let Some(m) = main_file {
            self.open_file(&m, None);
        }
        self.tabs.retain(|t| !matches!(t, Tab::Start));
        self.active = self.active.min(self.tabs.len().saturating_sub(1));
    }

    fn close_project(&mut self) {
        if let Some(b) = &self.build {
            b.kill();
        }
        self.project = None;
        self.tree = None;
        self.tabs = vec![Tab::Start];
        self.active = 0;
        self.diagnostics.clear();
        self.status = self.i18n.tr("status-ready");
    }

    /// Crée un projet depuis l'assistant.
    pub fn create_project(&mut self, kind: ProjectKind, parent: &Path, name: &str) {
        let comments = I18nComments(&self.i18n);
        match vrs_project::create_project(parent, name, kind, &comments) {
            Ok(p) => {
                self.created_here.insert(p.root.clone());
                // Créé par VRS : le code est connu, le projet est approuvé.
                self.settings.trust(&p.root);
                self.settings.projects_dir = Some(parent.to_path_buf());
                let name = p.name().to_owned();
                self.set_project(p);
                self.status = self
                    .i18n
                    .trf("status-project-created", &[("name", name.into())]);
            }
            Err(e) => {
                let msg = self
                    .i18n
                    .trf("error-create-project", &[("error", e.to_string().into())]);
                self.error(msg);
            }
        }
    }

    /// Ajoute une fenêtre au projet.
    pub fn add_form(&mut self, name: &str) {
        let comments = I18nComments(&self.i18n);
        let Some(project) = self.project.as_mut() else {
            return;
        };
        match project.add_form(name, &comments) {
            Ok(_) => {
                self.refresh_tree();
                self.open_form(name);
            }
            Err(e) => {
                let msg = self
                    .i18n
                    .trf("error-codegen", &[("error", e.to_string().into())]);
                self.error(msg);
            }
        }
    }

    pub fn refresh_tree(&mut self) {
        self.tree = self
            .project
            .as_ref()
            .map(|p| vrs_project::file_tree(&p.root));
    }

    // ----- Onglets ---------------------------------------------------------

    fn find_tab(&self, path: &Path) -> Option<usize> {
        self.tabs
            .iter()
            .position(|t| t.path().is_some_and(|p| same_path(p, path)))
    }

    /// Ouvre le concepteur d'une fenêtre.
    pub fn open_form(&mut self, name: &str) {
        let Some(project) = &self.project else { return };
        let path = project.form_path(name);
        if let Some(i) = self.find_tab(&path) {
            self.active = i;
            return;
        }
        match DesignerState::open(&path) {
            Ok(state) => {
                self.tabs.push(Tab::Designer(Box::new(state)));
                self.active = self.tabs.len() - 1;
            }
            Err(e) => {
                let msg = self
                    .i18n
                    .trf("error-read-file", &[("error", e.to_string().into())]);
                self.error(msg);
            }
        }
    }

    /// Ouvre un fichier dans l'éditeur (ou le concepteur pour un .vrsform).
    pub fn open_file(&mut self, path: &Path, line: Option<usize>) {
        if path.extension().and_then(|e| e.to_str()) == Some("vrsform")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            let stem = stem.to_owned();
            self.open_form(&stem);
            return;
        }
        let idx = match self.find_tab(path) {
            Some(i) => i,
            None => match CodeEditor::open(path) {
                Ok(mut e) => {
                    e.diagnostics = self.editor_diagnostics(path);
                    e.completion = self.completion_context_for(path);
                    self.tabs.push(Tab::Code(Box::new(e)));
                    self.tabs.len() - 1
                }
                Err(e) => {
                    let msg = self
                        .i18n
                        .trf("error-read-file", &[("error", e.to_string().into())]);
                    self.error(msg);
                    return;
                }
            },
        };
        self.active = idx;
        if let (Some(l), Some(Tab::Code(e))) = (line, self.tabs.get_mut(idx)) {
            e.go_to_line(l);
        }
    }

    fn close_tab(&mut self, i: usize) {
        if i >= self.tabs.len() {
            return;
        }
        if self.tabs[i].is_dirty() && self.dialogs.unsaved.is_none() {
            self.active = i;
            self.dialogs.unsaved = Some(Command::CloseTab(i));
            return;
        }
        self.tabs.remove(i);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len().saturating_sub(1);
        }
    }

    /// Ferme sans enregistrer (réponse « Ne pas enregistrer »).
    pub fn discard_and_continue(&mut self, then: Command, ctx: &egui::Context) {
        match then {
            Command::CloseTab(i) => {
                if i < self.tabs.len() {
                    self.tabs.remove(i);
                    self.active = self.active.min(self.tabs.len().saturating_sub(1));
                }
            }
            Command::CloseAllTabs => {
                self.tabs.clear();
                self.active = 0;
            }
            Command::CloseProject => self.close_project(),
            Command::Exit => {
                self.exit_confirmed = true;
                self.on_exit();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            other => self.execute(other, ctx),
        }
    }

    // ----- Enregistrement et génération -----------------------------------

    fn save_tab(&mut self, i: usize) -> Result<Option<PathBuf>, String> {
        let format = self.settings.format_on_save;
        let rustfmt = vrs_compiler::find_tool("rustfmt", None);
        match self.tabs.get_mut(i) {
            Some(Tab::Designer(d)) => {
                d.save().map_err(|e| e.to_string())?;
                Ok(Some(d.path.clone()))
            }
            Some(Tab::Code(e)) => {
                if format
                    && e.generated_for.is_none()
                    && e.path.extension().and_then(|x| x.to_str()) == Some("rs")
                    && let Some(rf) = &rustfmt
                    && let Ok(formatted) = vrs_editor::format_with_rustfmt(rf, &e.text)
                {
                    e.set_text(formatted);
                }
                e.save().map_err(|err| err.to_string())?;
                Ok(Some(e.path.clone()))
            }
            _ => Ok(None),
        }
    }

    /// Enregistre l'onglet actif (et régénère le code d'une fenêtre).
    pub fn save_active(&mut self) {
        let i = self.active;
        match self.save_tab(i) {
            Ok(Some(path)) => {
                let file = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                self.status = self.i18n.trf("status-saved", &[("file", file.into())]);
                if matches!(self.tabs.get(i), Some(Tab::Designer(_))) {
                    self.sync_code();
                }
                self.refresh_tree();
                if path
                    .extension()
                    .is_some_and(|e| e == "rs" || e == "vrsform")
                {
                    self.background_check();
                }
            }
            Ok(None) => {}
            Err(e) => {
                let msg = self.i18n.trf("error-save", &[("error", e.into())]);
                self.error(msg);
            }
        }
    }

    /// Enregistre tout et régénère le code. Retourne `false` en cas d'erreur.
    pub fn save_all(&mut self) -> bool {
        for i in 0..self.tabs.len() {
            if !self.tabs[i].is_dirty() {
                continue;
            }
            if let Err(e) = self.save_tab(i) {
                let msg = self.i18n.trf("error-save", &[("error", e.into())]);
                self.error(msg);
                return false;
            }
        }
        let ok = self.sync_code();
        self.refresh_tree();
        ok
    }

    /// Régénère le code des fenêtres et recharge les onglets concernés.
    pub fn sync_code(&mut self) -> bool {
        let Some(project) = &self.project else {
            return true;
        };
        if !project.is_gui() {
            return true;
        }
        let comments = I18nComments(&self.i18n);
        match project.sync_code(&[], &comments) {
            Ok(report) => {
                if !report.written.is_empty() {
                    self.status = self.i18n.trf(
                        "status-code-generated",
                        &[("count", report.written.len().into())],
                    );
                }
                for t in &mut self.tabs {
                    if let Tab::Code(e) = t {
                        e.reload_if_clean();
                    }
                }
                self.refresh_completion_contexts();
                true
            }
            Err(e) => {
                let msg = self
                    .i18n
                    .trf("error-codegen", &[("error", e.to_string().into())]);
                self.error(msg);
                false
            }
        }
    }

    fn format_active(&mut self) {
        let Some(rustfmt) = vrs_compiler::find_tool("rustfmt", None) else {
            return;
        };
        if let Some(Tab::Code(e)) = self.tabs.get_mut(self.active) {
            match vrs_editor::format_with_rustfmt(&rustfmt, &e.text) {
                Ok(t) => e.set_text(t),
                Err(err) => {
                    self.log(LineKind::Stderr, err);
                    self.show_output = true;
                    self.bottom_tab = BottomTab::Output;
                }
            }
        }
    }

    // ----- Concepteur ↔ code -----------------------------------------------

    fn active_form_name(&self) -> Option<String> {
        match self.active_tab()? {
            Tab::Designer(d) => Some(d.doc.name.clone()),
            Tab::Code(e) => {
                let stem = e.path.file_stem()?.to_str()?;
                let module = stem.strip_suffix("_designer").unwrap_or(stem);
                let project = self.project.as_ref()?;
                project
                    .file
                    .forms
                    .iter()
                    .find(|f| vrs_core::ident::to_snake_case(f) == module)
                    .cloned()
            }
            Tab::Start => None,
        }
    }

    fn view_code(&mut self) {
        let Some(form) = self.active_form_name() else {
            return;
        };
        let Some(project) = &self.project else { return };
        let path = vrs_codegen::form_paths(&project.root, &form).user;
        self.open_file(&path, None);
    }

    fn view_designer(&mut self) {
        if let Some(form) = self.active_form_name() {
            self.open_form(&form);
        }
    }

    /// Double-clic sur un contrôle : crée le gestionnaire si nécessaire et l'ouvre.
    pub fn open_handler(&mut self, req: HandlerRequest) {
        let Some(project) = self.project.clone() else {
            return;
        };
        let Some(Tab::Designer(d)) = self.tabs.get_mut(self.active) else {
            return;
        };
        let Some(handler) = d.ensure_event(&req) else {
            return;
        };
        let form = d.doc.name.clone();
        if let Err(e) = d.save() {
            let msg = self
                .i18n
                .trf("error-save", &[("error", e.to_string().into())]);
            self.error(msg);
            return;
        }
        // Le fichier utilisateur ouvert et modifié doit être enregistré avant
        // qu'un gestionnaire y soit inséré.
        let user = vrs_codegen::form_paths(&project.root, &form).user;
        if let Some(i) = self.find_tab(&user)
            && self.tabs[i].is_dirty()
            && let Err(e) = self.save_tab(i)
        {
            let msg = self.i18n.trf("error-save", &[("error", e.into())]);
            self.error(msg);
            return;
        }
        if !self.sync_code() {
            return;
        }
        let line = std::fs::read_to_string(&user)
            .ok()
            .and_then(|s| vrs_codegen::find_fn_line(&s, &handler))
            .map(|l| l + 1);
        self.open_file(&user, line);
        self.refresh_tree();
    }

    // ----- Compilation -----------------------------------------------------

    fn is_trusted(&self, root: &Path) -> bool {
        self.created_here.contains(root) || self.settings.is_trusted(root)
    }

    /// Accepte la confiance demandée et relance l'action.
    pub fn grant_trust(&mut self, req: TrustRequest, ctx: &egui::Context) {
        self.settings.trust(&req.root);
        self.created_here.insert(req.root.clone());
        self.save_settings();
        self.execute(req.then, ctx);
    }

    /// Vérification automatique (`cargo check`) après un enregistrement : les
    /// diagnostics s'affichent dans l'éditeur sans ouvrir la sortie.
    pub fn background_check(&mut self) {
        if !self.settings.check_on_save || self.is_building() {
            return;
        }
        let Some(project) = &self.project else { return };
        let root = project.root.clone();
        if !self.is_trusted(&root) {
            return;
        }
        let Some(cargo) = vrs_compiler::find_cargo(self.settings.build.cargo_path.as_deref())
        else {
            return;
        };
        let ctx = self.repaint_ctx.clone();
        self.build_quiet = true;
        self.pending_diagnostics.clear();
        self.build = Some(vrs_compiler::spawn(
            &cargo,
            &root,
            CargoCommand::Check,
            self.settings.build.release,
            move || {
                if let Some(c) = &ctx {
                    c.request_repaint();
                }
            },
        ));
    }

    fn run_cargo(&mut self, command: CargoCommand) {
        let Some(project) = &self.project else {
            let msg = self.i18n.tr("error-no-project");
            self.error(msg);
            return;
        };
        let root = project.root.clone();
        if self.is_building() && self.build_quiet {
            // Une vérification automatique est remplacée par la commande demandée.
            if let Some(b) = self.build.take() {
                b.kill();
            }
            self.build_quiet = false;
        }
        if self.is_building() {
            let msg = self.i18n.tr("error-already-running");
            self.error(msg);
            return;
        }
        if !self.is_trusted(&root) {
            self.dialogs.trust = Some(TrustRequest {
                root,
                then: Command::Cargo(command),
            });
            return;
        }
        if command != CargoCommand::Clean && !self.save_all() {
            return;
        }
        let Some(cargo) = vrs_compiler::find_cargo(self.settings.build.cargo_path.as_deref())
        else {
            let msg = self.i18n.tr("error-cargo-not-found");
            self.error(msg);
            return;
        };
        if command.emits_diagnostics() {
            self.diagnostics.clear();
            self.selected_diagnostic = None;
            self.apply_diagnostics_to_editors();
        }
        self.build_quiet = false;
        self.console.clear();
        self.show_output = true;
        self.bottom_tab = BottomTab::Output;
        let ctx = self.repaint_ctx.clone();
        self.build = Some(vrs_compiler::spawn(
            &cargo,
            &root,
            command,
            self.settings.build.release,
            move || {
                if let Some(c) = &ctx {
                    c.request_repaint();
                }
            },
        ));
    }

    /// Lit les événements du processus Cargo.
    pub fn poll_build(&mut self) {
        let Some(build) = self.build.as_mut() else {
            return;
        };
        let events = build.poll();
        let mut diag_changed = false;
        if self.build_quiet {
            // Vérification automatique : seuls les diagnostics sont utilisés, et
            // ils remplacent les précédents une fois la vérification terminée.
            for ev in events {
                match ev {
                    BuildEvent::Started(_) => {
                        self.status = self.i18n.tr("status-background-check");
                    }
                    BuildEvent::Diagnostic(d) => {
                        if d.span.is_some() && !self.pending_diagnostics.contains(&d) {
                            self.pending_diagnostics.push(*d);
                        }
                    }
                    BuildEvent::Finished { killed, .. } => {
                        self.build_quiet = false;
                        if !killed {
                            self.diagnostics = std::mem::take(&mut self.pending_diagnostics);
                            self.selected_diagnostic = None;
                            diag_changed = true;
                        }
                        self.status = self.i18n.tr("status-ready");
                    }
                    BuildEvent::Line(..) | BuildEvent::SpawnError(_) => {}
                }
            }
            if diag_changed {
                self.apply_diagnostics_to_editors();
            }
            return;
        }
        for ev in events {
            match ev {
                BuildEvent::Started(cmd) => {
                    self.status = self
                        .i18n
                        .trf("status-running", &[("command", cmd.clone().into())]);
                    self.log(LineKind::Info, format!("> {cmd}"));
                }
                BuildEvent::Line(stream, text) => {
                    let kind = match stream {
                        OutputStream::Stdout => LineKind::Stdout,
                        OutputStream::Stderr => LineKind::Stderr,
                    };
                    self.log(kind, text);
                }
                BuildEvent::Diagnostic(d) => {
                    if !d.rendered.is_empty() {
                        for l in d.rendered.trim_end().lines() {
                            let kind = if d.level == Level::Error {
                                LineKind::Failure
                            } else {
                                LineKind::Stderr
                            };
                            self.log(kind, l.to_owned());
                        }
                    }
                    // Les récapitulatifs sans emplacement (« aborting due to… ») restent dans la sortie.
                    if d.span.is_some() && !self.diagnostics.contains(&d) {
                        self.diagnostics.push(*d);
                        diag_changed = true;
                    }
                }
                BuildEvent::Finished {
                    success,
                    code,
                    duration,
                    killed,
                } => {
                    let (kind, text) = if killed {
                        (LineKind::Failure, self.i18n.tr("status-process-stopped"))
                    } else if success {
                        (LineKind::Success, self.i18n.tr("status-build-succeeded"))
                    } else {
                        let c = code.map(|c| c.to_string()).unwrap_or_else(|| "?".into());
                        (
                            LineKind::Failure,
                            self.i18n.trf("status-build-failed", &[("code", c.into())]),
                        )
                    };
                    self.log(kind, format!("{text} ({:.1} s)", duration.as_secs_f32()));
                    self.status = text;
                    if !success
                        && !killed
                        && self.diagnostics.iter().any(|d| d.level == Level::Error)
                    {
                        self.bottom_tab = BottomTab::Errors;
                    }
                }
                BuildEvent::SpawnError(e) => {
                    self.log(LineKind::Failure, e.clone());
                    let msg = self.i18n.tr("error-cargo-not-found");
                    self.status = msg.clone();
                    self.error(format!("{msg}\n{e}"));
                }
            }
        }
        if diag_changed {
            self.apply_diagnostics_to_editors();
        }
    }

    /// Contexte d'autocomplétion : contrôles de la fenêtre si `path` est le code
    /// d'une fenêtre (`src/forms/<fenêtre>.rs`).
    pub fn completion_context_for(&self, path: &Path) -> vrs_editor::assist::CompletionContext {
        let mut ctx = vrs_editor::assist::CompletionContext::default();
        let Some(project) = &self.project else {
            return ctx;
        };
        for form in &project.file.forms {
            if same_path(&vrs_codegen::form_paths(&project.root, form).user, path) {
                ctx.is_form = true;
                if let Ok(doc) = project.load_form(form) {
                    ctx.controls = doc
                        .controls
                        .iter()
                        .map(|c| (c.name.clone(), c.kind.type_name().to_owned()))
                        .collect();
                }
            }
        }
        ctx
    }

    fn refresh_completion_contexts(&mut self) {
        let updates: Vec<(usize, vrs_editor::assist::CompletionContext)> = self
            .tabs
            .iter()
            .enumerate()
            .filter_map(|(i, t)| match t {
                Tab::Code(e) => Some((i, self.completion_context_for(&e.path))),
                _ => None,
            })
            .collect();
        for (i, c) in updates {
            if let Some(Tab::Code(e)) = self.tabs.get_mut(i) {
                e.completion = c;
            }
        }
    }

    fn editor_diagnostics(&self, path: &Path) -> Vec<EditorDiagnostic> {
        self.diagnostics
            .iter()
            .filter_map(|d| {
                let s = d.span.as_ref()?;
                if !same_path(&s.file, path) {
                    return None;
                }
                let mut message = d.message.clone();
                if let Some(code) = &d.code {
                    message = format!("[{code}] {message}");
                }
                if let Some(key) = d.explanation_key()
                    && let Some(expl) = self.i18n.try_tr(&key, None)
                {
                    message = format!("{message}\n\n{expl}");
                }
                if let Some(label) = &s.label {
                    message = format!("{message}\n\n» {label}");
                }
                // Correction : suggestion du compilateur, sinon équivalence VB.NET
                // pour un nom inconnu (ex. `close()` → `app.exit()`).
                let fix = d
                    .suggestions
                    .iter()
                    .find(|sg| same_path(&sg.span.file, path))
                    .map(|sg| Fix {
                        replacement: sg.replacement.clone(),
                        label_key: "fix-apply-suggestion".into(),
                        range: Some((
                            sg.span.line_start,
                            sg.span.column_start,
                            sg.span.line_end,
                            sg.span.column_end,
                        )),
                    });
                if fix.is_none() && d.code.as_deref() == Some("E0425") {
                    let tabs_ctx = self.tabs.iter().find_map(|t| match t {
                        Tab::Code(e) if same_path(&e.path, path) => Some(e.completion.clone()),
                        _ => None,
                    });
                    if let Some(ctx) = tabs_ctx
                        && let Some(name) = d.message.split('`').nth(1)
                        && let Some(lint) =
                            vrs_editor::assist::lint_vb_habits(&format!("{name}()"), &ctx)
                                .into_iter()
                                .next()
                    {
                        message = format!("{message}\n\n💡 {}", self.i18n.tr(&lint.message_key));
                    }
                }
                Some(EditorDiagnostic {
                    severity: match d.level {
                        Level::Error => Severity::Error,
                        Level::Warning => Severity::Warning,
                        _ => Severity::Info,
                    },
                    line_start: s.line_start,
                    column_start: s.column_start,
                    line_end: s.line_end,
                    column_end: s.column_end,
                    message,
                    fix,
                })
            })
            .collect()
    }

    fn apply_diagnostics_to_editors(&mut self) {
        let updates: Vec<(usize, Vec<EditorDiagnostic>)> = self
            .tabs
            .iter()
            .enumerate()
            .filter_map(|(i, t)| match t {
                Tab::Code(e) => Some((i, self.editor_diagnostics(&e.path))),
                _ => None,
            })
            .collect();
        for (i, d) in updates {
            if let Some(Tab::Code(e)) = self.tabs.get_mut(i) {
                e.diagnostics = d;
            }
        }
    }
}
