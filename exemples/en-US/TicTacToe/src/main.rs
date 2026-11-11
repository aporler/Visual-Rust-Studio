//! Point d'entrée de l'application.
//!
//! Créé par Visual Rust Studio ; ce fichier vous appartient et n'est jamais régénéré.
//! La fenêtre de démarrage est définie dans le projet (src/forms/mod.rs).

// En mode Release sous Windows, aucune console n'est affichée.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod forms;
mod vrs;

fn main() -> eframe::Result {
    vrs::run_with(forms::startup_form(), forms::CLOSE_STRATEGY)
}
