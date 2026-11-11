//! Reads a text box, converts the text to a number and handles input errors (°C to °F).
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Read the text typed by the user (a comma is accepted as decimal separator).
        let input = self.text_box1.text.trim().replace(',', ".");
        // parse() returns Ok(number) or Err(error): both cases must be handled.
        match input.parse::<f64>() {
            Ok(celsius) => {
                let fahrenheit = celsius * 9.0 / 5.0 + 32.0;
                self.label2.text = format!("{celsius} °C = {fahrenheit:.1} °F");
            }
            Err(_) => {
                self.label2.text = String::from("Please type a number.");
            }
        }
    }
}
