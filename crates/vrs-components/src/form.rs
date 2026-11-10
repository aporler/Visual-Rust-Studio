//! Modèle déclaratif d'une fenêtre (formulaire).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{ComponentKind, EventKind};

/// Extension des fichiers de conception.
pub const FORM_EXTENSION: &str = "vrsform";

/// Version du format `.vrsform`. Incrémentée en cas de changement incompatible.
pub const FORM_FORMAT_VERSION: u32 = 1;

/// Un contrôle placé sur une fenêtre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Control {
    /// Nom du champ Rust (`button1`). Identifiant `snake_case` unique dans la fenêtre.
    pub name: String,
    pub kind: ComponentKind,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub text: String,
    #[serde(default = "yes")]
    pub visible: bool,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tooltip: String,
    /// Événement → nom de la méthode Rust qui le gère.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub events: BTreeMap<EventKind, String>,
}

fn yes() -> bool {
    true
}

impl Control {
    /// Nouveau contrôle avec les valeurs par défaut du composant.
    pub fn new(kind: ComponentKind, name: String, index: usize, x: f32, y: f32) -> Self {
        let (width, height) = kind.default_size();
        Self {
            name,
            kind,
            x,
            y,
            width,
            height,
            text: kind.default_text(index),
            visible: true,
            enabled: true,
            tooltip: String::new(),
            events: BTreeMap::new(),
        }
    }

    /// Rectangle (x, y, largeur, hauteur).
    pub fn rect(&self) -> [f32; 4] {
        [self.x, self.y, self.width, self.height]
    }
}

/// Une fenêtre conçue visuellement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormDocument {
    pub format_version: u32,
    /// Nom du type Rust (`Form1`). Identifiant `UpperCamelCase`.
    pub name: String,
    /// Titre affiché dans la barre de la fenêtre.
    pub title: String,
    pub width: f32,
    pub height: f32,
    /// Événements de la fenêtre elle-même (Load…).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub events: BTreeMap<EventKind, String>,
    /// Contrôles, dans l'ordre d'affichage (le dernier est au premier plan).
    #[serde(default)]
    pub controls: Vec<Control>,
}

/// Erreur de lecture ou d'écriture d'un formulaire.
#[derive(Debug)]
pub enum FormError {
    Io(std::io::Error),
    Parse(String),
    UnsupportedVersion(u32),
}

impl std::fmt::Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Parse(e) => write!(f, "format .vrsform invalide : {e}"),
            Self::UnsupportedVersion(v) => {
                write!(
                    f,
                    "version de format {v} non prise en charge (max {FORM_FORMAT_VERSION})"
                )
            }
        }
    }
}

impl std::error::Error for FormError {}

impl From<std::io::Error> for FormError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Raison pour laquelle un nom de contrôle est refusé.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameError {
    /// Pas un identifiant `snake_case` valide (ou mot-clé).
    Invalid,
    /// Déjà utilisé dans cette fenêtre.
    Taken,
}

impl FormDocument {
    /// Nouvelle fenêtre vide.
    pub fn new(name: &str) -> Self {
        Self {
            format_version: FORM_FORMAT_VERSION,
            name: name.to_owned(),
            title: name.to_owned(),
            width: 480.0,
            height: 320.0,
            events: BTreeMap::new(),
            controls: Vec::new(),
        }
    }

    /// Nom du module Rust (`Form1` → `form1`).
    pub fn module_name(&self) -> String {
        vrs_core::ident::to_snake_case(&self.name)
    }

    /// Lit un fichier `.vrsform`.
    pub fn load(path: &Path) -> Result<Self, FormError> {
        let text = std::fs::read_to_string(path)?;
        Self::from_ron(&text)
    }

    /// Analyse le contenu d'un fichier `.vrsform`.
    pub fn from_ron(text: &str) -> Result<Self, FormError> {
        let doc: Self = ron::from_str(text).map_err(|e| FormError::Parse(e.to_string()))?;
        if doc.format_version > FORM_FORMAT_VERSION {
            return Err(FormError::UnsupportedVersion(doc.format_version));
        }
        Ok(doc)
    }

    /// Sérialise en RON lisible.
    pub fn to_ron(&self) -> String {
        let cfg = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .struct_names(true)
            .separate_tuple_members(false);
        let body = ron::ser::to_string_pretty(self, cfg).expect("sérialisation RON");
        format!(
            "// Visual Rust Studio — conception de la fenêtre {}.\n\
             // Métadonnées de présentation uniquement (aucun code). Modifiées par le concepteur.\n\
             {body}\n",
            self.name
        )
    }

    /// Enregistre de façon atomique.
    pub fn save(&self, path: &Path) -> Result<(), FormError> {
        vrs_core::fs::atomic_write(path, self.to_ron())?;
        Ok(())
    }

    /// Index du contrôle portant ce nom.
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.controls.iter().position(|c| c.name == name)
    }

    /// Contrôle portant ce nom.
    pub fn control(&self, name: &str) -> Option<&Control> {
        self.controls.iter().find(|c| c.name == name)
    }

    /// Contrôle mutable portant ce nom.
    pub fn control_mut(&mut self, name: &str) -> Option<&mut Control> {
        self.controls.iter_mut().find(|c| c.name == name)
    }

    /// Premier nom libre pour un type de composant : `button1`, `button2`…
    /// Retourne aussi l'index utilisé (pour le texte par défaut `Button2`).
    pub fn next_name(&self, kind: ComponentKind) -> (String, usize) {
        let prefix = kind.name_prefix();
        (1..)
            .map(|i| (format!("{prefix}{i}"), i))
            .find(|(n, _)| self.check_name(n, None).is_ok())
            .expect("itérateur infini")
    }

    /// Vérifie un nom de contrôle. `current` : nom actuel du contrôle renommé.
    pub fn check_name(&self, name: &str, current: Option<&str>) -> Result<(), NameError> {
        if !vrs_core::ident::is_snake_case_ident(name) || is_reserved_member(name) {
            return Err(NameError::Invalid);
        }
        if Some(name) != current && self.controls.iter().any(|c| c.name == name) {
            return Err(NameError::Taken);
        }
        Ok(())
    }

    /// Ajoute un contrôle et retourne son nom.
    pub fn add_control(&mut self, kind: ComponentKind, x: f32, y: f32) -> String {
        let (name, index) = self.next_name(kind);
        self.controls
            .push(Control::new(kind, name.clone(), index, x, y));
        name
    }

    /// Renomme un contrôle (et ses gestionnaires dont le nom suivait la convention).
    pub fn rename_control(&mut self, old: &str, new: &str) -> Result<(), NameError> {
        self.check_name(new, Some(old))?;
        if let Some(c) = self.control_mut(old) {
            c.name = new.to_owned();
            for (event, handler) in c.events.iter_mut() {
                if *handler == event.default_handler(old) {
                    *handler = event.default_handler(new);
                }
            }
        }
        Ok(())
    }

    /// Tous les gestionnaires d'événements (méthode → (contrôle ou `None` pour la fenêtre, événement)).
    pub fn handlers(&self) -> Vec<(String, Option<String>, EventKind)> {
        let mut v: Vec<_> = self
            .events
            .iter()
            .map(|(e, h)| (h.clone(), None, *e))
            .collect();
        for c in &self.controls {
            for (e, h) in &c.events {
                v.push((h.clone(), Some(c.name.clone()), *e));
            }
        }
        v
    }

    /// Problèmes qui empêcheraient la génération de code.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if !vrs_core::ident::is_upper_camel_ident(&self.name) {
            problems.push(format!("nom de fenêtre invalide : {}", self.name));
        }
        let mut seen = std::collections::BTreeSet::new();
        for c in &self.controls {
            if !vrs_core::ident::is_snake_case_ident(&c.name) || is_reserved_member(&c.name) {
                problems.push(format!("nom de contrôle invalide : {}", c.name));
            }
            if !seen.insert(c.name.as_str()) {
                problems.push(format!("nom de contrôle en double : {}", c.name));
            }
        }
        let mut handlers = std::collections::BTreeSet::new();
        for (h, ..) in self.handlers() {
            if !vrs_core::ident::is_valid_ident(&h) {
                problems.push(format!("nom de gestionnaire invalide : {h}"));
            }
            if !handlers.insert(h.clone()) {
                problems.push(format!("gestionnaire utilisé deux fois : {h}"));
            }
        }
        problems
    }
}

/// Noms réservés par le code généré (méthodes du trait `Form`, etc.).
fn is_reserved_member(name: &str) -> bool {
    matches!(name, "show" | "title" | "size" | "load" | "default" | "new")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_snake_case() {
        let mut f = FormDocument::new("Form1");
        assert_eq!(f.add_control(ComponentKind::Button, 8.0, 8.0), "button1");
        assert_eq!(f.add_control(ComponentKind::Button, 8.0, 40.0), "button2");
        assert_eq!(
            f.add_control(ComponentKind::TextBox, 8.0, 80.0),
            "text_box1"
        );
        assert_eq!(f.controls[1].text, "Button2");
        assert_eq!(f.check_name("button1", None), Err(NameError::Taken));
        assert_eq!(f.check_name("Button9", None), Err(NameError::Invalid));
        assert_eq!(f.check_name("fn", None), Err(NameError::Invalid));
        assert_eq!(f.check_name("show", None), Err(NameError::Invalid));
        assert!(f.check_name("button1", Some("button1")).is_ok());
    }

    #[test]
    fn rename_updates_conventional_handlers() {
        let mut f = FormDocument::new("Form1");
        let n = f.add_control(ComponentKind::Button, 0.0, 0.0);
        f.control_mut(&n)
            .unwrap()
            .events
            .insert(EventKind::Click, "button1_click".into());
        f.rename_control("button1", "ok_button").unwrap();
        assert_eq!(f.controls[0].events[&EventKind::Click], "ok_button_click");
    }

    #[test]
    fn ron_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Form1.vrsform");
        let mut f = FormDocument::new("Form1");
        f.title = "Ma fenêtre — été".into();
        f.add_control(ComponentKind::Label, 10.0, 10.0);
        let b = f.add_control(ComponentKind::Button, 10.0, 40.0);
        f.control_mut(&b)
            .unwrap()
            .events
            .insert(EventKind::Click, "button1_click".into());
        f.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("// Visual Rust Studio"));
        let loaded = FormDocument::load(&path).unwrap();
        assert_eq!(f, loaded);
        assert!(loaded.validate().is_empty());
    }

    #[test]
    fn future_version_rejected() {
        let mut f = FormDocument::new("Form1");
        f.format_version = 99;
        let text = f.to_ron();
        assert!(matches!(
            FormDocument::from_ron(&text),
            Err(FormError::UnsupportedVersion(99))
        ));
    }
}
