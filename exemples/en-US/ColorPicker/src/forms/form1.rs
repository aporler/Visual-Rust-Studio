//! Three sliders (red, green, blue) change the background color of a panel: ValueChanged events and the BackColor property.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {}

impl Form1 {
    /// Handles the Load event of `Form1`.
    pub fn form1_load(&mut self, app: &mut AppContext) {
        self.update_color();
    }

    /// Handles the ValueChanged event of `slider1`.
    pub fn slider1_value_changed(&mut self, app: &mut AppContext) {
        self.update_color();
    }

    /// Handles the ValueChanged event of `slider2`.
    pub fn slider2_value_changed(&mut self, app: &mut AppContext) {
        self.update_color();
    }

    /// Handles the ValueChanged event of `slider3`.
    pub fn slider3_value_changed(&mut self, app: &mut AppContext) {
        self.update_color();
    }

    /// Shows the color chosen with the three sliders.
    fn update_color(&mut self) {
        // Slider values are f32; `as u8` converts them to 0–255.
        let (r, g, b) = (
            self.slider1.value as u8,
            self.slider2.value as u8,
            self.slider3.value as u8,
        );
        self.panel1.back_color = Some(crate::vrs::Color::from_rgb(r, g, b));
        self.label1.text = format!("#{r:02X}{g:02X}{b:02X}  ({r}, {g}, {b})");
    }
}
