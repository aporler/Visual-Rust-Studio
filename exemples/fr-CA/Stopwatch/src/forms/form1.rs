//! Un Timer déclenche Tick toutes les 100 ms : démarrer, arrêter, remettre à zéro, et une barre de progression pour les secondes.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {
    /// Temps écoulé, en dixièmes de seconde.
    pub tenths: u32,
}

impl Form1 {
    /// Gère l'événement Tick de `timer1`.
    pub fn timer1_tick(&mut self, app: &mut AppContext) {
        self.data.tenths += 1;
        self.show_time();
    }

    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // enabled démarre ou arrête le Timer.
        self.timer1.enabled = true;
    }

    /// Gère l'événement Click de `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        self.timer1.enabled = false;
    }

    /// Gère l'événement Click de `button3`.
    pub fn button3_click(&mut self, app: &mut AppContext) {
        self.data.tenths = 0;
        self.show_time();
    }

    /// Affiche le temps au format mm:ss.d (`:02` ajoute des zéros devant).
    fn show_time(&mut self) {
        let minutes = self.data.tenths / 600;
        let seconds = (self.data.tenths / 10) % 60;
        let tenths = self.data.tenths % 10;
        self.label1.text = format!("{minutes:02}:{seconds:02}.{tenths}");
        self.progress_bar1.value = seconds as f32 + tenths as f32 / 10.0;
    }
}
