//! A multiline text box, a File menu (New, Open, Save, Quit) and reading/writing a file with std::fs.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

/// File used to open and save the text (in the current folder).
const FILE: &str = "notes.txt";

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Handles the Click event of `menu_new`.
    pub fn menu_new_click(&mut self, app: &mut AppContext) {
        self.text_box1.text.clear();
        self.label1.text.clear();
    }

    /// Handles the Click event of `menu_open`.
    pub fn menu_open_click(&mut self, app: &mut AppContext) {
        // read_to_string returns Ok(text) or Err(error): the file may not exist yet.
        match std::fs::read_to_string(FILE) {
            Ok(text) => {
                self.text_box1.text = text;
                self.label1.text = format!("Opened: {}", FILE);
            }
            Err(e) => self.label1.text = format!("Error: {}", e),
        }
    }

    /// Handles the Click event of `menu_save`.
    pub fn menu_save_click(&mut self, app: &mut AppContext) {
        match std::fs::write(FILE, &self.text_box1.text) {
            Ok(()) => self.label1.text = format!("Saved in {}", FILE),
            Err(e) => self.label1.text = format!("Error: {}", e),
        }
    }

    /// Handles the Click event of `menu_quit`.
    pub fn menu_quit_click(&mut self, app: &mut AppContext) {
        app.exit();
    }
}
