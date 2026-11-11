//! Form1 ouvre une boîte de dialogue modale (Form2) qui demande votre nom, puis lit la réponse.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Ouvre Form2 en boîte de dialogue modale ; la fermeture (closure) s'exécute à sa fermeture.
        app.show_dialog_then(super::Form2::default(), |me: &mut Form1, form2: super::Form2, _app| {
            // me est cette fenêtre (Form1), form2 est la boîte de dialogue fermée.
            if form2.data.accepted && !form2.text_box1.text.trim().is_empty() {
                me.label1.text = format!("Bonjour, {} !", form2.text_box1.text.trim());
            }
        });
    }
}
