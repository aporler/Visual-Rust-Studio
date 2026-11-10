//! État global de l'IDE et boucle d'affichage.

use std::collections::HashSet;
use std::path::PathBuf;

use vrs_compiler::{BuildHandle, Diagnostic};
use vrs_components::Control;
use vrs_i18n::I18n;
use vrs_project::{FileNode, Project};
use vrs_settings::{Settings, Theme};

use crate::commands::Command;
use crate::tabs::Tab;

/// Onglet du panneau inférieur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BottomTab {
    Output,
    Errors,
    Debug,
}

/// Ligne de la console de sortie.
#[derive(Clone, Debug)]
pub struct ConsoleLine {
    pub text: String,
    pub kind: LineKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Info,
    Stdout,
    Stderr,
    Success,
    Failure,
}

/// Assistant « Nouveau projet ».
#[derive(Clone, Debug)]
pub struct NewProjectDialog {
    pub template: usize,
    pub name: String,
    pub location: String,
}

/// Paramètres en cours d'édition (pour pouvoir annuler).
#[derive(Clone, Debug)]
pub struct SettingsDialog {
    pub original: Settings,
    pub cargo_path: String,
    pub debugger_path: String,
}

/// Action en attente de confirmation (projet non approuvé).
#[derive(Clone, Debug)]
pub struct TrustRequest {
    pub root: PathBuf,
    pub then: Command,
}

/// Fenêtres de dialogue ouvertes.
#[derive(Default)]
pub struct Dialogs {
    pub new_project: Option<NewProjectDialog>,
    pub settings: Option<SettingsDialog>,
    pub add_form: Option<String>,
    /// Projets d'exemple : (exemple sélectionné, emplacement).
    pub examples: Option<(usize, String)>,
    /// Renommage d'une fenêtre : (ancien nom, nouveau nom).
    pub rename_form: Option<(String, String)>,
    pub trust: Option<TrustRequest>,
    pub error: Option<String>,
    pub about: bool,
    /// Commande à exécuter après la question « enregistrer les modifications ? ».
    pub unsaved: Option<Command>,
}

/// L'IDE.
pub struct VrsApp {
    pub settings: Settings,
    pub settings_path: Option<PathBuf>,
    pub i18n: I18n,
    pub project: Option<Project>,
    pub tree: Option<FileNode>,
    pub tabs: Vec<Tab>,
    pub active: usize,
    pub build: Option<BuildHandle>,
    /// rust-analyzer (diagnostics en direct, autocomplétion, définitions).
    pub lsp: Option<vrs_lsp::LspClient>,
    pub lsp_pending: Vec<(i64, crate::lsp::Pending)>,
    pub lsp_generation: u64,
    pub lsp_synced: std::collections::HashSet<PathBuf>,
    /// Session de débogage en cours.
    pub debug: Option<crate::debug::DebugState>,
    /// Lancer le débogueur à la fin de la compilation en cours.
    pub debug_after_build: bool,
    /// Points d'arrêt par fichier (lignes 1-based).
    pub breakpoints: std::collections::BTreeMap<PathBuf, std::collections::BTreeSet<usize>>,
    pub watches: Vec<crate::debug::Watch>,
    pub watch_input: String,
    /// La commande en cours est une vérification automatique (silencieuse).
    pub build_quiet: bool,
    /// Diagnostics reçus pendant une vérification automatique.
    pub pending_diagnostics: Vec<Diagnostic>,
    pub console: Vec<ConsoleLine>,
    pub diagnostics: Vec<Diagnostic>,
    pub selected_diagnostic: Option<usize>,
    pub show_errors: bool,
    pub show_warnings: bool,
    pub bottom_tab: BottomTab,
    pub show_explorer: bool,
    pub show_properties: bool,
    pub show_output: bool,
    pub dialogs: Dialogs,
    pub clipboard: Vec<Control>,
    pub status: String,
    pub cursor: Option<(usize, usize)>,
    /// Projets créés pendant cette session (approuvés automatiquement).
    pub created_here: HashSet<PathBuf>,
    pub commands: Vec<Command>,
    pub exit_confirmed: bool,
    /// Contexte egui, pour redessiner depuis le fil de compilation.
    pub repaint_ctx: Option<egui::Context>,
    last_title: String,
    applied_theme: Option<(Theme, u32)>,
}

impl VrsApp {
    /// Charge les paramètres et prépare l'IDE.
    pub fn load() -> Self {
        let settings_path = vrs_settings::settings_path();
        let mut errors = Vec::new();
        let settings = match &settings_path {
            Some(p) => Settings::load_from(p).unwrap_or_else(|e| {
                errors.push(e);
                Settings::default()
            }),
            None => Settings::default(),
        };
        let mut app = Self::with_settings(settings, settings_path);
        if let Some(e) = errors.pop() {
            app.dialogs.error = Some(e);
        }
        app
    }

    /// Crée l'IDE avec des paramètres donnés (tests).
    pub fn with_settings(settings: Settings, settings_path: Option<PathBuf>) -> Self {
        let i18n = crate::lang::build_i18n(settings.language.as_deref());
        let status = i18n.tr("status-ready");
        Self {
            settings,
            settings_path,
            i18n,
            project: None,
            tree: None,
            tabs: vec![Tab::Start],
            active: 0,
            build: None,
            lsp: None,
            lsp_pending: Vec::new(),
            lsp_generation: u64::MAX,
            lsp_synced: Default::default(),
            debug: None,
            debug_after_build: false,
            breakpoints: Default::default(),
            watches: Vec::new(),
            watch_input: String::new(),
            build_quiet: false,
            pending_diagnostics: Vec::new(),
            console: Vec::new(),
            diagnostics: Vec::new(),
            selected_diagnostic: None,
            show_errors: true,
            show_warnings: true,
            bottom_tab: BottomTab::Output,
            show_explorer: true,
            show_properties: true,
            show_output: true,
            dialogs: Dialogs::default(),
            clipboard: Vec::new(),
            status,
            cursor: None,
            created_here: HashSet::new(),
            commands: Vec::new(),
            exit_confirmed: false,
            repaint_ctx: None,
            last_title: String::new(),
            applied_theme: None,
        }
    }

    /// Applique thème et zoom.
    pub fn apply_style(&self, ctx: &egui::Context) {
        ctx.set_theme(match self.settings.theme {
            Theme::System => egui::ThemePreference::System,
            Theme::Light => egui::ThemePreference::Light,
            Theme::Dark => egui::ThemePreference::Dark,
        });
        ctx.set_zoom_factor(self.settings.ui_scale);
    }

    /// Polices : la police à chasse fixe (Hack) sert de repli à la police
    /// proportionnelle, pour les symboles absents de celle-ci (flèches « → »
    /// utilisées dans les traductions, par exemple).
    pub fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        let mono = fonts
            .families
            .get(&egui::FontFamily::Monospace)
            .and_then(|f| f.first().cloned());
        if let (Some(mono), Some(prop)) = (
            mono,
            fonts.families.get_mut(&egui::FontFamily::Proportional),
        ) && !prop.contains(&mono)
        {
            prop.push(mono);
        }
        ctx.set_fonts(fonts);
        // Images (PictureBox) : chargeurs de fichiers et de formats d'image.
        egui_extras::install_image_loaders(ctx);
    }

    /// Enregistre les paramètres.
    pub fn save_settings(&mut self) {
        if let Some(p) = &self.settings_path
            && let Err(e) = self.settings.save_to(p)
        {
            log::error!("paramètres : {e}");
        }
    }

    /// Ajoute une commande à exécuter après l'image.
    pub fn cmd(&mut self, c: Command) {
        self.commands.push(c);
    }

    /// Message dans la console.
    pub fn log(&mut self, kind: LineKind, text: impl Into<String>) {
        self.console.push(ConsoleLine {
            text: text.into(),
            kind,
        });
        const MAX: usize = 20_000;
        if self.console.len() > MAX {
            self.console.drain(..self.console.len() - MAX);
        }
    }

    /// Affiche une erreur dans une boîte de dialogue.
    pub fn error(&mut self, text: String) {
        log::warn!("{text}");
        self.dialogs.error = Some(text);
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }

    /// Concepteur actif, s'il y en a un.
    pub fn active_designer(&mut self) -> Option<&mut vrs_designer::DesignerState> {
        match self.tabs.get_mut(self.active) {
            Some(Tab::Designer(d)) => Some(d),
            _ => None,
        }
    }

    pub fn has_unsaved(&self) -> bool {
        self.tabs.iter().any(Tab::is_dirty)
    }

    pub fn is_building(&self) -> bool {
        self.build.as_ref().is_some_and(|b| !b.is_finished())
    }

    /// Une image complète de l'IDE (utilisée aussi par les tests sans affichage).
    pub fn frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        if self.repaint_ctx.is_none() {
            self.repaint_ctx = Some(ctx.clone());
            Self::setup_fonts(&ctx);
        }
        let style_key = (self.settings.theme, (self.settings.ui_scale * 100.0) as u32);
        if self.applied_theme != Some(style_key) {
            self.apply_style(&ctx);
            self.applied_theme = Some(style_key);
        }
        self.poll_build();
        self.poll_lsp();
        self.poll_debug();
        self.update_title(&ctx);
        self.handle_close_request(&ctx);
        crate::ui::shortcuts(self, &ctx);

        egui::Panel::top("vrs-menu").show(ui, |ui| crate::ui::menus::menu_bar(self, ui));
        egui::Panel::top("vrs-toolbar").show(ui, |ui| crate::ui::toolbar::toolbar(self, ui));
        egui::Panel::bottom("vrs-status").show(ui, |ui| crate::ui::status_bar(self, ui));
        if self.show_output {
            egui::Panel::bottom("vrs-output")
                .resizable(true)
                .default_size(200.0)
                .min_size(80.0)
                .show(ui, |ui| crate::ui::bottom::bottom_panel(self, ui));
        }
        if self.show_explorer {
            egui::Panel::left("vrs-explorer")
                .resizable(true)
                .default_size(240.0)
                .min_size(140.0)
                .show(ui, |ui| crate::ui::explorer::explorer(self, ui));
        }
        if self.show_properties {
            egui::Panel::right("vrs-properties")
                .resizable(true)
                .default_size(300.0)
                .min_size(200.0)
                .show(ui, |ui| crate::ui::side::properties_panel(self, ui));
        }
        egui::CentralPanel::default().show(ui, |ui| crate::ui::central::central(self, ui));

        crate::ui::dialogs::dialogs(self, &ctx);

        let commands = std::mem::take(&mut self.commands);
        for c in commands {
            self.execute(c, &ctx);
        }
        if self.is_building() || self.is_debugging() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let title = match &self.project {
            Some(p) => self.i18n.trf(
                "app-title-project",
                &[("project", p.name().to_owned().into())],
            ),
            None => self.i18n.tr("app-title"),
        };
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }

    fn handle_close_request(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.exit_confirmed {
            if self.has_unsaved() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.dialogs.unsaved = Some(Command::Exit);
            } else {
                self.on_exit();
            }
        }
    }

    /// Nettoyage avant de quitter.
    pub fn on_exit(&mut self) {
        if let Some(b) = &self.build {
            b.kill();
        }
        self.stop_lsp();
        if let Some(d) = self.debug.take() {
            d.session.terminate();
        }
        self.save_settings();
    }
}

impl eframe::App for VrsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame(ui);
    }
}
