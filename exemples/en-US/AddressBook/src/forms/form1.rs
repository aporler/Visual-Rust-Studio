//! A contact list; a second window (modal dialog) asks for the name and phone number.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // Opens Form2 as a modal dialog; the closure runs when it closes.
        app.show_dialog_then(super::Form2::default(), |me: &mut Form1, contact: super::Form2, _app| {
            let name = contact.text_box1.text.trim();
            let phone = contact.text_box2.text.trim();
            if contact.data.accepted && !name.is_empty() {
                me.list_box1.items.push(format!("{name} — {phone}"));
                me.list_box1.items.sort();
            }
        });
    }

    /// Handles the Click event of `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        if let Some(i) = self.list_box1.selected.take() {
            self.list_box1.items.remove(i);
        }
    }
}
