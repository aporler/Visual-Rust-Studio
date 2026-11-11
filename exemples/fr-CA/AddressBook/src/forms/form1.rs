//! Une liste de contacts ; une deuxième fenêtre (boîte de dialogue modale) demande le nom et le téléphone.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Ouvre Form2 en boîte de dialogue modale ; la fermeture (closure) s'exécute à sa fermeture.
        app.show_dialog_then(super::Form2::default(), |me: &mut Form1, contact: super::Form2, _app| {
            let name = contact.text_box1.text.trim();
            let phone = contact.text_box2.text.trim();
            if contact.data.accepted && !name.is_empty() {
                me.list_box1.items.push(format!("{name} — {phone}"));
                me.list_box1.items.sort();
            }
        });
    }

    /// Gère l'événement Click de `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        if let Some(i) = self.list_box1.selected.take() {
            self.list_box1.items.remove(i);
        }
    }
}
