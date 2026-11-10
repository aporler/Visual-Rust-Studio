//! Fichiers des modèles de projets.

use std::path::Path;

use crate::ProjectKind;

/// Nom de paquet Cargo dérivé du nom du projet (`MonApp` → `mon-app`).
pub fn cargo_package_name(project_name: &str) -> String {
    vrs_core::ident::to_snake_case(project_name).replace('_', "-")
}

const GITIGNORE: &str = "/target\n*.bak\n";

fn cargo_toml(name: &str, kind: ProjectKind) -> String {
    let package = cargo_package_name(name);
    let mut s = format!(
        "[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n"
    );
    if kind == ProjectKind::Gui {
        s.push_str(&format!("eframe = \"{}\"\n", vrs_codegen::EFRAME_VERSION));
        s.push_str(
            "\n# Optimise les dépendances (egui) même en mode Debug : l'interface reste fluide.\n\
             [profile.dev.package.\"*\"]\nopt-level = 2\n",
        );
    }
    s
}

const GUI_MAIN: &str = r#"//! Point d'entrée de l'application.
//!
//! Créé par Visual Rust Studio ; ce fichier vous appartient et n'est jamais régénéré.
//! La fenêtre de démarrage est définie dans le projet (src/forms/mod.rs).

// En mode Release sous Windows, aucune console n'est affichée.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod forms;
mod vrs;

fn main() -> eframe::Result {
    vrs::run(forms::startup_form())
}
"#;

const CONSOLE_MAIN: &str = r#"fn main() {
    println!("Bonjour, le monde ! / Hello, world!");
}
"#;

const LIBRARY_LIB: &str = r#"//! Bibliothèque Rust.

/// Additionne deux nombres.
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(add(2, 2), 4);
    }
}
"#;

fn readme(name: &str, kind: ProjectKind) -> String {
    let url = "https://github.com/aporler/Visual-Rust-Studio";
    let gui = kind == ProjectKind::Gui;
    let mut s = format!(
        "# {name}\n\n\
         *[English](#english) · [Français](#français)*\n\n\
         ## English\n\n\
         Project created with [Visual Rust Studio]({url}).\n\n\
         It is a standard Cargo project: it builds and runs **without** Visual Rust Studio.\n\n\
         ```sh\ncargo run          # build and run\ncargo build --release\n```\n\n\
         The code of this project belongs to you: license it as you wish.\n"
    );
    if gui {
        s.push_str(
            "\n### Layout\n\n\
             | File | Role |\n|---|---|\n\
             | `src/main.rs` | entry point (yours) |\n\
             | `src/forms/form1.rs` | **your code**: Form1 event handlers |\n\
             | `src/forms/form1_designer.rs` | generated from the designer (do not edit) |\n\
             | `src/forms/mod.rs` | generated: window list |\n\
             | `src/vrs/mod.rs` | small runtime on top of egui/eframe (do not edit) |\n\
             | `forms/*.vrsform` | window designs (metadata, no code) |\n\
             | `*.vrsproj` | project settings for Visual Rust Studio |\n",
        );
    }
    s.push_str(&format!(
        "\n## Français\n\n\
         Projet créé avec [Visual Rust Studio]({url}).\n\n\
         C'est un projet Cargo standard : il compile et s'exécute **sans** Visual Rust Studio.\n\n\
         ```sh\ncargo run          # compiler et exécuter\ncargo build --release\n```\n\n\
         Le code de ce projet vous appartient : choisissez librement sa licence.\n"
    ));
    if gui {
        s.push_str(
            "\n### Organisation\n\n\
             | Fichier | Rôle |\n|---|---|\n\
             | `src/main.rs` | point d'entrée (le vôtre) |\n\
             | `src/forms/form1.rs` | **votre code** : gestionnaires d'événements de Form1 |\n\
             | `src/forms/form1_designer.rs` | généré à partir du concepteur (ne pas modifier) |\n\
             | `src/forms/mod.rs` | généré : liste des fenêtres |\n\
             | `src/vrs/mod.rs` | petit runtime au-dessus d'egui/eframe (ne pas modifier) |\n\
             | `forms/*.vrsform` | conception des fenêtres (métadonnées, pas de code) |\n\
             | `*.vrsproj` | paramètres du projet pour Visual Rust Studio |\n",
        );
    }
    s
}

/// Écrit les fichiers d'un modèle dans `root` (qui existe et est vide).
pub(crate) fn write_template(root: &Path, name: &str, kind: ProjectKind) -> std::io::Result<()> {
    use vrs_core::fs::atomic_write;
    atomic_write(&root.join("Cargo.toml"), cargo_toml(name, kind))?;
    atomic_write(&root.join(".gitignore"), GITIGNORE)?;
    atomic_write(&root.join("README.md"), readme(name, kind))?;
    match kind {
        ProjectKind::Gui => atomic_write(&root.join("src/main.rs"), GUI_MAIN)?,
        ProjectKind::Console | ProjectKind::Cargo => {
            atomic_write(&root.join("src/main.rs"), CONSOLE_MAIN)?
        }
        ProjectKind::Library => atomic_write(&root.join("src/lib.rs"), LIBRARY_LIB)?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_names() {
        assert_eq!(cargo_package_name("MonApp"), "mon-app");
        assert_eq!(cargo_package_name("my_tool"), "my-tool");
        assert_eq!(cargo_package_name("app2"), "app2");
    }
}
