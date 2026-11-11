//! Une zone de texte multiligne, un menu Fichier (Nouveau, Ouvrir, Enregistrer, Quitter) et la lecture/écriture d'un fichier avec std::fs.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

/// Fichier utilisé pour ouvrir et enregistrer le texte (dans le dossier courant).
const FILE: &str = "notes.txt";

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Gère l'événement Click de `menu_nouveau`.
    pub fn menu_nouveau_click(&mut self, app: &mut AppContext) {
        self.text_box1.text.clear();
        self.label1.text.clear();
    }

    /// Gère l'événement Click de `menu_ouvrir`.
    pub fn menu_ouvrir_click(&mut self, app: &mut AppContext) {
        // read_to_string donne Ok(texte) ou Err(erreur) : le fichier n'existe peut-être pas encore.
        match std::fs::read_to_string(FILE) {
            Ok(text) => {
                self.text_box1.text = text;
                self.label1.text = format!("Ouvert : {}", FILE);
            }
            Err(e) => self.label1.text = format!("Erreur : {}", e),
        }
    }

    /// Gère l'événement Click de `menu_enregistrer`.
    pub fn menu_enregistrer_click(&mut self, app: &mut AppContext) {
        match std::fs::write(FILE, &self.text_box1.text) {
            Ok(()) => self.label1.text = format!("Enregistré dans {}", FILE),
            Err(e) => self.label1.text = format!("Erreur : {}", e),
        }
    }

    /// Gère l'événement Click de `menu_quitter`.
    pub fn menu_quitter_click(&mut self, app: &mut AppContext) {
        app.exit();
    }
}
