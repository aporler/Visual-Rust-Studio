//! Adds and removes items in a list (Vec<String>), with a small helper method.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        let task = self.text_box1.text.trim().to_owned();
        // Nothing to add if the text is empty.
        if task.is_empty() {
            return;
        }
        self.list_box1.items.push(task);
        self.text_box1.text.clear();
        self.update_count();
    }

    /// Handles the Click event of `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        // selected is an Option<usize>: Some(index) if an item is selected.
        if let Some(i) = self.list_box1.selected {
            self.list_box1.items.remove(i);
            self.list_box1.selected = None;
            self.update_count();
        }
    }

    /// Updates the label that shows the number of tasks.
    fn update_count(&mut self) {
        self.label1.text = format!("{} task(s)", self.list_box1.items.len());
    }
}
