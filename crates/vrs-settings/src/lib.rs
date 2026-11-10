//! Paramètres de Visual Rust Studio.
//!
//! Les paramètres sont stockés en TOML dans le répertoire de configuration
//! propre à chaque système :
//! - Linux : `~/.config/visualruststudio/settings.toml`
//! - macOS : `~/Library/Application Support/ca.VRS.VisualRustStudio/settings.toml`
//! - Windows : `%APPDATA%\VRS\VisualRustStudio\config\settings.toml`
//!
//! La variable d'environnement `VRS_CONFIG_DIR` permet d'utiliser un autre
//! répertoire (mode portable, tests). Aucune télémétrie n'est collectée.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Nom du fichier de paramètres.
pub const SETTINGS_FILE: &str = "settings.toml";

/// Thème de l'interface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

/// Préférences du concepteur visuel.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesignerSettings {
    pub grid_size: f32,
    pub snap_to_grid: bool,
    pub show_grid: bool,
}

impl Default for DesignerSettings {
    fn default() -> Self {
        Self {
            grid_size: 8.0,
            snap_to_grid: true,
            show_grid: true,
        }
    }
}

/// Paramètres de compilation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BuildSettings {
    /// Compiler en mode `--release`.
    pub release: bool,
    /// Chemin explicite vers `cargo` (sinon détection automatique).
    pub cargo_path: Option<PathBuf>,
}

/// Tous les paramètres de l'IDE.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Langue de l'interface (BCP 47). `None` : langue du système.
    pub language: Option<String>,
    pub theme: Theme,
    /// Facteur de zoom de l'interface.
    pub ui_scale: f32,
    pub editor_font_size: f32,
    pub tab_width: usize,
    pub format_on_save: bool,
    /// Lancer `cargo check` en arrière-plan à chaque enregistrement.
    pub check_on_save: bool,
    /// Utiliser rust-analyzer (diagnostics en direct, autocomplétion, définitions).
    pub lsp_enabled: bool,
    pub designer: DesignerSettings,
    pub build: BuildSettings,
    /// Répertoire par défaut des nouveaux projets.
    pub projects_dir: Option<PathBuf>,
    /// Projets récents (le plus récent en premier).
    pub recent_projects: Vec<PathBuf>,
    /// Projets approuvés par l'utilisateur (compilation autorisée sans confirmation).
    pub trusted_projects: Vec<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: None,
            theme: Theme::System,
            ui_scale: 1.0,
            editor_font_size: 14.0,
            tab_width: 4,
            format_on_save: true,
            check_on_save: true,
            lsp_enabled: true,
            designer: DesignerSettings::default(),
            build: BuildSettings::default(),
            projects_dir: None,
            recent_projects: Vec::new(),
            trusted_projects: Vec::new(),
        }
    }
}

/// Nombre maximal de projets récents mémorisés.
pub const MAX_RECENT: usize = 10;

impl Settings {
    /// Charge les paramètres depuis un fichier. Un fichier absent donne les valeurs
    /// par défaut ; un fichier invalide est signalé et remplacé par les valeurs par défaut.
    pub fn load_from(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str::<Settings>(&text)
                .map(|s| s.sanitized())
                .map_err(|e| format!("{}: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("{}: {e}", path.display())),
        }
    }

    /// Enregistre les paramètres (écriture atomique).
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        let header = "# Paramètres de Visual Rust Studio — modifiable à la main.\n\n";
        vrs_core::fs::atomic_write(path, format!("{header}{text}"))
    }

    /// Corrige les valeurs hors limites (fichier modifié à la main).
    pub fn sanitized(mut self) -> Self {
        self.ui_scale = self.ui_scale.clamp(0.5, 3.0);
        self.editor_font_size = self.editor_font_size.clamp(8.0, 48.0);
        self.tab_width = self.tab_width.clamp(1, 16);
        self.designer.grid_size = self.designer.grid_size.clamp(2.0, 64.0);
        self.recent_projects.truncate(MAX_RECENT);
        self
    }

    /// Ajoute un projet en tête de la liste des projets récents.
    pub fn push_recent(&mut self, path: &Path) {
        self.recent_projects.retain(|p| p != path);
        self.recent_projects.insert(0, path.to_path_buf());
        self.recent_projects.truncate(MAX_RECENT);
    }

    /// Le projet est-il approuvé ?
    pub fn is_trusted(&self, root: &Path) -> bool {
        let root = normalize(root);
        self.trusted_projects.iter().any(|p| normalize(p) == root)
    }

    /// Approuve un projet.
    pub fn trust(&mut self, root: &Path) {
        if !self.is_trusted(root) {
            self.trusted_projects.push(normalize(root));
        }
    }
}

fn normalize(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

/// Répertoire de configuration de VRS.
pub fn config_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("VRS_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    directories::ProjectDirs::from("ca", "VRS", "VisualRustStudio")
        .map(|d| d.config_dir().to_path_buf())
}

/// Chemin du fichier de paramètres.
pub fn settings_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join(SETTINGS_FILE))
}

/// Répertoire des traductions externes (`<config>/locales`).
pub fn external_locales_dir() -> Option<PathBuf> {
    config_dir().map(|d| d.join("locales"))
}

/// Répertoire proposé par défaut pour les nouveaux projets (`~/VRSProjects`).
pub fn default_projects_dir() -> PathBuf {
    directories::UserDirs::new()
        .map(|u| u.home_dir().join("VRSProjects"))
        .unwrap_or_else(|| PathBuf::from("VRSProjects"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        let mut s = Settings {
            language: Some("fr-CA".into()),
            theme: Theme::Dark,
            ..Default::default()
        };
        s.designer.grid_size = 10.0;
        s.push_recent(Path::new("/tmp/a"));
        s.save_to(&path).unwrap();
        let loaded = Settings::load_from(&path).unwrap();
        assert_eq!(s, loaded);
    }

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let s = Settings::load_from(&dir.path().join("absent.toml")).unwrap();
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn partial_and_out_of_range() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        std::fs::write(&path, "language = \"es-ES\"\nui_scale = 99.0\n").unwrap();
        let s = Settings::load_from(&path).unwrap();
        assert_eq!(s.language.as_deref(), Some("es-ES"));
        assert_eq!(s.ui_scale, 3.0);
        assert_eq!(s.tab_width, 4);
    }

    #[test]
    fn invalid_file_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        std::fs::write(&path, "language = [").unwrap();
        assert!(Settings::load_from(&path).is_err());
    }

    #[test]
    fn recent_and_trust() {
        let mut s = Settings::default();
        for i in 0..15 {
            s.push_recent(Path::new(&format!("/p/{i}")));
        }
        s.push_recent(Path::new("/p/3"));
        assert_eq!(s.recent_projects.len(), MAX_RECENT);
        assert_eq!(s.recent_projects[0], PathBuf::from("/p/3"));
        let dir = tempfile::tempdir().unwrap();
        assert!(!s.is_trusted(dir.path()));
        s.trust(dir.path());
        assert!(s.is_trusted(dir.path()));
    }
}
