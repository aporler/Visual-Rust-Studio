//! A window variable (self.data) increases on each click; a second button resets it.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

/// Variables of this window (like the variables declared in a VB.NET form). Add your fields here and use them with `self.data`.
#[derive(Default)]
pub struct Form1Data {
    /// Number of clicks.
    pub count: u32,
}

impl Form1 {
    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Window variables are in self.data (see Form1Data above).
        self.data.count += 1;
        self.label1.text = self.data.count.to_string();
    }

    /// Handles the Click event of `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        self.data.count = 0;
        self.label1.text = String::from("0");
    }
}
