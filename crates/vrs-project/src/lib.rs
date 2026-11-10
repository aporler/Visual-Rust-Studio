//! Projets de Visual Rust Studio.
//!
//! Un projet VRS est **un projet Cargo ordinaire** auquel s'ajoutent :
//! - `<Nom>.vrsproj` : paramètres du projet pour l'IDE (TOML) ;
//! - `forms/*.vrsform` : conception des fenêtres (projets graphiques).
//!
//! Ces fichiers ne sont pas nécessaires pour compiler : `cargo build` fonctionne
//! sans VRS. Un dossier Cargo existant peut aussi être ouvert tel quel.

mod templates;
mod tree;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vrs_codegen::{Comments, SyncReport};
use vrs_components::{FORM_EXTENSION, FormDocument};

pub use templates::cargo_package_name;
pub use tree::{FileNode, file_tree};

/// Extension du fichier de projet.
pub const PROJECT_EXTENSION: &str = "vrsproj";
/// Version du format `.vrsproj`.
pub const PROJECT_FORMAT_VERSION: u32 = 1;

/// Type de projet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    /// Application graphique conçue visuellement.
    Gui,
    /// Application console.
    Console,
    /// Bibliothèque.
    Library,
    /// Projet Cargo existant, ouvert sans fichier `.vrsproj`.
    Cargo,
}

/// Stratégie à la fermeture de la fenêtre principale (utilisée par le runtime
/// multifenêtre, prévu pour la phase 5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseStrategy {
    /// Fermer la fenêtre principale ferme l'application.
    #[default]
    MainFormCloses,
    /// L'application se ferme quand la dernière fenêtre est fermée.
    LastFormCloses,
}

/// Contenu du fichier `.vrsproj`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectFile {
    pub vrs_format: u32,
    pub name: String,
    pub kind: ProjectKind,
    /// Fenêtre de démarrage (projets graphiques).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_form: Option<String>,
    /// Fenêtres du projet (fichiers `forms/<Nom>.vrsform`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forms: Vec<String>,
    #[serde(default)]
    pub close_strategy: CloseStrategy,
}

/// Erreur liée aux projets.
#[derive(Debug)]
pub enum ProjectError {
    InvalidName(String),
    AlreadyExists(PathBuf),
    NotAProject(PathBuf),
    Parse(String),
    Form(String),
    Codegen(vrs_codegen::CodegenError),
    Io(std::io::Error),
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName(n) => write!(f, "nom invalide : {n}"),
            Self::AlreadyExists(p) => write!(f, "le dossier existe déjà : {}", p.display()),
            Self::NotAProject(p) => write!(
                f,
                "{} ne contient ni fichier .{PROJECT_EXTENSION} ni Cargo.toml",
                p.display()
            ),
            Self::Parse(e) => write!(f, "{e}"),
            Self::Form(e) => write!(f, "{e}"),
            Self::Codegen(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ProjectError {}

impl From<std::io::Error> for ProjectError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<vrs_codegen::CodegenError> for ProjectError {
    fn from(e: vrs_codegen::CodegenError) -> Self {
        Self::Codegen(e)
    }
}

/// Projet ouvert dans l'IDE.
#[derive(Clone, Debug)]
pub struct Project {
    /// Répertoire racine (contient `Cargo.toml`).
    pub root: PathBuf,
    /// Chemin du `.vrsproj` (absent pour un projet Cargo importé).
    pub file_path: Option<PathBuf>,
    pub file: ProjectFile,
}

impl Project {
    /// Nom affiché.
    pub fn name(&self) -> &str {
        &self.file.name
    }

    /// Projet graphique conçu avec VRS ?
    pub fn is_gui(&self) -> bool {
        self.file.kind == ProjectKind::Gui
    }

    /// Chemin du fichier de conception d'une fenêtre.
    pub fn form_path(&self, form: &str) -> PathBuf {
        self.root
            .join("forms")
            .join(format!("{form}.{FORM_EXTENSION}"))
    }

    /// Charge une fenêtre.
    pub fn load_form(&self, form: &str) -> Result<FormDocument, ProjectError> {
        FormDocument::load(&self.form_path(form))
            .map_err(|e| ProjectError::Form(format!("{} : {e}", self.form_path(form).display())))
    }

    /// Charge toutes les fenêtres.
    pub fn load_forms(&self) -> Result<Vec<FormDocument>, ProjectError> {
        self.file.forms.iter().map(|f| self.load_form(f)).collect()
    }

    /// Enregistre le `.vrsproj`.
    pub fn save(&self) -> Result<(), ProjectError> {
        if let Some(path) = &self.file_path {
            write_project_file(path, &self.file)?;
        }
        Ok(())
    }

    /// Régénère tout le code (projets graphiques). `overrides` remplace les
    /// fenêtres lues sur le disque (fenêtres ouvertes non encore enregistrées).
    pub fn sync_code(
        &self,
        overrides: &[FormDocument],
        comments: &dyn Comments,
    ) -> Result<SyncReport, ProjectError> {
        if !self.is_gui() {
            return Ok(SyncReport::default());
        }
        let mut forms = self.load_forms()?;
        for f in &mut forms {
            if let Some(o) = overrides.iter().find(|o| o.name == f.name) {
                *f = o.clone();
            }
        }
        let startup = self
            .file
            .startup_form
            .clone()
            .or_else(|| self.file.forms.first().cloned())
            .unwrap_or_default();
        Ok(vrs_codegen::sync_project(
            &self.root,
            &forms,
            &startup,
            self.file.close_strategy == CloseStrategy::LastFormCloses,
            comments,
        )?)
    }

    /// Le nom de fenêtre est-il valide et libre ?
    pub fn check_form_name(&self, name: &str) -> Result<(), ProjectError> {
        if !vrs_core::ident::is_upper_camel_ident(name) {
            return Err(ProjectError::InvalidName(name.to_owned()));
        }
        if self.file.forms.iter().any(|f| f.eq_ignore_ascii_case(name)) {
            return Err(ProjectError::AlreadyExists(self.form_path(name)));
        }
        Ok(())
    }

    /// Prochain nom libre : `Form2`, `Form3`…
    pub fn next_form_name(&self) -> String {
        (1..)
            .map(|i| format!("Form{i}"))
            .find(|n| self.check_form_name(n).is_ok())
            .expect("itérateur infini")
    }

    /// Ajoute une fenêtre au projet et génère son code.
    pub fn add_form(
        &mut self,
        name: &str,
        comments: &dyn Comments,
    ) -> Result<FormDocument, ProjectError> {
        self.check_form_name(name)?;
        let form = FormDocument::new(name);
        form.save(&self.form_path(name))
            .map_err(|e| ProjectError::Form(e.to_string()))?;
        self.file.forms.push(name.to_owned());
        if self.file.startup_form.is_none() {
            self.file.startup_form = Some(name.to_owned());
        }
        self.save()?;
        self.sync_code(&[], comments)?;
        Ok(form)
    }
}

fn write_project_file(path: &Path, file: &ProjectFile) -> Result<(), ProjectError> {
    let body = toml::to_string_pretty(file).map_err(|e| ProjectError::Parse(e.to_string()))?;
    let text = format!(
        "# Projet Visual Rust Studio. Ce fichier n'est pas nécessaire pour compiler avec Cargo.\n\n{body}"
    );
    vrs_core::fs::atomic_write(path, text)?;
    Ok(())
}

/// Exécutable produit par `cargo build` pour ce projet (premier `[[bin]]`, sinon
/// le nom du paquet), dans `target/debug` ou `target/release`.
pub fn binary_path(root: &Path, release: bool) -> Option<PathBuf> {
    let manifest: toml::Table = std::fs::read_to_string(root.join("Cargo.toml"))
        .ok()?
        .parse()
        .ok()?;
    let bin = manifest
        .get("bin")
        .and_then(|b| b.as_array())
        .and_then(|a| a.first())
        .and_then(|b| b.get("name"))
        .and_then(|n| n.as_str())
        .or_else(|| manifest.get("package")?.get("name")?.as_str())?
        .to_owned();
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    let profile = if release { "release" } else { "debug" };
    Some(
        target
            .join(profile)
            .join(format!("{bin}{}", std::env::consts::EXE_SUFFIX)),
    )
}

/// Ouvre un projet : fichier `.vrsproj`, dossier qui en contient un, ou dossier Cargo.
pub fn open_project(path: &Path) -> Result<Project, ProjectError> {
    let path = path.canonicalize()?;
    let (root, file_path) = if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some(PROJECT_EXTENSION) {
            (
                path.parent().unwrap_or(&path).to_path_buf(),
                Some(path.clone()),
            )
        } else if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
            let root = path.parent().unwrap_or(&path).to_path_buf();
            let fp = find_project_file(&root);
            (root, fp)
        } else {
            return Err(ProjectError::NotAProject(path));
        }
    } else {
        let fp = find_project_file(&path);
        (path.clone(), fp)
    };

    if let Some(fp) = file_path {
        let text = std::fs::read_to_string(&fp)?;
        let file: ProjectFile = toml::from_str(&text)
            .map_err(|e| ProjectError::Parse(format!("{} : {e}", fp.display())))?;
        if file.vrs_format > PROJECT_FORMAT_VERSION {
            return Err(ProjectError::Parse(format!(
                "{} : version {} non prise en charge",
                fp.display(),
                file.vrs_format
            )));
        }
        return Ok(Project {
            root,
            file_path: Some(fp),
            file,
        });
    }

    let manifest = root.join("Cargo.toml");
    if !manifest.is_file() {
        return Err(ProjectError::NotAProject(root));
    }
    let name = std::fs::read_to_string(&manifest)
        .ok()
        .and_then(|t| t.parse::<toml::Table>().ok())
        .and_then(|t| t.get("package")?.get("name")?.as_str().map(str::to_owned))
        .unwrap_or_else(|| {
            root.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        });
    Ok(Project {
        root,
        file_path: None,
        file: ProjectFile {
            vrs_format: PROJECT_FORMAT_VERSION,
            name,
            kind: ProjectKind::Cargo,
            startup_form: None,
            forms: Vec::new(),
            close_strategy: CloseStrategy::default(),
        },
    })
}

fn find_project_file(dir: &Path) -> Option<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(PROJECT_EXTENSION))
        .collect();
    found.sort();
    found.into_iter().next()
}

/// Crée un nouveau projet dans `parent/name`.
pub fn create_project(
    parent: &Path,
    name: &str,
    kind: ProjectKind,
    comments: &dyn Comments,
) -> Result<Project, ProjectError> {
    if !vrs_core::ident::is_valid_crate_name(name) {
        return Err(ProjectError::InvalidName(name.to_owned()));
    }
    let root = parent.join(name);
    if root.exists()
        && std::fs::read_dir(&root)
            .map(|mut d| d.next().is_some())
            .unwrap_or(true)
    {
        return Err(ProjectError::AlreadyExists(root));
    }
    std::fs::create_dir_all(&root)?;
    let root = root.canonicalize()?;
    templates::write_template(&root, name, kind)?;

    let file = ProjectFile {
        vrs_format: PROJECT_FORMAT_VERSION,
        name: name.to_owned(),
        kind,
        startup_form: (kind == ProjectKind::Gui).then(|| "Form1".to_owned()),
        forms: if kind == ProjectKind::Gui {
            vec!["Form1".to_owned()]
        } else {
            Vec::new()
        },
        close_strategy: CloseStrategy::default(),
    };
    let file_path = root.join(format!("{name}.{PROJECT_EXTENSION}"));
    write_project_file(&file_path, &file)?;
    let project = Project {
        root,
        file_path: Some(file_path),
        file,
    };
    if kind == ProjectKind::Gui {
        let form = FormDocument::new("Form1");
        form.save(&project.form_path("Form1"))
            .map_err(|e| ProjectError::Form(e.to_string()))?;
        project.sync_code(&[], comments)?;
    }
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vrs_codegen::EnglishComments;

    #[test]
    fn create_and_reopen_gui_project() {
        let dir = tempfile::tempdir().unwrap();
        let p = create_project(dir.path(), "MonApp", ProjectKind::Gui, &EnglishComments).unwrap();
        for f in [
            "Cargo.toml",
            "MonApp.vrsproj",
            "forms/Form1.vrsform",
            "src/main.rs",
            "src/vrs/mod.rs",
            "src/forms/mod.rs",
            "src/forms/form1.rs",
            "src/forms/form1_designer.rs",
            ".gitignore",
            "README.md",
        ] {
            assert!(p.root.join(f).is_file(), "{f} manquant");
        }
        let cargo = std::fs::read_to_string(p.root.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("name = \"mon-app\""));
        assert!(cargo.contains("eframe = \"0.37\""));

        let reopened = open_project(&p.root).unwrap();
        assert_eq!(reopened.file, p.file);
        let forms = reopened.load_forms().unwrap();
        assert_eq!(forms[0].name, "Form1");

        // Le dossier existe déjà : refus.
        assert!(matches!(
            create_project(dir.path(), "MonApp", ProjectKind::Gui, &EnglishComments),
            Err(ProjectError::AlreadyExists(_))
        ));
    }

    #[test]
    fn add_form_and_user_code_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let mut p = create_project(dir.path(), "app", ProjectKind::Gui, &EnglishComments).unwrap();
        let user = p.root.join("src/forms/form1.rs");
        let mut text = std::fs::read_to_string(&user).unwrap();
        text.push_str("\n// mon commentaire précieux\n");
        std::fs::write(&user, &text).unwrap();

        assert_eq!(p.next_form_name(), "Form2");
        p.add_form("Form2", &EnglishComments).unwrap();
        assert!(p.root.join("src/forms/form2_designer.rs").is_file());
        assert!(p.check_form_name("Form2").is_err());
        assert!(p.check_form_name("form3").is_err());
        assert_eq!(std::fs::read_to_string(&user).unwrap(), text);
        let reopened = open_project(&p.root).unwrap();
        assert_eq!(reopened.file.forms, vec!["Form1", "Form2"]);
    }

    #[test]
    fn open_plain_cargo_folder() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"outil\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let p = open_project(dir.path()).unwrap();
        assert_eq!(p.name(), "outil");
        assert_eq!(p.file.kind, ProjectKind::Cargo);
        assert!(p.file_path.is_none());
        assert!(open_project(&dir.path().join("Cargo.toml")).is_ok());
        let empty = tempfile::tempdir().unwrap();
        assert!(matches!(
            open_project(empty.path()),
            Err(ProjectError::NotAProject(_))
        ));
    }

    #[test]
    fn binary_paths() {
        let dir = tempfile::tempdir().unwrap();
        let p =
            create_project(dir.path(), "MonApp", ProjectKind::Console, &EnglishComments).unwrap();
        let bin = binary_path(&p.root, false).unwrap();
        assert!(bin.ends_with(format!("debug/mon-app{}", std::env::consts::EXE_SUFFIX)));
    }

    #[test]
    fn invalid_names() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            create_project(
                dir.path(),
                "mon app",
                ProjectKind::Console,
                &EnglishComments
            ),
            Err(ProjectError::InvalidName(_))
        ));
    }

    #[test]
    fn console_and_library() {
        let dir = tempfile::tempdir().unwrap();
        let c = create_project(dir.path(), "cli", ProjectKind::Console, &EnglishComments).unwrap();
        assert!(c.root.join("src/main.rs").is_file());
        let l = create_project(dir.path(), "lib1", ProjectKind::Library, &EnglishComments).unwrap();
        assert!(l.root.join("src/lib.rs").is_file());
        assert!(!l.root.join("forms").exists());
    }
}
