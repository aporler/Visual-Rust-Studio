//! Export des projets d'exemple (dossier `exemples/` du dépôt).

use std::path::{Path, PathBuf};

use vrs_i18n::I18n;

use crate::lang::I18nComments;

/// Langues exportées dans `exemples/`.
pub const EXPORT_LANGUAGES: [&str; 2] = ["en-US", "fr-CA"];

/// Génère tous les exemples dans `dir/<langue>/<Projet>/`.
pub fn export_examples(dir: &Path, lang: &str) -> Result<Vec<PathBuf>, String> {
    let i18n = I18n::with_embedded(lang);
    let comments = I18nComments(&i18n);
    let tr = |k: &str| i18n.tr(k);
    let target = dir.join(lang);
    std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for ex in vrs_project::examples::EXAMPLES {
        let existing = target.join(ex.project_name);
        if existing.exists() {
            std::fs::remove_dir_all(&existing).map_err(|e| e.to_string())?;
        }
        let p = vrs_project::examples::create_example(ex.id, &target, &tr, &comments)
            .map_err(|e| format!("{}: {e}", ex.id))?;
        out.push(p.root);
    }
    Ok(out)
}
