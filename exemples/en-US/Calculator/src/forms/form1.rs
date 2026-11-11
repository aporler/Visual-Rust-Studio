//! Two numbers, an operation chosen in a ComboBox, match on the selection, and a message box for division by zero.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

/// Converts the text to a number: Some(number), or None if the text is not a number.
fn number(text: &str) -> Option<f64> {
    text.trim().replace(',', ".").parse().ok()
}

impl Form1 {
    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        let (Some(a), Some(b)) = (number(&self.text_box1.text), number(&self.text_box2.text)) else {
            self.label1.text = String::from("Please type a number.");
            return;
        };
        // The selected operation (index in the ComboBox).
        let result = match self.combo_box1.selected {
            Some(0) => a + b,
            Some(1) => a - b,
            Some(2) => a * b,
            Some(3) if b == 0.0 => {
                app.message_box("Division by zero is impossible.");
                return;
            }
            Some(3) => a / b,
            _ => return,
        };
        self.label1.text = format!("= {result}");
    }
}
