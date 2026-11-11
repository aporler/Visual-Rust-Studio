//! Trois curseurs (rouge, vert, bleu) changent la couleur de fond d'un panneau : événements ValueChanged et propriété BackColor.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Gère l'événement Load de `Form1`.
    pub fn form1_load(&mut self, app: &mut AppContext) {
        self.update_color();
    }

    /// Gère l'événement ValueChanged de `slider1`.
    pub fn slider1_value_changed(&mut self, app: &mut AppContext) {
        self.update_color();
    }

    /// Gère l'événement ValueChanged de `slider2`.
    pub fn slider2_value_changed(&mut self, app: &mut AppContext) {
        self.update_color();
    }

    /// Gère l'événement ValueChanged de `slider3`.
    pub fn slider3_value_changed(&mut self, app: &mut AppContext) {
        self.update_color();
    }

    /// Affiche la couleur choisie avec les trois curseurs.
    fn update_color(&mut self) {
        // Les valeurs des curseurs sont des f32 ; `as u8` les convertit en 0–255.
        let (r, g, b) = (
            self.slider1.value as u8,
            self.slider2.value as u8,
            self.slider3.value as u8,
        );
        self.panel1.back_color = Some(crate::vrs::Color::from_rgb(r, g, b));
        self.label1.text = format!("#{r:02X}{g:02X}{b:02X}  ({r}, {g}, {b})");
    }
}
