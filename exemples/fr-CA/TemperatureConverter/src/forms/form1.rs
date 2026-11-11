//! Lit une zone de texte, convertit le texte en nombre et gère les erreurs de saisie (°C vers °F).
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Lire le texte saisi (la virgule est acceptée comme séparateur décimal).
        let input = self.text_box1.text.trim().replace(',', ".");
        // parse() donne Ok(nombre) ou Err(erreur) : les deux cas doivent être traités.
        match input.parse::<f64>() {
            Ok(celsius) => {
                let fahrenheit = celsius * 9.0 / 5.0 + 32.0;
                self.label2.text = format!("{celsius} °C = {fahrenheit:.1} °F");
            }
            Err(_) => {
                self.label2.text = String::from("Veuillez saisir un nombre.");
            }
        }
    }
}
