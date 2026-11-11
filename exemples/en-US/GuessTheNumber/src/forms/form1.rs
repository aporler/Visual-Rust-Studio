//! A secret number, "higher / lower" hints, a try counter and a message box when you win.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {
    /// Number to guess.
    pub secret: u32,
    /// Number of tries.
    pub tries: u32,
}

/// Simple pseudo-random number from the clock (no external library).
fn random_1_to_100() -> u32 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42);
    nanos % 100 + 1
}

impl Form1 {
    /// Handles the Load event of `Form1`.
    pub fn form1_load(&mut self, app: &mut AppContext) {
        self.data.secret = random_1_to_100();
    }

    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        let Ok(guess) = self.text_box1.text.trim().parse::<u32>() else {
            self.label2.text = String::from("Please type a number.");
            return;
        };
        self.data.tries += 1;
        self.label3.text = format!("Tries : {}", self.data.tries);
        // match compares the guess with the secret number.
        match guess.cmp(&self.data.secret) {
            std::cmp::Ordering::Less => self.label2.text = String::from("Higher!"),
            std::cmp::Ordering::Greater => self.label2.text = String::from("Lower!"),
            std::cmp::Ordering::Equal => {
                app.message_box(format!("Well done! The number was {} ! Tries : {}", self.data.secret, self.data.tries));
                // New game.
                self.data.secret = random_1_to_100();
                self.data.tries = 0;
                self.label2.text.clear();
                self.label3.text.clear();
            }
        }
        self.text_box1.text.clear();
    }
}
