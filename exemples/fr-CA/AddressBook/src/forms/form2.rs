//! Une liste de contacts ; une deuxième fenêtre (boîte de dialogue modale) demande le nom et le téléphone.
#![allow(unused_imports, unused_variables)]

use super::Form2;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form2Data {
    /// true si l'utilisateur a cliqué sur OK.
    pub accepted: bool,
}

impl Form2 {
    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        self.data.accepted = true;
        app.close();
    }

    /// Gère l'événement Click de `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        app.close();
    }
}
