//! Ajoute et supprime des éléments dans une liste (Vec<String>), avec une petite méthode utilitaire.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        let task = self.text_box1.text.trim().to_owned();
        // Rien à ajouter si le texte est vide.
        if task.is_empty() {
            return;
        }
        self.list_box1.items.push(task);
        self.text_box1.text.clear();
        self.update_count();
    }

    /// Gère l'événement Click de `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        // selected est une Option<usize> : Some(index) si un élément est sélectionné.
        if let Some(i) = self.list_box1.selected {
            self.list_box1.items.remove(i);
            self.list_box1.selected = None;
            self.update_count();
        }
    }

    /// Met à jour l'étiquette qui affiche le nombre de tâches.
    fn update_count(&mut self) {
        self.label1.text = format!("{} tâche(s)", self.list_box1.items.len());
    }
}
