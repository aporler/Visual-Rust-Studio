//! Une variable de fenêtre (self.data) augmente à chaque clic ; un deuxième bouton la remet à zéro.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

/// Variables de la fenêtre (comme les variables déclarées dans un formulaire VB.NET). Ajoutez vos champs ici et utilisez-les avec `self.data`.
#[derive(Default)]
pub struct Form1Data {
    /// Nombre de clics.
    pub count: u32,
}

impl Form1 {
    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Les variables de la fenêtre sont dans self.data (voir Form1Data ci-dessus).
        self.data.count += 1;
        self.label1.text = self.data.count.to_string();
    }

    /// Gère l'événement Click de `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        self.data.count = 0;
        self.label1.text = String::from("0");
    }
}
