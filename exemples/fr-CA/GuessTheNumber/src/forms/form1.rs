//! Un nombre secret, des indices « plus grand / plus petit », un compteur d'essais et une boîte de message à la victoire.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {
    /// Nombre à deviner.
    pub secret: u32,
    /// Nombre d'essais.
    pub tries: u32,
}

/// Nombre pseudo-aléatoire simple tiré de l'horloge (sans bibliothèque externe).
fn random_1_to_100() -> u32 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42);
    nanos % 100 + 1
}

impl Form1 {
    /// Gère l'événement Load de `Form1`.
    pub fn form1_load(&mut self, app: &mut AppContext) {
        self.data.secret = random_1_to_100();
    }

    /// Gère l'événement Click de `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        let Ok(guess) = self.text_box1.text.trim().parse::<u32>() else {
            self.label2.text = String::from("Veuillez saisir un nombre.");
            return;
        };
        self.data.tries += 1;
        self.label3.text = format!("Essais : {}", self.data.tries);
        // match compare la proposition avec le nombre secret.
        match guess.cmp(&self.data.secret) {
            std::cmp::Ordering::Less => self.label2.text = String::from("Plus grand !"),
            std::cmp::Ordering::Greater => self.label2.text = String::from("Plus petit !"),
            std::cmp::Ordering::Equal => {
                app.message_box(format!("Bravo ! Le nombre était {} ! Essais : {}", self.data.secret, self.data.tries));
                // Nouvelle partie.
                self.data.secret = random_1_to_100();
                self.data.tries = 0;
                self.label2.text.clear();
                self.label3.text.clear();
            }
        }
        self.text_box1.text.clear();
    }
}
