//! A contact list; a second window (modal dialog) asks for the name and phone number.
#![allow(unused_imports, unused_variables)]

use super::Form2;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form2Data {
    /// true if the user clicked OK.
    pub accepted: bool,
}

impl Form2 {
    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        self.data.accepted = true;
        app.close();
    }

    /// Handles the Click event of `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        app.close();
    }
}
