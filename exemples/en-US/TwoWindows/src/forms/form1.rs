//! Form1 opens a modal dialog (Form2) that asks for your name, then reads the answer.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Opens Form2 as a modal dialog; the closure runs when it closes.
        app.show_dialog_then(super::Form2::default(), |me: &mut Form1, form2: super::Form2, _app| {
            // me is this window (Form1), form2 is the closed dialog.
            if form2.data.accepted && !form2.text_box1.text.trim().is_empty() {
                me.label1.text = format!("Hello, {} !", form2.text_box1.text.trim());
            }
        });
    }
}
