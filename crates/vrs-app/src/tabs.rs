//! Onglets de la zone centrale.

use std::path::Path;

use vrs_designer::DesignerState;
use vrs_editor::CodeEditor;
use vrs_i18n::I18n;

/// Un document ouvert.
pub enum Tab {
    /// Page de démarrage.
    Start,
    /// Concepteur d'une fenêtre.
    Designer(Box<DesignerState>),
    /// Fichier texte / code Rust.
    Code(Box<CodeEditor>),
}

impl Tab {
    /// Titre de l'onglet (avec `*` si modifié).
    pub fn title(&self, i18n: &I18n) -> String {
        let (name, dirty) = match self {
            Tab::Start => (i18n.tr("tab-start"), false),
            Tab::Designer(d) => (
                i18n.trf("designer-tab-title", &[("form", d.doc.name.clone().into())]),
                d.is_dirty(),
            ),
            Tab::Code(e) => (e.file_name(), e.is_dirty()),
        };
        if dirty { format!("{name} *") } else { name }
    }

    pub fn is_dirty(&self) -> bool {
        match self {
            Tab::Start => false,
            Tab::Designer(d) => d.is_dirty(),
            Tab::Code(e) => e.is_dirty(),
        }
    }

    /// Fichier associé.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Tab::Start => None,
            Tab::Designer(d) => Some(&d.path),
            Tab::Code(e) => Some(&e.path),
        }
    }
}

/// Compare deux chemins (canonisés si possible).
pub fn same_path(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}
