//! Choix de la langue de l'interface au démarrage.

use std::path::PathBuf;

use vrs_i18n::{Catalog, FallbackPolicy, I18n, LanguageIdentifier, default_locale, parse_locale};

/// Langue du système, si elle peut être déterminée.
pub fn system_locale() -> Option<LanguageIdentifier> {
    sys_locale::get_locale().and_then(|l| parse_locale(&l))
}

/// Répertoires de traductions externes : `<config>/locales` et `locales/` à côté
/// de l'exécutable (installation portable, développement).
pub fn external_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = vrs_settings::external_locales_dir() {
        dirs.push(d);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        dirs.push(dir.join("locales"));
    }
    dirs
}

/// Catalogue complet : traductions intégrées + fichiers externes.
pub fn load_catalog() -> Catalog {
    let mut catalog = Catalog::embedded();
    for dir in external_dirs() {
        for issue in catalog.load_dir(&dir) {
            log::warn!("traductions : {} : {}", issue.path.display(), issue.message);
        }
    }
    catalog
}

/// Langue demandée : préférence enregistrée, sinon langue du système, sinon en-US.
pub fn requested_language(setting: Option<&str>) -> LanguageIdentifier {
    setting
        .and_then(parse_locale)
        .or_else(system_locale)
        .unwrap_or_else(default_locale)
}

/// Construit le moteur de traduction.
pub fn build_i18n(setting: Option<&str>) -> I18n {
    I18n::new(
        load_catalog(),
        FallbackPolicy::default(),
        requested_language(setting),
    )
}

/// Textes des commentaires insérés dans le code, dans la langue de l'IDE.
pub struct I18nComments<'a>(pub &'a I18n);

impl vrs_codegen::Comments for I18nComments<'_> {
    fn handler_doc(&self, event: &str, owner: &str) -> String {
        self.0.trf(
            "codegen-handler-doc",
            &[("event", event.into()), ("control", owner.into())],
        )
    }
    fn todo(&self) -> String {
        self.0.tr("codegen-todo")
    }
    fn user_file_doc(&self, form: &str) -> String {
        self.0
            .trf("codegen-user-file-doc", &[("form", form.into())])
    }
    fn data_doc(&self) -> String {
        self.0.tr("codegen-data-doc")
    }
}
