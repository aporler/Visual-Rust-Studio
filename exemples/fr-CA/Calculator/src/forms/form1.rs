//! Deux nombres, une opération choisie dans une ComboBox, un match sur la sélection et une boîte de message pour la division par zéro.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

/// Convertit le texte en nombre : Some(nombre), ou None si le texte n'est pas un nombre.
fn number(text: &str) -> Option<f64> {
    text.trim().replace(',', ".").parse().ok()
}

impl Form1 {
    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        let (Some(a), Some(b)) = (number(&self.text_box1.text), number(&self.text_box2.text)) else {
            self.label1.text = String::from("Veuillez saisir un nombre.");
            return;
        };
        // L'opération choisie (index dans la ComboBox).
        let result = match self.combo_box1.selected {
            Some(0) => a + b,
            Some(1) => a - b,
            Some(2) => a * b,
            Some(3) if b == 0.0 => {
                app.message_box("Division par zéro impossible.");
                return;
            }
            Some(3) => a / b,
            _ => return,
        };
        self.label1.text = format!("= {result}");
    }
}
