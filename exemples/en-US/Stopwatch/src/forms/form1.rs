//! A Timer raises Tick every 100 ms: start, stop, reset and a progress bar for the seconds.
#![allow(unused_imports, unused_variables)]

use super::Form1;
use crate::vrs::{AppContext, egui};

#[derive(Default)]
pub struct Form1Data {
    /// Elapsed time, in tenths of a second.
    pub tenths: u32,
}

impl Form1 {
    /// Handles the Tick event of `timer1`.
    pub fn timer1_tick(&mut self, app: &mut AppContext) {
        self.data.tenths += 1;
        self.show_time();
    }

    /// Handles the Click event of `button1`.
    pub fn button1_click(&mut self, app: &mut AppContext) {
        // enabled starts or stops the Timer.
        self.timer1.enabled = true;
    }

    /// Handles the Click event of `button2`.
    pub fn button2_click(&mut self, app: &mut AppContext) {
        self.timer1.enabled = false;
    }

    /// Handles the Click event of `button3`.
    pub fn button3_click(&mut self, app: &mut AppContext) {
        self.data.tenths = 0;
        self.show_time();
    }

    /// Shows the time as mm:ss.t (`:02` adds leading zeros).
    fn show_time(&mut self) {
        let minutes = self.data.tenths / 600;
        let seconds = (self.data.tenths / 10) % 60;
        let tenths = self.data.tenths % 10;
        self.label1.text = format!("{minutes:02}:{seconds:02}.{tenths}");
        self.progress_bar1.value = seconds as f32 + tenths as f32 / 10.0;
    }
}
