//! Catalogue des projets d'exemple, téléchargé à la demande.
//!
//! Les exemples ne sont pas intégrés à l'IDE : ils se trouvent dans le dossier
//! `exemples/` du dépôt, décrit par `exemples/index.toml`. L'IDE lit ce
//! catalogue puis télécharge uniquement les fichiers de l'exemple choisi, dans
//! la langue de l'interface. On peut donc ajouter des exemples sans recompiler.
//!
//! La source peut être une adresse web (`https://…`) ou un dossier local.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;

/// Catalogue officiel (dépôt GitHub de Visual Rust Studio).
pub const DEFAULT_CATALOG: &str =
    "https://raw.githubusercontent.com/aporler/Visual-Rust-Studio/main/exemples";

/// Version du format de `index.toml` comprise par cet IDE.
pub const CATALOG_VERSION: u32 = 1;

/// Taille maximale d'un fichier d'exemple.
const MAX_FILE: u64 = 10 * 1024 * 1024;
/// Nombre maximal de fichiers par exemple.
const MAX_FILES: usize = 500;

/// Catalogue (`index.toml`).
#[derive(Clone, Debug, Deserialize)]
pub struct Catalog {
    pub version: u32,
    /// Langues disponibles (dossiers `exemples/<langue>/`).
    pub languages: Vec<String>,
    #[serde(rename = "example", default)]
    pub examples: Vec<CatalogExample>,
}

/// Un exemple du catalogue.
#[derive(Clone, Debug, Deserialize)]
pub struct CatalogExample {
    pub id: String,
    /// `beginner`, `intermediate` ou `advanced`.
    pub level: String,
    /// Dossier du projet dans chaque dossier de langue.
    pub folder: String,
    /// Titre par langue.
    pub title: BTreeMap<String, String>,
    /// Description par langue.
    pub description: BTreeMap<String, String>,
    /// Fichiers du projet (chemins relatifs, séparateur `/`).
    pub files: Vec<String>,
}

/// Langue du catalogue la plus proche de `wanted` : exacte, même langue, en-US.
pub fn choose_language(available: &[String], wanted: &str) -> Option<String> {
    let base = wanted.split('-').next().unwrap_or(wanted);
    available
        .iter()
        .find(|l| l.as_str() == wanted)
        .or_else(|| available.iter().find(|l| l.split('-').next() == Some(base)))
        .or_else(|| available.iter().find(|l| l.as_str() == "en-US"))
        .or_else(|| available.first())
        .cloned()
}

impl CatalogExample {
    /// Titre dans la langue la plus proche.
    pub fn title_in(&self, lang: &str) -> String {
        Self::text(&self.title, lang)
    }

    /// Description dans la langue la plus proche.
    pub fn description_in(&self, lang: &str) -> String {
        Self::text(&self.description, lang)
    }

    fn text(map: &BTreeMap<String, String>, lang: &str) -> String {
        let keys: Vec<String> = map.keys().cloned().collect();
        choose_language(&keys, lang)
            .and_then(|k| map.get(&k).cloned())
            .unwrap_or_default()
    }
}

/// Chemin relatif sûr : pas de `..`, pas de chemin absolu.
fn safe_relative(path: &str) -> Option<PathBuf> {
    let p = Path::new(path);
    if path.is_empty() || p.is_absolute() {
        return None;
    }
    p.components()
        .all(|c| matches!(c, Component::Normal(_)))
        .then(|| p.to_path_buf())
}

/// Lit un fichier de la source (web ou dossier local).
fn fetch(source: &str, relative: &str) -> Result<Vec<u8>, String> {
    let source = source.trim_end_matches('/');
    if source.starts_with("http://") || source.starts_with("https://") {
        let url = format!(
            "{source}/{}",
            relative
                .split('/')
                .map(urlencode)
                .collect::<Vec<_>>()
                .join("/")
        );
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        let mut response = agent.get(&url).call().map_err(|e| format!("{url} : {e}"))?;
        response
            .body_mut()
            .with_config()
            .limit(MAX_FILE)
            .read_to_vec()
            .map_err(|e| format!("{url} : {e}"))
    } else {
        let root = PathBuf::from(source.strip_prefix("file://").unwrap_or(source));
        let path = root.join(relative);
        let meta = std::fs::metadata(&path).map_err(|e| format!("{} : {e}", path.display()))?;
        if meta.len() > MAX_FILE {
            return Err(format!("{} : fichier trop volumineux", path.display()));
        }
        std::fs::read(&path).map_err(|e| format!("{} : {e}", path.display()))
    }
}

fn urlencode(segment: &str) -> String {
    segment
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Télécharge et valide le catalogue.
pub fn load_catalog(source: &str) -> Result<Catalog, String> {
    let bytes = fetch(source, "index.toml")?;
    let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    let catalog: Catalog = toml::from_str(&text).map_err(|e| format!("index.toml : {e}"))?;
    if catalog.version > CATALOG_VERSION {
        return Err(format!(
            "index.toml : version {} (cet IDE comprend la version {CATALOG_VERSION} ; mettez VRS à jour)",
            catalog.version
        ));
    }
    for ex in &catalog.examples {
        if safe_relative(&ex.folder).is_none() || ex.files.len() > MAX_FILES {
            return Err(format!("index.toml : exemple « {} » invalide", ex.id));
        }
        if let Some(bad) = ex.files.iter().find(|f| safe_relative(f).is_none()) {
            return Err(format!("index.toml : chemin refusé « {bad} »"));
        }
    }
    Ok(catalog)
}

/// Télécharge un exemple dans `parent/<dossier>` (nom libre choisi
/// automatiquement) et retourne le dossier du projet créé.
pub fn download_example(
    source: &str,
    catalog: &Catalog,
    example: &CatalogExample,
    lang: &str,
    parent: &Path,
) -> Result<PathBuf, String> {
    let lang = choose_language(&catalog.languages, lang).ok_or("index.toml : aucune langue")?;
    let folder = safe_relative(&example.folder).ok_or("dossier invalide")?;
    let base_name = folder
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| example.id.clone());
    let name = (1..)
        .map(|i| {
            if i == 1 {
                base_name.clone()
            } else {
                format!("{base_name}{i}")
            }
        })
        .find(|n| !parent.join(n).exists())
        .expect("itérateur infini");
    let dest = parent.join(&name);
    // Téléchargement complet avant d'écrire : pas de projet à moitié créé.
    let mut downloaded = Vec::new();
    for file in &example.files {
        let rel = safe_relative(file).ok_or_else(|| format!("chemin refusé : {file}"))?;
        let bytes = fetch(source, &format!("{lang}/{}/{file}", example.folder))?;
        downloaded.push((rel, bytes));
    }
    for (rel, bytes) in downloaded {
        let path = dest.join(rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        vrs_core::fs::atomic_write(&path, bytes).map_err(|e| e.to_string())?;
    }
    // Le fichier de projet porte le nom du dossier d'origine : le renommer
    // s'il a fallu choisir un autre nom de dossier.
    if name != base_name {
        let old = dest.join(format!("{base_name}.{}", crate::PROJECT_EXTENSION));
        if old.is_file() {
            let _ = std::fs::rename(
                &old,
                dest.join(format!("{name}.{}", crate::PROJECT_EXTENSION)),
            );
        }
    }
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_examples() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../exemples")
    }

    #[test]
    fn languages() {
        let langs = vec!["en-US".to_string(), "fr-CA".to_string()];
        assert_eq!(choose_language(&langs, "fr-FR").as_deref(), Some("fr-CA"));
        assert_eq!(choose_language(&langs, "es-ES").as_deref(), Some("en-US"));
        assert_eq!(choose_language(&langs, "en-CA").as_deref(), Some("en-US"));
    }

    #[test]
    fn unsafe_paths_are_refused() {
        assert!(safe_relative("src/main.rs").is_some());
        assert!(safe_relative("../x").is_none());
        assert!(safe_relative("/etc/passwd").is_none());
        assert!(safe_relative("a/../../b").is_none());
    }

    /// Le catalogue du dépôt est valide et décrit exactement les fichiers présents.
    #[test]
    fn repository_catalog_matches_files() {
        let root = repo_examples();
        let catalog = load_catalog(&root.display().to_string()).unwrap();
        assert!(!catalog.examples.is_empty());
        for lang in &catalog.languages {
            for ex in &catalog.examples {
                let dir = root.join(lang).join(&ex.folder);
                let mut actual = Vec::new();
                let mut stack = vec![dir.clone()];
                while let Some(d) = stack.pop() {
                    for e in std::fs::read_dir(&d).unwrap().flatten() {
                        let p = e.path();
                        if p.is_dir() {
                            if p.file_name().is_some_and(|n| n != "target") {
                                stack.push(p);
                            }
                        } else {
                            actual.push(
                                p.strip_prefix(&dir)
                                    .unwrap()
                                    .to_string_lossy()
                                    .replace('\\', "/"),
                            );
                        }
                    }
                }
                actual.sort();
                let mut listed = ex.files.clone();
                listed.sort();
                assert_eq!(
                    actual, listed,
                    "{lang}/{} : fichiers différents de index.toml",
                    ex.folder
                );
                assert!(!ex.title_in(lang).is_empty() && !ex.description_in(lang).is_empty());
            }
        }
    }

    #[test]
    fn download_from_local_folder() {
        let root = repo_examples();
        let source = root.display().to_string();
        let catalog = load_catalog(&source).unwrap();
        let ex = catalog
            .examples
            .iter()
            .find(|e| e.id == "hello-terminal")
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let p1 = download_example(&source, &catalog, ex, "fr-FR", dir.path()).unwrap();
        let main = std::fs::read_to_string(p1.join("src/main.rs")).unwrap();
        assert!(main.contains("Bonjour"));
        let p2 = download_example(&source, &catalog, ex, "es-ES", dir.path()).unwrap();
        assert!(p2.ends_with("HelloWorldTerminal2"));
        assert!(
            std::fs::read_to_string(p2.join("src/main.rs"))
                .unwrap()
                .contains("Hello")
        );
        assert!(p2.join("HelloWorldTerminal2.vrsproj").is_file());
        crate::open_project(&p2).unwrap();
    }
}
