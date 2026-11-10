//! Système multilingue de Visual Rust Studio.
//!
//! - Format : [Fluent](https://projectfluent.org/) (`.ftl`). Il gère les paramètres
//!   dynamiques, les pluriels, les variantes et l'Unicode.
//! - Organisation : `locales/<code BCP 47>/<fichier>.ftl` (ex. `locales/fr-CA/main.ftl`).
//! - Les traductions de base sont intégrées à l'exécutable. Des fichiers externes
//!   peuvent compléter ou remplacer des messages sans recompiler VRS.
//! - Les fichiers de traduction ne sont **jamais** exécutés : Fluent est un format de
//!   données, et seules les fonctions intégrées (NUMBER…) sont disponibles.
//! - La chaîne de repli est prévisible : locale exacte → même langue → `en-US` → clé.
//!
//! Les clés suivent le style `menu-file`, `button-save`, `designer-add-window`
//! (Fluent n'autorise pas le point dans les identifiants).

mod catalog;
mod embedded;
mod negotiate;
pub mod validate;

use std::cell::RefCell;
use std::collections::HashSet;

use fluent_bundle::{FluentBundle, FluentResource};

pub use catalog::{Catalog, LoadIssue, ResourceOrigin};
pub use fluent_bundle::{FluentArgs, FluentValue};
pub use negotiate::{FallbackPolicy, fallback_chain};
pub use unic_langid::LanguageIdentifier;

/// Locale de référence : toujours complète, utilisée en dernier recours.
pub const DEFAULT_LOCALE: &str = "en-US";

/// Analyse un code de langue BCP 47 (`fr-CA`, `es_MX`, `en-US.UTF-8`…).
pub fn parse_locale(code: &str) -> Option<LanguageIdentifier> {
    // Les locales système POSIX ressemblent à `fr_CA.UTF-8@euro`.
    let cleaned = code
        .split(['.', '@'])
        .next()
        .unwrap_or_default()
        .replace('_', "-");
    if cleaned.is_empty() || cleaned == "C" || cleaned == "POSIX" {
        return None;
    }
    cleaned.parse().ok()
}

/// Moteur de traduction de l'IDE.
pub struct I18n {
    catalog: Catalog,
    policy: FallbackPolicy,
    requested: LanguageIdentifier,
    chain: Vec<LanguageIdentifier>,
    bundles: Vec<FluentBundle<FluentResource>>,
    missing: RefCell<HashSet<String>>,
}

impl std::fmt::Debug for I18n {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18n")
            .field("requested", &self.requested)
            .field("chain", &self.chain)
            .finish_non_exhaustive()
    }
}

impl I18n {
    /// Crée le moteur pour la langue demandée avec le catalogue et la politique donnés.
    pub fn new(catalog: Catalog, policy: FallbackPolicy, requested: LanguageIdentifier) -> Self {
        let mut this = Self {
            catalog,
            policy,
            requested: requested.clone(),
            chain: Vec::new(),
            bundles: Vec::new(),
            missing: RefCell::new(HashSet::new()),
        };
        this.set_language(requested);
        this
    }

    /// Raccourci : catalogue intégré, politique par défaut.
    pub fn with_embedded(requested: &str) -> Self {
        let lang = parse_locale(requested).unwrap_or_else(default_locale);
        Self::new(Catalog::embedded(), FallbackPolicy::default(), lang)
    }

    /// Change la langue à chaud. Les textes suivants utilisent la nouvelle langue.
    pub fn set_language(&mut self, requested: LanguageIdentifier) {
        let available = self.catalog.available();
        self.chain = fallback_chain(&requested, &available, &self.policy);
        self.bundles = self
            .chain
            .iter()
            .map(|lang| self.catalog.build_bundle(lang))
            .collect();
        self.requested = requested;
        self.missing.borrow_mut().clear();
    }

    /// Langue demandée par l'utilisateur.
    pub fn requested(&self) -> &LanguageIdentifier {
        &self.requested
    }

    /// Langue effectivement utilisée en premier.
    pub fn active(&self) -> &LanguageIdentifier {
        &self.chain[0]
    }

    /// Chaîne de repli complète (pour diagnostic).
    pub fn chain(&self) -> &[LanguageIdentifier] {
        &self.chain
    }

    /// Catalogue chargé.
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Recharge le catalogue (ex. après l'ajout de fichiers externes).
    pub fn reload(&mut self, catalog: Catalog) {
        self.catalog = catalog;
        let lang = self.requested.clone();
        self.set_language(lang);
    }

    /// Traduit un message sans paramètre.
    pub fn tr(&self, id: &str) -> String {
        self.format(id, None)
    }

    /// Traduit un message avec paramètres.
    ///
    /// ```
    /// # use vrs_i18n::{I18n, FluentArgs};
    /// let i18n = I18n::with_embedded("fr-CA");
    /// let mut args = FluentArgs::new();
    /// args.set("count", 3);
    /// assert!(i18n.tr_args("status-errors", &args).contains('3'));
    /// ```
    pub fn tr_args(&self, id: &str, args: &FluentArgs) -> String {
        self.format(id, Some(args))
    }

    /// Variante pratique : `i18n.trf("project-created", &[("name", "MonApp".into())])`.
    pub fn trf(&self, id: &str, args: &[(&str, FluentValue)]) -> String {
        let mut fa = FluentArgs::new();
        for (k, v) in args {
            fa.set(*k, v.clone());
        }
        self.format(id, Some(&fa))
    }

    /// Le message existe-t-il dans au moins une langue de la chaîne ?
    pub fn has(&self, id: &str) -> bool {
        self.bundles.iter().any(|b| b.has_message(id))
    }

    /// Traduit si la clé existe, sinon `None` (aucune journalisation).
    pub fn try_tr(&self, id: &str, args: Option<&FluentArgs>) -> Option<String> {
        self.bundles.iter().find_map(|b| format_in(b, id, args))
    }

    /// Clés manquantes rencontrées depuis le dernier changement de langue.
    pub fn missing_keys(&self) -> Vec<String> {
        let mut v: Vec<_> = self.missing.borrow().iter().cloned().collect();
        v.sort();
        v
    }

    fn format(&self, id: &str, args: Option<&FluentArgs>) -> String {
        if let Some(text) = self.try_tr(id, args) {
            return text;
        }
        if self.missing.borrow_mut().insert(id.to_owned()) {
            log::warn!("i18n : clé absente dans {:?} : {id}", self.chain);
        }
        // Ne jamais afficher une chaîne vide : la clé reste lisible pour les développeurs.
        id.to_owned()
    }
}

fn format_in(
    bundle: &FluentBundle<FluentResource>,
    id: &str,
    args: Option<&FluentArgs>,
) -> Option<String> {
    let msg = bundle.get_message(id)?;
    let pattern = msg.value()?;
    let mut errors = Vec::new();
    let text = bundle.format_pattern(pattern, args, &mut errors);
    if !errors.is_empty() {
        log::warn!("i18n : erreurs de formatage pour {id} : {errors:?}");
    }
    let text = text.into_owned();
    (!text.trim().is_empty()).then_some(text)
}

/// Retourne `en-US`.
pub fn default_locale() -> LanguageIdentifier {
    DEFAULT_LOCALE.parse().expect("en-US est un code valide")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn french_canadian_menu() {
        let i18n = I18n::with_embedded("fr-CA");
        assert_eq!(i18n.tr("menu-file"), "Fichier");
        assert_eq!(i18n.tr("menu-edit"), "Édition");
        assert_eq!(i18n.tr("button-save"), "Enregistrer");
    }

    #[test]
    fn english_menu() {
        let i18n = I18n::with_embedded("en-US");
        assert_eq!(i18n.tr("menu-file"), "File");
        assert_eq!(i18n.tr("designer-add-window"), "Add Window");
    }

    #[test]
    fn hot_switch() {
        let mut i18n = I18n::with_embedded("en-US");
        assert_eq!(i18n.tr("menu-build"), "Build");
        i18n.set_language("fr-CA".parse().unwrap());
        assert_eq!(i18n.tr("menu-build"), "Compiler");
    }

    #[test]
    fn missing_key_returns_id_and_is_logged() {
        let i18n = I18n::with_embedded("fr-CA");
        assert_eq!(i18n.tr("does-not-exist"), "does-not-exist");
        assert_eq!(i18n.missing_keys(), vec!["does-not-exist".to_string()]);
    }

    #[test]
    fn plurals_and_args() {
        let i18n = I18n::with_embedded("fr-CA");
        let one = i18n.trf("status-errors", &[("count", 1.into())]);
        let many = i18n.trf("status-errors", &[("count", 5.into())]);
        assert_eq!(one, "1 erreur");
        assert_eq!(many, "5 erreurs");
        let en = I18n::with_embedded("en-US");
        assert_eq!(en.trf("status-errors", &[("count", 1.into())]), "1 error");
        assert_eq!(en.trf("status-errors", &[("count", 0.into())]), "0 errors");
    }

    #[test]
    fn partial_locale_falls_back() {
        // en-CA ne redéfinit que quelques messages : le reste vient de en-US.
        let i18n = I18n::with_embedded("en-CA");
        assert_eq!(i18n.active().to_string(), "en-CA");
        assert_eq!(i18n.tr("menu-file"), "File");
        assert!(i18n.missing_keys().is_empty());
    }

    #[test]
    fn posix_locale_parsing() {
        assert_eq!(parse_locale("fr_CA.UTF-8").unwrap().to_string(), "fr-CA");
        assert_eq!(parse_locale("es-MX").unwrap().to_string(), "es-MX");
        assert!(parse_locale("C").is_none());
    }

    #[test]
    fn no_isolation_marks() {
        let i18n = I18n::with_embedded("en-US");
        let s = i18n.trf("status-errors", &[("count", 2.into())]);
        assert!(!s.contains('\u{2068}'));
    }
}
