//! Catalogue des ressources de traduction (intégrées et externes).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

/// Taille maximale d'un fichier `.ftl` externe (protection contre les fichiers aberrants).
const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024;

/// Provenance d'une ressource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceOrigin {
    /// Intégrée à l'exécutable lors de la compilation.
    Embedded,
    /// Chargée depuis un fichier externe.
    External(PathBuf),
}

/// Un fichier `.ftl` d'une langue.
#[derive(Clone, Debug)]
pub struct ResourceFile {
    pub file_name: String,
    pub origin: ResourceOrigin,
    pub source: String,
}

/// Problème rencontré lors du chargement des fichiers externes.
#[derive(Clone, Debug)]
pub struct LoadIssue {
    pub path: PathBuf,
    pub message: String,
}

/// Ensemble des traductions connues, par langue.
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    locales: BTreeMap<String, (LanguageIdentifier, Vec<ResourceFile>)>,
}

impl Catalog {
    /// Catalogue vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Catalogue contenant les traductions intégrées à VRS.
    pub fn embedded() -> Self {
        let mut c = Self::new();
        for (lang, file, source) in super::embedded::EMBEDDED {
            let id: LanguageIdentifier = lang.parse().expect("locale intégrée valide");
            c.add(id, file, ResourceOrigin::Embedded, source.to_string());
        }
        c
    }

    /// Ajoute une ressource.
    pub fn add(
        &mut self,
        lang: LanguageIdentifier,
        file_name: &str,
        origin: ResourceOrigin,
        source: String,
    ) {
        let entry = self
            .locales
            .entry(lang.to_string())
            .or_insert_with(|| (lang.clone(), Vec::new()));
        entry.1.push(ResourceFile {
            file_name: file_name.to_owned(),
            origin,
            source,
        });
    }

    /// Charge un répertoire `locales/<langue>/*.ftl`.
    ///
    /// Les fichiers externes sont ajoutés **après** les fichiers intégrés : leurs
    /// messages remplacent ceux de même identifiant. Une nouvelle langue peut ainsi
    /// être ajoutée par la communauté sans recompiler VRS.
    pub fn load_dir(&mut self, dir: &Path) -> Vec<LoadIssue> {
        let mut issues = Vec::new();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return issues;
        };
        let mut lang_dirs: Vec<_> = entries.filter_map(Result::ok).collect();
        lang_dirs.sort_by_key(|e| e.file_name());
        for lang_dir in lang_dirs {
            let path = lang_dir.path();
            if !path.is_dir() {
                continue;
            }
            let name = lang_dir.file_name().to_string_lossy().to_string();
            let Ok(lang) = name.parse::<LanguageIdentifier>() else {
                issues.push(LoadIssue {
                    path: path.clone(),
                    message: format!("nom de dossier qui n'est pas un code BCP 47 : {name}"),
                });
                continue;
            };
            let Ok(files) = std::fs::read_dir(&path) else {
                continue;
            };
            let mut files: Vec<_> = files.filter_map(Result::ok).collect();
            files.sort_by_key(|e| e.file_name());
            for f in files {
                let fpath = f.path();
                if fpath.extension().and_then(|e| e.to_str()) != Some("ftl") {
                    continue;
                }
                match std::fs::metadata(&fpath) {
                    Ok(m) if m.len() > MAX_FILE_SIZE => {
                        issues.push(LoadIssue {
                            path: fpath.clone(),
                            message: "fichier trop volumineux (ignoré)".into(),
                        });
                        continue;
                    }
                    Err(e) => {
                        issues.push(LoadIssue {
                            path: fpath.clone(),
                            message: e.to_string(),
                        });
                        continue;
                    }
                    _ => {}
                }
                match std::fs::read_to_string(&fpath) {
                    Ok(source) => {
                        let file_name = f.file_name().to_string_lossy().to_string();
                        self.add(
                            lang.clone(),
                            &file_name,
                            ResourceOrigin::External(fpath),
                            source,
                        );
                    }
                    Err(e) => issues.push(LoadIssue {
                        path: fpath,
                        message: e.to_string(),
                    }),
                }
            }
        }
        issues
    }

    /// Langues disponibles, triées.
    pub fn available(&self) -> Vec<LanguageIdentifier> {
        self.locales.values().map(|(id, _)| id.clone()).collect()
    }

    /// Fichiers d'une langue.
    pub fn files(&self, lang: &LanguageIdentifier) -> &[ResourceFile] {
        self.locales
            .get(&lang.to_string())
            .map(|(_, f)| f.as_slice())
            .unwrap_or_default()
    }

    /// Construit un bundle Fluent pour une langue (sans repli).
    pub fn build_bundle(&self, lang: &LanguageIdentifier) -> FluentBundle<FluentResource> {
        let mut bundle = FluentBundle::new(vec![lang.clone()]);
        // Les marques d'isolation Unicode (FSI/PDI) ne sont pas utiles dans l'IDE
        // et s'affichent mal dans certaines polices.
        bundle.set_use_isolating(false);
        for file in self.files(lang) {
            let res = match FluentResource::try_new(file.source.clone()) {
                Ok(r) => r,
                Err((r, errors)) => {
                    log::warn!(
                        "i18n : erreurs de syntaxe dans {lang}/{} : {errors:?}",
                        file.file_name
                    );
                    r
                }
            };
            bundle.add_resource_overriding(res);
        }
        bundle
    }

    /// Nom de la langue dans cette langue (message `language-name`), ex. « Français (Canada) ».
    pub fn display_name(&self, lang: &LanguageIdentifier) -> String {
        let bundle = self.build_bundle(lang);
        bundle
            .get_message("language-name")
            .and_then(|m| m.value())
            .map(|p| {
                let mut errs = Vec::new();
                bundle.format_pattern(p, None, &mut errs).into_owned()
            })
            .unwrap_or_else(|| lang.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_has_initial_languages() {
        let c = Catalog::embedded();
        let langs: Vec<String> = c.available().iter().map(|l| l.to_string()).collect();
        for l in ["en-CA", "en-US", "es-ES", "fr-CA", "fr-FR"] {
            assert!(langs.contains(&l.to_string()), "{l} manquante");
        }
        assert_eq!(
            c.display_name(&"fr-CA".parse().unwrap()),
            "Français (Canada)"
        );
    }

    #[test]
    fn external_dir_adds_language_and_overrides() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("de-DE")).unwrap();
        std::fs::write(
            dir.path().join("de-DE/main.ftl"),
            "language-name = Deutsch\nmenu-file = Datei\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("fr-CA")).unwrap();
        std::fs::write(dir.path().join("fr-CA/extra.ftl"), "menu-file = FICHIER\n").unwrap();
        std::fs::create_dir_all(dir.path().join("pas une langue!")).unwrap();

        let mut c = Catalog::embedded();
        let issues = c.load_dir(dir.path());
        assert_eq!(issues.len(), 1);

        let de = crate::I18n::new(c.clone(), Default::default(), "de-DE".parse().unwrap());
        assert_eq!(de.tr("menu-file"), "Datei");
        // Message absent en allemand : repli sur en-US.
        assert_eq!(de.tr("menu-edit"), "Edit");

        let fr = crate::I18n::new(c, Default::default(), "fr-CA".parse().unwrap());
        assert_eq!(fr.tr("menu-file"), "FICHIER");
    }
}
